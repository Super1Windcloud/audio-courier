#![allow(clippy::collapsible_if)]

/// https://developers.deepgram.com/reference/speech-to-text-api/listen-streaming
use crate::provider_config::{
    TranscriptRuntimeConfig, resolve_optional_string, resolve_required_string,
};
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
use tokio::sync::{Mutex as AsyncMutex, mpsc, mpsc::error::TrySendError, oneshot, watch};
use tokio::time::{self, Duration};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        client::{ClientRequestBuilder, IntoClientRequest},
        protocol::Message,
    },
};

const BASE_URL: &str = "wss://api.deepgram.com/v1/listen";
const KEEPALIVE_INTERVAL_SECONDS: u64 = 3;
const DEFAULT_ENDPOINTING_MS: u32 = 500;
const DEFAULT_UTTERANCE_END_MS: u32 = 1_000;
const VENDOR_NAME: &str = "Deepgram";

enum StreamCommand {
    Audio(Vec<u8>),
    Finalize,
}

pub struct DeepgramApiTranscriber {
    sender: Mutex<Option<mpsc::Sender<StreamCommand>>>,
    shutdown: Mutex<Option<oneshot::Sender<()>>>,
    handle: Mutex<Option<JoinHandle<()>>>,
    stop_requested: Arc<AtomicBool>,
}

impl DeepgramApiTranscriber {
    pub fn start(
        sample_rate: u32,
        callback: PcmCallback,
        status_callback: Option<StatusCallback>,
        transcript_config: TranscriptRuntimeConfig,
    ) -> Result<Self, String> {
        let api_key = resolve_required_string(
            transcript_config.deepgram_api_key.as_deref(),
            &["DEEPGRAM_API_KEY"],
            "DEEPGRAM_API_KEY",
        )?;
        let language = resolve_optional_string(
            transcript_config.deepgram_language.as_deref(),
            &["DEEPGRAM_LANGUAGE"],
        );

        let (sender, receiver) = mpsc::channel::<StreamCommand>(64);
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let stop_requested = Arc::new(AtomicBool::new(false));
        let stop_requested_for_thread = stop_requested.clone();

        let handle = thread::spawn(move || {
            let runtime = Runtime::new().expect("Failed to build Tokio runtime");
            if let Err(err) = runtime.block_on(run_session(
                api_key,
                language,
                sample_rate,
                callback,
                receiver,
                shutdown_rx,
                stop_requested_for_thread,
            )) {
                if let Some(cb) = status_callback.as_ref() {
                    cb(format!("deepgram_api: {err}"));
                }
                eprintln!("Deepgram API streaming error: {err}");
            }
        });

        Ok(Self {
            sender: Mutex::new(Some(sender)),
            shutdown: Mutex::new(Some(shutdown_tx)),
            handle: Mutex::new(Some(handle)),
            stop_requested,
        })
    }

    pub fn enqueue_chunk(&self, chunk: Vec<i16>) -> Result<(), String> {
        let sender = self
            .sender
            .lock()
            .unwrap()
            .as_ref()
            .cloned()
            .ok_or_else(|| "Deepgram API transcriber is not running".to_string())?;

        let mut bytes = Vec::with_capacity(chunk.len() * 2);
        for sample in chunk {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }

        match sender.try_send(StreamCommand::Audio(bytes)) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_chunk)) => {
                Err("Failed to queue PCM chunk for Deepgram API: channel is full".into())
            }
            Err(TrySendError::Closed(_chunk)) => {
                Err("Failed to queue PCM chunk for Deepgram API: channel closed".into())
            }
        }
    }

    pub fn request_finalize(&self) -> Result<(), String> {
        let sender = self
            .sender
            .lock()
            .unwrap()
            .as_ref()
            .cloned()
            .ok_or_else(|| "Deepgram API transcriber is not running".to_string())?;

        sender
            .blocking_send(StreamCommand::Finalize)
            .map_err(|e| format!("Failed to queue Deepgram Finalize: {e}"))
    }

    pub fn stop(&self) {
        self.stop_requested.store(true, Ordering::SeqCst);
        self.sender.lock().unwrap().take();
        if let Some(shutdown) = self.shutdown.lock().unwrap().take() {
            let _ = shutdown.send(());
        }

        if let Some(handle) = self.handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
}

impl Drop for DeepgramApiTranscriber {
    fn drop(&mut self) {
        self.stop();
    }
}

impl StreamingTranscriber for DeepgramApiTranscriber {
    fn queue_chunk(&self, chunk: Vec<i16>) -> Result<(), String> {
        self.enqueue_chunk(chunk)
    }

    fn get_vendor_name(&self) -> String {
        VENDOR_NAME.to_string()
    }

    fn force_endpoint(&self) -> Result<(), String> {
        self.request_finalize()
    }

    fn shutdown(&self) {
        self.stop();
        println!("Deepgram API websocket shutdown invoked");
    }
}

async fn run_session(
    api_key: String,
    language: Option<String>,
    sample_rate: u32,
    callback: PcmCallback,
    mut audio_rx: mpsc::Receiver<StreamCommand>,
    mut shutdown_rx: oneshot::Receiver<()>,
    stop_requested: Arc<AtomicBool>,
) -> Result<(), String> {
    let url = build_streaming_url(language.as_deref(), sample_rate);
    let uri: Uri = url
        .parse()
        .map_err(|e| format!("Failed to parse Deepgram streaming URI: {e}"))?;
    let builder =
        ClientRequestBuilder::new(uri).with_header("Authorization", format!("Token {api_key}"));
    let client_request = builder
        .into_client_request()
        .map_err(|e| format!("Failed to build Deepgram websocket request: {e}"))?;

    let (ws_stream, _) = connect_async(client_request)
        .await
        .map_err(|e| format!("Failed to connect to Deepgram API: {e}"))?;

    let (mut sink, mut stream) = ws_stream.split();
    let (termination_tx, mut termination_rx) = watch::channel(false);
    let close_sent = Arc::new(AtomicBool::new(false));
    let utterance_buffer = Arc::new(AsyncMutex::new(String::new()));

    let send_audio = {
        let close_sent = close_sent.clone();
        async move {
            let mut keepalive = time::interval(Duration::from_secs(KEEPALIVE_INTERVAL_SECONDS));
            keepalive.set_missed_tick_behavior(time::MissedTickBehavior::Delay);
            keepalive.tick().await;

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => break,
                    result = termination_rx.changed() => {
                        if result.is_err() || *termination_rx.borrow() {
                            return Ok::<(), String>(());
                        }
                    }
                    _ = keepalive.tick() => {
                        sink.send(Message::Text(json!({"type": "KeepAlive"}).to_string().into()))
                            .await
                            .map_err(|e| format!("Failed to send Deepgram KeepAlive: {e}"))?;
                    }
                    command = audio_rx.recv() => match command {
                        Some(StreamCommand::Audio(bytes)) => {
                            sink.send(Message::Binary(bytes.into()))
                                .await
                                .map_err(|e| format!("Failed to send audio chunk to Deepgram: {e}"))?;
                        }
                        Some(StreamCommand::Finalize) => {
                            sink.send(Message::Text(json!({"type": "Finalize"}).to_string().into()))
                                .await
                                .map_err(|e| format!("Failed to send Deepgram Finalize: {e}"))?;
                        }
                        None => break,
                    }
                }
            }

            close_sent.store(true, Ordering::SeqCst);
            sink.send(Message::Text(
                json!({"type": "CloseStream"}).to_string().into(),
            ))
            .await
            .map_err(|e| format!("Failed to send Deepgram CloseStream: {e}"))?;
            sink.close()
                .await
                .map_err(|e| format!("Failed to close Deepgram socket: {e}"))?;
            Ok::<(), String>(())
        }
    };

    let receive_events = {
        let callback = callback.clone();
        let termination_tx = termination_tx.clone();
        let close_sent = close_sent.clone();
        let utterance_buffer = utterance_buffer.clone();

        async move {
            while let Some(message) = stream.next().await {
                let message = match message {
                    Ok(message) => message,
                    Err(err) => {
                        let _ = termination_tx.send(true);
                        return Err(format!("Deepgram receive error: {err}"));
                    }
                };

                match message {
                    Message::Text(payload) => {
                        let value: Value = serde_json::from_str(&payload)
                            .map_err(|e| format!("Failed to parse Deepgram payload: {e}"))?;

                        match value.get("type").and_then(|entry| entry.as_str()) {
                            Some("Results") => {
                                if let Some(transcript) = extract_transcript(&value) {
                                    let is_final = value
                                        .get("is_final")
                                        .and_then(|entry| entry.as_bool())
                                        .unwrap_or(false);
                                    let speech_final = value
                                        .get("speech_final")
                                        .and_then(|entry| entry.as_bool())
                                        .unwrap_or(false);

                                    if is_final {
                                        append_utterance_segment(
                                            &utterance_buffer,
                                            transcript.as_str(),
                                        )
                                        .await;
                                        emit_buffer_as_draft(&utterance_buffer, &callback).await;
                                    } else {
                                        emit_buffer_with_segment_as_draft(
                                            &utterance_buffer,
                                            &callback,
                                            transcript.as_str(),
                                        )
                                        .await;
                                    }

                                    if speech_final {
                                        flush_utterance(&utterance_buffer, &callback).await;
                                    }
                                } else if value
                                    .get("speech_final")
                                    .and_then(|entry| entry.as_bool())
                                    .unwrap_or(false)
                                {
                                    flush_utterance(&utterance_buffer, &callback).await;
                                }
                            }
                            Some("UtteranceEnd") => {
                                flush_utterance(&utterance_buffer, &callback).await;
                            }
                            Some("Metadata") => {
                                continue;
                            }
                            Some("Error") => {
                                let _ = termination_tx.send(true);
                                let reason = value
                                    .get("description")
                                    .and_then(|entry| entry.as_str())
                                    .unwrap_or("unknown Deepgram error");
                                return Err(format!("Deepgram returned error: {reason}"));
                            }
                            _ => {}
                        }
                    }
                    Message::Close(_) => {
                        let _ = termination_tx.send(true);
                        flush_utterance(&utterance_buffer, &callback).await;
                        if close_sent.load(Ordering::SeqCst)
                            || stop_requested.load(Ordering::SeqCst)
                        {
                            return Ok::<(), String>(());
                        }
                        return Err("Deepgram websocket closed unexpectedly".into());
                    }
                    _ => {}
                }
            }

            let _ = termination_tx.send(true);
            flush_utterance(&utterance_buffer, &callback).await;
            if close_sent.load(Ordering::SeqCst) || stop_requested.load(Ordering::SeqCst) {
                Ok::<(), String>(())
            } else {
                Err("Deepgram websocket closed unexpectedly".into())
            }
        }
    };

    try_join(send_audio, receive_events).await?;
    Ok(())
}

fn build_streaming_url(language: Option<&str>, sample_rate: u32) -> String {
    let model = select_model(language);
    let mut query = vec![
        ("model", model.to_string()),
        ("encoding", "linear16".to_string()),
        ("sample_rate", sample_rate.to_string()),
        ("channels", "1".to_string()),
        ("endpointing", DEFAULT_ENDPOINTING_MS.to_string()),
        ("interim_results", "true".to_string()),
        ("utterance_end_ms", DEFAULT_UTTERANCE_END_MS.to_string()),
        ("smart_format", "false".to_string()),
        ("punctuate", "false".to_string()),
    ];

    if let Some(language) = normalize_language(language) {
        query.push(("language", language));
    }

    let query = query
        .into_iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&");

    format!("{BASE_URL}?{query}")
}

fn normalize_language(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim();
    if trimmed.is_empty() {
        return None;
    }

    let normalized = match trimmed.replace('_', "-").to_lowercase().as_str() {
        "zh" => "zh".to_string(),
        "zh-cn" => "zh-CN".to_string(),
        "en" => "en".to_string(),
        _ => trimmed.replace('_', "-"),
    };

    Some(normalized)
}

fn select_model(language: Option<&str>) -> &'static str {
    match normalize_language(language).as_deref() {
        Some("zh") | Some("zh-CN") | None => "nova-2",
        _ => "nova-3",
    }
}

fn extract_transcript(value: &Value) -> Option<String> {
    value
        .get("channel")
        .and_then(|channel| channel.get("alternatives"))
        .and_then(|alternatives| alternatives.as_array())
        .and_then(|alternatives| alternatives.first())
        .and_then(|entry| entry.get("transcript"))
        .and_then(|entry| entry.as_str())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(ToString::to_string)
}

async fn append_utterance_segment(buffer: &AsyncMutex<String>, segment: &str) {
    let mut guard = buffer.lock().await;
    if !guard.is_empty() {
        guard.push(' ');
    }
    guard.push_str(segment);
}

async fn emit_buffer_as_draft(buffer: &AsyncMutex<String>, callback: &PcmCallback) {
    let guard = buffer.lock().await;
    let trimmed = guard.trim();
    if trimmed.is_empty() {
        return;
    }

    emit_draft(callback, VENDOR_NAME, trimmed);
}

async fn emit_buffer_with_segment_as_draft(
    buffer: &AsyncMutex<String>,
    callback: &PcmCallback,
    segment: &str,
) {
    let guard = buffer.lock().await;
    let merged = merge_segments(guard.as_str(), segment);
    if merged.is_empty() {
        return;
    }

    emit_draft(callback, VENDOR_NAME, merged);
}

async fn flush_utterance(buffer: &AsyncMutex<String>, callback: &PcmCallback) {
    let mut guard = buffer.lock().await;
    let trimmed = guard.trim();
    if trimmed.is_empty() {
        guard.clear();
        return;
    }

    emit_commit(callback, VENDOR_NAME, trimmed);
    guard.clear();
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

#[cfg(test)]
mod tests {
    use super::{build_streaming_url, extract_transcript, normalize_language, select_model};
    use serde_json::json;

    #[test]
    fn build_streaming_url_uses_expected_v1_endpoint() {
        let url = build_streaming_url(Some("zh_CN"), 16_000);

        assert!(url.starts_with("wss://api.deepgram.com/v1/listen?"));
        assert!(url.contains("model=nova-2"));
        assert!(url.contains("encoding=linear16"));
        assert!(url.contains("sample_rate=16000"));
        assert!(url.contains("channels=1"));
        assert!(url.contains("endpointing=500"));
        assert!(url.contains("interim_results=true"));
        assert!(url.contains("utterance_end_ms=1000"));
        assert!(url.contains("language=zh-CN"));
    }

    #[test]
    fn model_selection_matches_existing_language_behavior() {
        assert_eq!(select_model(None), "nova-2");
        assert_eq!(select_model(Some("zh")), "nova-2");
        assert_eq!(select_model(Some("zh_CN")), "nova-2");
        assert_eq!(select_model(Some("en")), "nova-3");
    }

    #[test]
    fn normalize_language_preserves_bcp47_shape() {
        assert_eq!(normalize_language(Some("zh_CN")), Some("zh-CN".to_string()));
        assert_eq!(normalize_language(Some("en-US")), Some("en-US".to_string()));
    }

    #[test]
    fn extract_transcript_reads_first_alternative() {
        let value = json!({
            "type": "Results",
            "channel": {
                "alternatives": [
                    { "transcript": " hello world " }
                ]
            }
        });

        assert_eq!(extract_transcript(&value), Some("hello world".to_string()));
    }
}
