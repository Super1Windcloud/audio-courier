#![allow(clippy::collapsible_if)]

/// https://docs.speechmatics.com/api-ref/realtime-transcription-websocket#addtranslation
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
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        client::{ClientRequestBuilder, IntoClientRequest},
        protocol::Message,
    },
};

const DEFAULT_RT_URL: &str = "wss://eu2.rt.speechmatics.com/v2/";
const DEFAULT_LANGUAGE: &str = "en";
const END_OF_UTTERANCE_SILENCE_TRIGGER: f32 = 0.5;
const MAX_DELAY_SECONDS: f32 = 1.5;

enum StreamCommand {
    Audio(Vec<u8>),
    ForceEndpoint,
}

pub struct SpeechmaticsTranscriber {
    sender: Mutex<Option<mpsc::Sender<StreamCommand>>>,
    shutdown: Mutex<Option<oneshot::Sender<()>>>,
    handle: Mutex<Option<JoinHandle<()>>>,
    stop_requested: Arc<AtomicBool>,
}

impl SpeechmaticsTranscriber {
    pub fn start(
        sample_rate: u32,
        callback: PcmCallback,
        status_callback: Option<StatusCallback>,
        transcript_config: TranscriptRuntimeConfig,
    ) -> Result<Self, String> {
        let api_key = resolve_required_string(
            transcript_config.speechmatics_api_key.as_deref(),
            &["SPEECHMATICS_API_KEY"],
            "SPEECHMATICS_API_KEY",
        )?;
        let language = resolve_optional_string(
            transcript_config.speechmatics_language.as_deref(),
            &["SPEECHMATICS_LANGUAGE"],
        );
        let url = resolve_optional_string(
            transcript_config.speechmatics_rt_url.as_deref(),
            &["SPEECHMATICS_RT_URL"],
        );

        let (sender, receiver) = mpsc::channel::<StreamCommand>(64);
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let callback_clone = callback.clone();
        let status_callback_clone = status_callback.clone();
        let stop_requested = Arc::new(AtomicBool::new(false));
        let stop_requested_for_thread = stop_requested.clone();

        let handle = thread::spawn(move || {
            let runtime = Runtime::new().expect("Failed to build Tokio runtime");
            if let Err(err) = runtime.block_on(run_session(
                api_key,
                url,
                language,
                sample_rate,
                callback_clone,
                receiver,
                shutdown_rx,
                stop_requested_for_thread,
            )) {
                if let Some(cb) = status_callback_clone.as_ref() {
                    cb(format!("speechmatics: {err}"));
                }
                eprintln!("Speechmatics streaming error: {err}");
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
            .ok_or_else(|| "Speechmatics transcriber is not running".to_string())?;

        let mut bytes = Vec::with_capacity(chunk.len() * 2);
        for sample in chunk {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }

        match sender.try_send(StreamCommand::Audio(bytes)) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_chunk)) => Err(
                "Failed to queue PCM chunk for Speechmatics: channel is full (consumer stalled)"
                    .into(),
            ),
            Err(TrySendError::Closed(_chunk)) => {
                Err("Failed to queue PCM chunk for Speechmatics: channel closed".into())
            }
        }
    }

    pub fn request_force_endpoint(&self) -> Result<(), String> {
        let sender = self
            .sender
            .lock()
            .unwrap()
            .as_ref()
            .cloned()
            .ok_or_else(|| "Speechmatics transcriber is not running".to_string())?;

        sender
            .blocking_send(StreamCommand::ForceEndpoint)
            .map_err(|e| format!("Failed to queue Speechmatics force endpoint: {e}"))
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

impl Drop for SpeechmaticsTranscriber {
    fn drop(&mut self) {
        self.stop();
    }
}

impl StreamingTranscriber for SpeechmaticsTranscriber {
    fn queue_chunk(&self, chunk: Vec<i16>) -> Result<(), String> {
        self.enqueue_chunk(chunk)
    }

    fn get_vendor_name(&self) -> String {
        "SpeechMatics".to_string()
    }

    fn force_endpoint(&self) -> Result<(), String> {
        self.request_force_endpoint()
    }

    fn shutdown(&self) {
        self.stop();
        println!("Speechmatics websocket shutdown invoked");
    }
}

async fn run_session(
    api_key: String,
    rt_url: Option<String>,
    language: Option<String>,
    sample_rate: u32,
    callback: PcmCallback,
    mut audio_rx: mpsc::Receiver<StreamCommand>,
    mut shutdown_rx: oneshot::Receiver<()>,
    stop_requested: Arc<AtomicBool>,
) -> Result<(), String> {
    let url = rt_url.unwrap_or_else(|| DEFAULT_RT_URL.to_string());
    let language = language.unwrap_or_else(|| DEFAULT_LANGUAGE.to_string());
    let uri: Uri = url
        .parse()
        .map_err(|e| format!("Failed to parse Speechmatics streaming URI: {e}"))?;
    let builder =
        ClientRequestBuilder::new(uri).with_header("Authorization", format!("Bearer {api_key}"));
    let client_request = builder
        .into_client_request()
        .map_err(|e| format!("Failed to build Speechmatics websocket request: {e}"))?;

    let (ws_stream, _) = connect_async(client_request)
        .await
        .map_err(|e| format!("Failed to connect to Speechmatics: {e}"))?;

    let (mut sink, mut stream) = ws_stream.split();
    let start_payload = build_start_recognition_payload(&language, sample_rate);

    sink.send(Message::Text(start_payload.to_string().into()))
        .await
        .map_err(|e| format!("Failed to send Speechmatics StartRecognition: {e}"))?;

    let (termination_tx, mut termination_rx) = watch::channel(false);
    let (started_tx, mut started_rx) = watch::channel(false);
    let utterance_buffer = Arc::new(AsyncMutex::new(String::new()));
    let last_partial = Arc::new(AsyncMutex::new(None::<String>));
    let eos_sent = Arc::new(AtomicBool::new(false));

    let send_audio = {
        let eos_sent = eos_sent.clone();
        async move {
            let mut chunk_seq_no = 0_i32;
            let mut total_samples_sent = 0_u64;

            loop {
                if *started_rx.borrow() {
                    break;
                }

                tokio::select! {
                    _ = &mut shutdown_rx => return Ok::<(), String>(()),
                    result = termination_rx.changed() => {
                        if result.is_err() || *termination_rx.borrow() {
                            return Ok::<(), String>(());
                        }
                    }
                    result = started_rx.changed() => {
                        if result.is_err() {
                            return Err("Speechmatics readiness watcher closed unexpectedly".into());
                        }
                    }
                }
            }

            let mut should_send_end_of_stream = true;

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => break,
                    result = termination_rx.changed() => {
                        if result.is_err() || *termination_rx.borrow() {
                            should_send_end_of_stream = false;
                            break;
                        }
                    }
                    command = audio_rx.recv() => match command {
                        Some(StreamCommand::Audio(bytes)) => {
                            total_samples_sent = total_samples_sent.saturating_add((bytes.len() / 2) as u64);
                            sink.send(Message::Binary(bytes.into()))
                                .await
                                .map_err(|e| format!("Failed to send audio chunk to Speechmatics: {e}"))?;
                            chunk_seq_no += 1;
                        }
                        Some(StreamCommand::ForceEndpoint) => {
                            let timestamp = total_samples_sent as f64 / sample_rate as f64;
                            let payload = json!({
                                "message": "ForceEndOfUtterance",
                                "timestamp": timestamp
                            });
                            sink.send(Message::Text(payload.to_string().into()))
                                .await
                                .map_err(|e| format!("Failed to send Speechmatics ForceEndOfUtterance: {e}"))?;
                        }
                        None => break,
                    }
                }
            }

            if should_send_end_of_stream {
                let payload = json!({
                    "message": "EndOfStream",
                    "last_seq_no": chunk_seq_no
                });
                sink.send(Message::Text(payload.to_string().into()))
                    .await
                    .map_err(|e| format!("Failed to send Speechmatics EndOfStream: {e}"))?;
                eos_sent.store(true, Ordering::SeqCst);
            }

            sink.close()
                .await
                .map_err(|e| format!("Failed to close Speechmatics socket: {e}"))?;
            Ok::<(), String>(())
        }
    };

    let receive_events = {
        let callback = callback.clone();
        let termination_tx = termination_tx.clone();
        let started_tx = started_tx.clone();
        let utterance_buffer = utterance_buffer.clone();
        let last_partial = last_partial.clone();
        let eos_sent = eos_sent.clone();
        let stop_requested = stop_requested.clone();

        async move {
            while let Some(message) = stream.next().await {
                let message = match message {
                    Ok(message) => message,
                    Err(err) => {
                        let _ = termination_tx.send(true);
                        return Err(format!("Speechmatics receive error: {err}"));
                    }
                };

                match message {
                    Message::Text(payload) => {
                        let value: Value = serde_json::from_str(&payload)
                            .map_err(|e| format!("Failed to parse Speechmatics payload: {e}"))?;
                        let message_type = value
                            .get("message")
                            .and_then(|entry| entry.as_str())
                            .unwrap_or_default();

                        match message_type {
                            "RecognitionStarted" => {
                                let _ = started_tx.send(true);
                            }
                            "AddPartialTranscript" | "AddPartialTranslation" => {
                                if let Some(text) = extract_payload_text(&value) {
                                    *last_partial.lock().await = Some(text.clone());
                                    let draft = {
                                        let buffer = utterance_buffer.lock().await;
                                        merge_segments(&buffer, &text)
                                    };
                                    emit_draft(&callback, "SpeechMatics", draft);
                                }
                            }
                            "AddTranscript" | "AddTranslation" => {
                                if let Some(text) = extract_payload_text(&value) {
                                    {
                                        let mut buffer = utterance_buffer.lock().await;
                                        append_utterance_segment(&mut buffer, &text);
                                        emit_draft(&callback, "SpeechMatics", buffer.trim());
                                    }
                                    *last_partial.lock().await = None;
                                }
                            }
                            "EndOfUtterance" => {
                                flush_current_utterance(
                                    &callback,
                                    &utterance_buffer,
                                    &last_partial,
                                )
                                .await;
                            }
                            "EndOfTranscript" => {
                                flush_current_utterance(
                                    &callback,
                                    &utterance_buffer,
                                    &last_partial,
                                )
                                .await;
                                let _ = termination_tx.send(true);
                                if eos_sent.load(Ordering::SeqCst)
                                    || stop_requested.load(Ordering::SeqCst)
                                {
                                    return Ok::<(), String>(());
                                }
                                return Err("Speechmatics websocket closed unexpectedly".into());
                            }
                            "Error" => {
                                let _ = termination_tx.send(true);
                                let reason = value
                                    .get("reason")
                                    .and_then(|entry| entry.as_str())
                                    .unwrap_or("unknown Speechmatics error");
                                return Err(format!("Speechmatics returned error: {reason}"));
                            }
                            _ => {}
                        }
                    }
                    Message::Close(_) => {
                        let _ = termination_tx.send(true);
                        flush_current_utterance(&callback, &utterance_buffer, &last_partial).await;
                        if eos_sent.load(Ordering::SeqCst) || stop_requested.load(Ordering::SeqCst)
                        {
                            return Ok::<(), String>(());
                        }
                        return Err("Speechmatics websocket closed unexpectedly".into());
                    }
                    _ => {}
                }
            }

            let _ = termination_tx.send(true);
            flush_current_utterance(&callback, &utterance_buffer, &last_partial).await;
            if eos_sent.load(Ordering::SeqCst) || stop_requested.load(Ordering::SeqCst) {
                Ok::<(), String>(())
            } else {
                Err("Speechmatics websocket closed unexpectedly".into())
            }
        }
    };

    try_join(send_audio, receive_events).await?;
    Ok(())
}

fn build_start_recognition_payload(language: &str, sample_rate: u32) -> Value {
    json!({
        "message": "StartRecognition",
        "audio_format": {
            "type": "raw",
            "encoding": "pcm_s16le",
            "sample_rate": sample_rate
        },
        "transcription_config": {
            "language": language,
            "max_delay": MAX_DELAY_SECONDS,
            "enable_partials": true,
            "conversation_config": {
                "end_of_utterance_silence_trigger": END_OF_UTTERANCE_SILENCE_TRIGGER
            }
        }
    })
}

fn extract_payload_text(value: &Value) -> Option<String> {
    if let Some(text) = value
        .get("metadata")
        .and_then(|metadata| metadata.get("transcript"))
        .and_then(|entry| entry.as_str())
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        return Some(text.to_string());
    }

    let Some(results) = value.get("results").and_then(|entry| entry.as_array()) else {
        return None;
    };

    let mut segments = Vec::new();
    for result in results {
        if let Some(text) = result
            .get("content")
            .and_then(|entry| entry.as_str())
            .map(str::trim)
            .filter(|text| !text.is_empty())
        {
            segments.push(text.to_string());
        }
    }

    if segments.is_empty() {
        None
    } else {
        Some(segments.join(" "))
    }
}

async fn flush_last_partial_as_final(
    utterance_buffer: &Arc<AsyncMutex<String>>,
    last_partial: &Arc<AsyncMutex<Option<String>>>,
) -> Option<String> {
    let partial = last_partial.lock().await.take();
    let mut buffer = utterance_buffer.lock().await;
    if let Some(text) = partial {
        append_utterance_segment(&mut buffer, &text);
    }

    let final_text = buffer.trim().to_string();
    buffer.clear();

    if final_text.is_empty() {
        None
    } else {
        Some(final_text)
    }
}

async fn flush_current_utterance(
    callback: &PcmCallback,
    utterance_buffer: &Arc<AsyncMutex<String>>,
    last_partial: &Arc<AsyncMutex<Option<String>>>,
) {
    if let Some(text) = flush_last_partial_as_final(utterance_buffer, last_partial).await {
        emit_commit(callback, "SpeechMatics", text);
    }
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

#[cfg(test)]
mod tests {
    use super::{
        append_utterance_segment, build_start_recognition_payload, extract_payload_text,
        merge_segments,
    };
    use serde_json::json;

    #[test]
    fn start_payload_includes_conversation_config_and_partials() {
        let payload = build_start_recognition_payload("cmn", 16_000);

        assert_eq!(payload["message"], "StartRecognition");
        assert_eq!(payload["audio_format"]["sample_rate"], 16_000);
        assert_eq!(payload["transcription_config"]["language"], "cmn");
        assert_eq!(payload["transcription_config"]["enable_partials"], true);
        assert_eq!(
            payload["transcription_config"]["conversation_config"]["end_of_utterance_silence_trigger"],
            0.5
        );
    }

    #[test]
    fn extract_payload_text_prefers_transcript_metadata() {
        let value = json!({
            "message": "AddTranscript",
            "metadata": {
                "transcript": "hello world"
            },
            "results": [
                {"content": "ignored"}
            ]
        });

        assert_eq!(
            extract_payload_text(&value),
            Some("hello world".to_string())
        );
    }

    #[test]
    fn extract_payload_text_supports_translation_results() {
        let value = json!({
            "message": "AddTranslation",
            "results": [
                {"content": "ni hao"},
                {"content": "world"}
            ]
        });

        assert_eq!(
            extract_payload_text(&value),
            Some("ni hao world".to_string())
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
}
