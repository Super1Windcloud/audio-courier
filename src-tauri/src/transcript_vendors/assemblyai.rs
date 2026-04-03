#![allow(clippy::collapsible_if)]

///https://www.assemblyai.com/docs/api-reference/streaming-api/universal-streaming/universal-streaming
use crate::provider_config::{TranscriptRuntimeConfig, resolve_required_string};
use crate::transcript_vendors::{
    PcmCallback, StatusCallback, StreamingTranscriber, emit_commit, emit_draft,
};
use futures_util::{SinkExt, StreamExt, future::try_join};
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use tauri::http::Uri;
use tokio::runtime::Runtime;
use tokio::sync::{Mutex as AsyncMutex, mpsc, oneshot, watch};
use tokio::time::{self, MissedTickBehavior};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tungstenite::client::{ClientRequestBuilder, IntoClientRequest};

const BASE_URL: &str = "wss://streaming.assemblyai.com/v3/ws";
const SPEECH_MODEL: &str = "whisper-rt";
const AUDIO_ENCODING: &str = "pcm_s16le";
const MIN_TURN_SILENCE_MS: u32 = 600;
const INACTIVITY_TIMEOUT_SECS: u32 = 3600;
const HEARTBEAT_INTERVAL_SECS: u64 = 20;

enum StreamCommand {
    Audio(Vec<i16>),
    ForceEndpoint,
}

pub struct AssemblyAiTranscriber {
    sender: mpsc::Sender<StreamCommand>,
    shutdown: Mutex<Option<oneshot::Sender<()>>>,
    handle: Mutex<Option<JoinHandle<()>>>,
    stop_requested: Arc<AtomicBool>,
}

impl AssemblyAiTranscriber {
    pub fn start(
        sample_rate: u32,
        callback: PcmCallback,
        status_callback: Option<StatusCallback>,
        transcript_config: TranscriptRuntimeConfig,
    ) -> Result<Self, String> {
        let api_key = resolve_required_string(
            transcript_config.assembly_api_key.as_deref(),
            &["ASSEMBLY_API_KEY"],
            "ASSEMBLY_API_KEY",
        )?;

        let (sender, receiver) = mpsc::channel::<StreamCommand>(64);
        let (shutdown, shutdown_rx) = oneshot::channel::<()>();
        let stop_requested = Arc::new(AtomicBool::new(false));
        let stop_requested_for_thread = stop_requested.clone();

        let handle = thread::spawn(move || {
            let runtime = Runtime::new().expect("Failed to build Tokio runtime");
            if let Err(err) = runtime.block_on(run_stream(
                api_key,
                sample_rate,
                callback,
                receiver,
                shutdown_rx,
                stop_requested_for_thread,
            )) {
                if let Some(cb) = status_callback.as_ref() {
                    cb(format!("assemblyai: {err}"));
                }
                eprintln!("AssemblyAI streaming error: {err}");
            }
        });

        Ok(Self {
            sender,
            shutdown: Mutex::new(Some(shutdown)),
            handle: Mutex::new(Some(handle)),
            stop_requested,
        })
    }

    pub fn enqueue_chunk(&self, chunk: Vec<i16>) -> Result<(), String> {
        self.sender
            .blocking_send(StreamCommand::Audio(chunk))
            .map_err(|e| format!("Failed to queue PCM chunk for AssemblyAI: {e}"))
    }

    pub fn request_force_endpoint(&self) -> Result<(), String> {
        self.sender
            .blocking_send(StreamCommand::ForceEndpoint)
            .map_err(|e| format!("Failed to queue AssemblyAI force endpoint: {e}"))
    }

    pub fn stop(&self) {
        self.stop_requested.store(true, Ordering::SeqCst);
        if let Some(shutdown) = self.shutdown.lock().unwrap().take() {
            let _ = shutdown.send(());
        }

        if let Some(handle) = self.handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
}

impl Drop for AssemblyAiTranscriber {
    fn drop(&mut self) {
        self.stop();
    }
}

async fn run_stream(
    api_key: String,
    sample_rate: u32,
    callback: PcmCallback,
    mut audio_rx: mpsc::Receiver<StreamCommand>,
    mut shutdown_rx: oneshot::Receiver<()>,
    stop_requested: Arc<AtomicBool>,
) -> Result<(), String> {
    let query = format!(
        "sample_rate={sample_rate}&speech_model={SPEECH_MODEL}&encoding={AUDIO_ENCODING}&format_turns=true&min_turn_silence={MIN_TURN_SILENCE_MS}&inactivity_timeout={INACTIVITY_TIMEOUT_SECS}"
    );
    let url = format!("{BASE_URL}?{query}");

    let uri: Uri = url
        .parse()
        .map_err(|e| format!("Failed to parse streaming URI: {e}"))?;
    let builder = ClientRequestBuilder::new(uri)
        .with_header("Authorization", api_key)
        .with_header("Content-Type", "application/json");
    let client_request = builder
        .into_client_request()
        .map_err(|e| format!("Failed to build websocket request: {e}"))?;

    let (ws_stream, _) = connect_async(client_request)
        .await
        .map_err(|e| format!("Failed to connect to AssemblyAI: {e}"))?;

    let (mut sink, mut stream) = ws_stream.split();
    let (termination_tx, mut termination_rx) = watch::channel(false);

    let send_audio = async {
        let mut heartbeat = time::interval(std::time::Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
        heartbeat.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = &mut shutdown_rx => break,
                result = termination_rx.changed() => {
                    if result.is_err() || *termination_rx.borrow() {
                        break;
                    }
                },
                chunk = audio_rx.recv() => match chunk {
                    Some(StreamCommand::Audio(samples)) => {
                        let audio_bytes = samples
                            .iter()
                            .flat_map(|sample| sample.to_le_bytes())
                            .collect::<Vec<u8>>();

                        sink.send(Message::Binary(audio_bytes.into()))
                            .await
                            .map_err(|e| format!("Failed to send audio chunk: {e}"))?;
                    }
                    Some(StreamCommand::ForceEndpoint) => {
                        let payload = json!({ "type": "ForceEndpoint" });
                        sink.send(Message::Text(payload.to_string().into()))
                            .await
                            .map_err(|e| format!("Failed to send AssemblyAI force endpoint: {e}"))?;
                    }
                    None => break,
                },
                _ = heartbeat.tick() => {
                    sink.send(Message::Ping(Vec::new().into()))
                        .await
                        .map_err(|e| format!("Failed to send AssemblyAI heartbeat ping: {e}"))?;
                }
            }
        }

        let terminate_payload = json!({ "type": "Terminate" });

        sink.send(Message::Text(terminate_payload.to_string().into()))
            .await
            .map_err(|e| format!("Failed to send termination payload: {e}"))?;

        sink.close()
            .await
            .map_err(|e| format!("Failed to close AssemblyAI socket: {e}"))?;

        println!("AssemblyAI websocket stop completed!");

        Ok::<(), String>(())
    };

    let receive_events = {
        let callback = callback.clone();
        let termination_tx = termination_tx.clone();

        async move {
            let mut last_emitted: Option<(bool, String)> = None;
            let utterance_buffer = Arc::new(AsyncMutex::new(String::new()));

            while let Some(message) = stream.next().await {
                let message = message.map_err(|e| {
                    let _ = termination_tx.send(true);
                    format!("AssemblyAI receive error: {e}")
                })?;
                match message {
                    Message::Text(payload) => {
                        if let Ok(value) = serde_json::from_str::<Value>(&payload) {
                            match resolve_event_type(&value) {
                                Some("Termination") => {
                                    let _ = termination_tx.send(true);
                                    if stop_requested.load(Ordering::SeqCst) {
                                        break;
                                    }
                                    return Err(
                                        "AssemblyAI websocket terminated unexpectedly".into()
                                    );
                                }
                                _ => {
                                    let transcripts = extract_transcripts(&value);
                                    for (transcript, is_final) in transcripts {
                                        let normalized = normalize_transcript_text(&transcript);
                                        if normalized.is_empty() {
                                            continue;
                                        }

                                        let next_event =
                                            (is_final, normalize_transcript_dedup_key(&normalized));
                                        if last_emitted.as_ref() == Some(&next_event) {
                                            continue;
                                        }

                                        last_emitted = Some(next_event);
                                        if is_final {
                                            let commit = {
                                                let mut buffer = utterance_buffer.lock().await;
                                                append_utterance_segment(&mut buffer, &normalized);
                                                buffer.trim().to_string()
                                            };
                                            emit_commit(&callback, "AssemblyAI", &commit);
                                            utterance_buffer.lock().await.clear();
                                        } else {
                                            let draft = {
                                                let buffer = utterance_buffer.lock().await;
                                                merge_segments(&buffer, &normalized)
                                            };
                                            emit_draft(&callback, "AssemblyAI", &draft);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Message::Close(frame) => {
                        let _ = termination_tx.send(true);
                        if stop_requested.load(Ordering::SeqCst) {
                            break;
                        }
                        return Err(describe_close_frame("AssemblyAI", frame.as_ref()));
                    }
                    _ => {}
                }
            }

            if !stop_requested.load(Ordering::SeqCst) {
                let _ = termination_tx.send(true);
                return Err(
                    "AssemblyAI websocket closed unexpectedly without a close frame".into(),
                );
            }

            let _ = termination_tx.send(true);
            Ok::<(), String>(())
        }
    };

    try_join(send_audio, receive_events).await?;
    Ok(())
}

fn describe_close_frame(vendor: &str, frame: Option<&tungstenite::protocol::CloseFrame>) -> String {
    match frame {
        Some(frame) => format!(
            "{vendor} websocket closed unexpectedly (code={:?}, reason={})",
            frame.code, frame.reason
        ),
        None => format!("{vendor} websocket closed unexpectedly without a close frame"),
    }
}

fn extract_transcripts(value: &Value) -> Vec<(String, bool)> {
    let mut transcripts = Vec::new();
    let event_type = resolve_event_type(value);

    match event_type {
        Some("Turn") => transcripts.extend(extract_turn_transcripts(value)),
        Some("FinalTranscript") => transcripts.extend(extract_plain_transcripts(value, true)),
        Some("PartialTranscript") => transcripts.extend(extract_plain_transcripts(value, false)),
        Some("Transcript") => transcripts.extend(extract_plain_transcripts(value, false)),
        _ => transcripts.extend(extract_plain_transcripts(value, false)),
    }

    if transcripts.is_empty() {
        transcripts.extend(extract_nested_turns(value));
    }

    transcripts
}

fn resolve_event_type(value: &Value) -> Option<&str> {
    value
        .get("type")
        .or_else(|| value.get("message_type"))
        .and_then(|entry| entry.as_str())
}

fn extract_turn_transcripts(value: &Value) -> Vec<(String, bool)> {
    let text = first_non_empty_text(value, &["utterance", "transcript"]);
    let Some(text) = text else {
        return Vec::new();
    };

    if !bool_flag(value, &["end_of_turn", "turn_is_final"], false) {
        return vec![(text.to_string(), false)];
    }

    vec![(text.to_string(), true)]
}

fn extract_plain_transcripts(value: &Value, treat_type_as_final: bool) -> Vec<(String, bool)> {
    let is_final = if treat_type_as_final {
        true
    } else {
        bool_flag(
            value,
            &["is_final", "final", "end_of_turn", "turn_is_final"],
            false,
        )
    };

    first_non_empty_text(value, &["text", "transcript", "utterance"])
        .map(|text| vec![(text.to_string(), is_final)])
        .unwrap_or_default()
}

fn extract_nested_turns(value: &Value) -> Vec<(String, bool)> {
    let turns = value
        .get("conversation")
        .or_else(|| value.get("turns"))
        .and_then(|turns| turns.as_array());

    let mut transcripts = Vec::new();
    if let Some(turns) = turns {
        for turn in turns {
            if bool_flag(turn, &["turn_is_final", "is_final", "end_of_turn"], true) {
                if let Some(text) = first_non_empty_text(
                    turn,
                    &[
                        "utterance",
                        "transcript",
                        "formatted_text",
                        "formatted_transcript",
                        "text",
                    ],
                ) {
                    transcripts.push((text.to_string(), true));
                }
            }
        }
    }

    transcripts
}

fn bool_flag(value: &Value, keys: &[&str], default: bool) -> bool {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(|flag| flag.as_bool()))
        .unwrap_or(default)
}

fn first_non_empty_text<'a>(value: &'a Value, fields: &[&str]) -> Option<&'a str> {
    fields.iter().find_map(|field| {
        value
            .get(*field)
            .and_then(|entry| entry.as_str())
            .map(str::trim)
            .filter(|text| !text.is_empty())
    })
}

fn normalize_transcript_text(input: &str) -> String {
    let chars = input.trim().chars().collect::<Vec<_>>();
    let mut normalized = String::with_capacity(input.len());

    for (index, ch) in chars.iter().enumerate() {
        if ch.is_whitespace() {
            let prev = chars[..index]
                .iter()
                .rev()
                .find(|candidate| !candidate.is_whitespace())
                .copied();
            let next = chars[index + 1..]
                .iter()
                .find(|candidate| !candidate.is_whitespace())
                .copied();

            if matches!(
                (prev, next),
                (Some(left), Some(right)) if should_collapse_spacing(left, right)
            ) {
                continue;
            }

            if !normalized.ends_with(' ') && !normalized.is_empty() {
                normalized.push(' ');
            }
            continue;
        }

        normalized.push(*ch);
    }

    normalized.trim().to_string()
}

fn normalize_transcript_dedup_key(input: &str) -> String {
    input
        .chars()
        .filter(|ch| !ch.is_whitespace() && !is_dedup_ignorable_punctuation(*ch))
        .collect()
}

fn is_dedup_ignorable_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '.' | ',' | '!' | '?' | ':' | ';' | '，' | '。' | '！' | '？' | '：' | '；' | '、' | '…'
    )
}

fn should_collapse_spacing(left: char, right: char) -> bool {
    is_cjk(left)
        || is_cjk(right)
        || left.is_ascii_digit()
        || right.is_ascii_digit()
        || is_spacing_punctuation(left)
        || is_spacing_punctuation(right)
}

fn append_utterance_segment(buffer: &mut String, segment: &str) {
    let segment = segment.trim();
    if segment.is_empty() {
        return;
    }

    if buffer.is_empty() {
        buffer.push_str(segment);
        return;
    }

    if should_join_without_space(buffer, segment) {
        buffer.push_str(segment);
    } else {
        buffer.push(' ');
        buffer.push_str(segment);
    }
}

fn merge_segments(prefix: &str, suffix: &str) -> String {
    let prefix = prefix.trim();
    let suffix = suffix.trim();

    match (prefix.is_empty(), suffix.is_empty()) {
        (true, true) => String::new(),
        (false, true) => prefix.to_string(),
        (true, false) => suffix.to_string(),
        (false, false) => {
            if should_join_without_space(prefix, suffix) {
                format!("{prefix}{suffix}")
            } else {
                format!("{prefix} {suffix}")
            }
        }
    }
}

fn should_join_without_space(prefix: &str, suffix: &str) -> bool {
    let Some(last) = prefix.chars().next_back() else {
        return true;
    };
    let Some(first) = suffix.chars().next() else {
        return true;
    };

    last.is_whitespace()
        || first.is_whitespace()
        || is_cjk(last)
        || is_cjk(first)
        || is_spacing_punctuation(last)
        || is_spacing_punctuation(first)
}

fn is_cjk(ch: char) -> bool {
    matches!(
        ch as u32,
        0x4E00..=0x9FFF
            | 0x3400..=0x4DBF
            | 0x3040..=0x30FF
            | 0xAC00..=0xD7AF
            | 0xF900..=0xFAFF
    )
}

fn is_spacing_punctuation(ch: char) -> bool {
    matches!(
        ch,
        ',' | '.'
            | '!'
            | '?'
            | ':'
            | ';'
            | ')'
            | ']'
            | '}'
            | '，'
            | '。'
            | '！'
            | '？'
            | '：'
            | '；'
            | '）'
            | '】'
            | '」'
            | '、'
    )
}

impl StreamingTranscriber for AssemblyAiTranscriber {
    fn queue_chunk(&self, chunk: Vec<i16>) -> Result<(), String> {
        self.enqueue_chunk(chunk)
    }

    fn get_vendor_name(&self) -> String {
        "AssemblyAI".to_string()
    }

    fn force_endpoint(&self) -> Result<(), String> {
        self.request_force_endpoint()
    }

    fn shutdown(&self) {
        self.stop();
        println!("AssemblyAI websocket shutdown invoked");
    }
}

#[cfg(test)]
mod tests {
    use super::{
        append_utterance_segment, extract_plain_transcripts, extract_turn_transcripts,
        merge_segments, normalize_transcript_dedup_key, normalize_transcript_text,
    };
    use serde_json::json;

    #[test]
    fn turn_transcript_marks_non_final_turns_as_draft() {
        let value = json!({
            "type": "Turn",
            "utterance": "hello world",
            "end_of_turn": false
        });

        assert_eq!(
            extract_turn_transcripts(&value),
            vec![("hello world".to_string(), false)]
        );
    }

    #[test]
    fn plain_transcript_respects_final_flags() {
        let value = json!({
            "type": "PartialTranscript",
            "text": "hello world",
            "turn_is_final": true
        });

        assert_eq!(
            extract_plain_transcripts(&value, false),
            vec![("hello world".to_string(), true)]
        );
    }

    #[test]
    fn merge_segments_includes_prior_final_text_for_partial_drafts() {
        assert_eq!(
            merge_segments("hello world", "again"),
            "hello world again".to_string()
        );
    }

    #[test]
    fn append_utterance_segment_keeps_cjk_compact() {
        let mut buffer = String::new();
        append_utterance_segment(&mut buffer, "你好");
        append_utterance_segment(&mut buffer, "世界");

        assert_eq!(buffer, "你好世界".to_string());
    }

    #[test]
    fn normalize_transcript_text_compacts_spaced_cjk_tokens() {
        assert_eq!(
            normalize_transcript_text("虽 然 内 心 复 杂 . 却 不 得 不 听 从 莫 斯 科 的 指 令 。"),
            "虽然内心复杂.却不得不听从莫斯科的指令。".to_string()
        );
        assert_eq!(
            normalize_transcript_text("于 是 9 月 初"),
            "于是9月初".to_string()
        );
    }

    #[test]
    fn normalize_transcript_dedup_key_ignores_spacing_and_punctuation() {
        assert_eq!(
            normalize_transcript_dedup_key("中共与国民政府的接触正式开始"),
            normalize_transcript_dedup_key("中共与国民政府的接触正式开始.")
        );
        assert_eq!(
            normalize_transcript_dedup_key("事 实 上"),
            normalize_transcript_dedup_key("事实上。")
        );
    }
}
