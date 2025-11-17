#![allow(clippy::collapsible_if)]

use crate::transcript_vendors::{PcmCallback, StatusCallback, StreamingTranscriber};
use futures_util::{SinkExt, StreamExt, future::try_join};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use tauri::http::Uri;
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, mpsc::error::TrySendError, oneshot, watch};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        client::{ClientRequestBuilder, IntoClientRequest},
        protocol::Message,
    },
};

pub struct GladiaTranscriber {
    sender: mpsc::Sender<Vec<i16>>,
    shutdown: Mutex<Option<oneshot::Sender<()>>>,
    handle: Mutex<Option<JoinHandle<()>>>,
}

const RECEIVE_PARTIAL_TRANSCRIPTS: bool = false;
const RECEIVE_FINAL_TRANSCRIPTS: bool = true;

impl GladiaTranscriber {
    pub fn start(
        sample_rate: u32,
        callback: PcmCallback,
        status_callback: Option<StatusCallback>,
    ) -> Result<Self, String> {
        let api_key = env::var("GLADIA_API_KEY")
            .map_err(|e| format!("Missing GLADIA_API_KEY environment variable: {e}"))?;
        let language = env::var("GLADIA_LANGUAGE").ok();

        let (sender, receiver) = mpsc::channel::<Vec<i16>>(64);
        let (shutdown, shutdown_rx) = oneshot::channel::<()>();

        let handle = thread::spawn(move || {
            let runtime = Runtime::new().expect("Failed to build Tokio runtime");
            if let Err(err) = runtime.block_on(run_stream(
                api_key,
                language,
                sample_rate,
                callback,
                receiver,
                shutdown_rx,
            )) {
                if let Some(cb) = status_callback.as_ref() {
                    cb(format!("gladia: {err}"));
                }
                eprintln!("Gladia streaming error: {err}");
            }
        });

        Ok(Self {
            sender,
            shutdown: Mutex::new(Some(shutdown)),
            handle: Mutex::new(Some(handle)),
        })
    }

    pub fn enqueue_chunk(&self, chunk: Vec<i16>) -> Result<(), String> {
        match self.sender.try_send(chunk) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_chunk)) => Err(
                "Failed to queue PCM chunk for Gladia: channel is full (consumer stalled)".into(),
            ),
            Err(TrySendError::Closed(_chunk)) => {
                Err("Failed to queue PCM chunk for Gladia: channel closed".into())
            }
        }
    }

    pub fn stop(&self) {
        if let Some(shutdown) = self.shutdown.lock().unwrap().take() {
            let _ = shutdown.send(());
        }

        if let Some(handle) = self.handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
}

impl Drop for GladiaTranscriber {
    fn drop(&mut self) {
        self.stop();
    }
}

impl StreamingTranscriber for GladiaTranscriber {
    fn queue_chunk(&self, chunk: Vec<i16>) -> Result<(), String> {
        self.enqueue_chunk(chunk)
    }

    fn get_vendor_name(&self) -> String {
        "Gladia".to_string()
    }

    fn shutdown(&self) {
        self.stop();
        println!("Gladia websocket shutdown invoked");
    }
}

async fn run_stream(
    api_key: String,
    language: Option<String>,
    sample_rate: u32,
    callback: PcmCallback,
    mut audio_rx: mpsc::Receiver<Vec<i16>>,
    mut shutdown_rx: oneshot::Receiver<()>,
) -> Result<(), String> {
    let ws_url = create_live_session(&api_key, sample_rate, language.as_deref()).await?;
    let uri: Uri = ws_url
        .parse()
        .map_err(|e| format!("Failed to parse Gladia websocket URI: {e}"))?;
    let builder = ClientRequestBuilder::new(uri)
        .with_header("x-gladia-key", api_key)
        .with_header("Content-Type", "application/json");
    let request = builder
        .into_client_request()
        .map_err(|e| format!("Failed to build Gladia websocket request: {e}"))?;

    let (ws_stream, _) = connect_async(request)
        .await
        .map_err(|e| format!("Failed to connect to Gladia: {e}"))?;

    let (mut sink, mut stream) = ws_stream.split();
    let (termination_tx, mut termination_rx) = watch::channel(false);

    let send_audio = async move {
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => break,
                result = termination_rx.changed() => {
                    if result.is_err() || *termination_rx.borrow() {
                        break;
                    }
                },
                chunk = audio_rx.recv() => match chunk {
                    Some(samples) => {
                        let audio_bytes = samples
                            .iter()
                            .flat_map(|sample| sample.to_le_bytes())
                            .collect::<Vec<u8>>();
                        sink.send(Message::Binary(audio_bytes.into()))
                            .await
                            .map_err(|e| format!("Failed to send audio chunk to Gladia: {e}"))?;
                    }
                    None => break,
                },
            }
        }

        sink.send(Message::Text(r#"{"type":"stop_recording"}"#.into()))
            .await
            .map_err(|e| format!("Failed to send Gladia stop message: {e}"))?;

        sink.close()
            .await
            .map_err(|e| format!("Failed to close Gladia socket: {e}"))?;

        println!("Gladia websock streaming stop completely!");
        Ok::<(), String>(())
    };

    let receive_events = {
        async move {
            while let Some(message) = stream.next().await {
                let message = match message {
                    Ok(msg) => msg,
                    Err(err) => {
                        let _ = termination_tx.send(true);
                        return Err(format!("Gladia receive error: {err}"));
                    }
                };

                match message {
                    Message::Text(payload) => {
                        if let Some((kind, text)) = parse_transcript(&payload) {
                            let should_emit = match kind {
                                TranscriptKind::Partial => RECEIVE_PARTIAL_TRANSCRIPTS,
                                TranscriptKind::Final => RECEIVE_FINAL_TRANSCRIPTS,
                            };

                            if should_emit && !text.is_empty() {
                                callback(text.as_str());
                            }
                        } else if is_error_payload(&payload) {
                            let _ = termination_tx.send(true);
                            return Err(format!("Gladia returned error payload: {payload}"));
                        }
                    }
                    Message::Close(_) => {
                        let _ = termination_tx.send(true);
                        break;
                    }
                    _ => {}
                }
            }

            let _ = termination_tx.send(true);
            Ok::<(), String>(())
        }
    };

    try_join(send_audio, receive_events).await?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TranscriptKind {
    Partial,
    Final,
}

fn parse_transcript(payload: &str) -> Option<(TranscriptKind, String)> {
    let value: Value = serde_json::from_str(payload).ok()?;
    let event_type = value.get("type").and_then(|v| v.as_str())?;

    match event_type {
        "transcript" => {
            if let Some(data) = value.get("data") {
                return extract_text_from_data(data).map(|text| {
                    let kind = transcript_kind(data);
                    (kind, text)
                });
            }
            parse_legacy_transcript(&value)
        }
        "post_final_transcript" => value
            .get("data")
            .and_then(extract_text_from_data)
            .map(|text| (TranscriptKind::Final, text)),
        _ => None,
    }
}

fn parse_legacy_transcript(value: &Value) -> Option<(TranscriptKind, String)> {
    if value.get("type").and_then(|v| v.as_str())? != "transcript" {
        return None;
    }
    let kind = transcript_kind(value);

    if let Some(text) = value.get("transcript").and_then(|v| v.as_str()) {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some((kind, trimmed.to_string()));
        }
    }

    let transcript = value
        .get("alternatives")
        .and_then(|alts| alts.as_array())
        .and_then(|alts| alts.first())
        .and_then(|first| first.get("transcript"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())?;

    Some((kind, transcript))
}

fn transcript_kind(value: &Value) -> TranscriptKind {
    if let Some(kind) = value.get("type").and_then(|v| v.as_str()) {
        if kind.eq_ignore_ascii_case("final") || kind.contains("final") {
            return TranscriptKind::Final;
        }
        if kind.eq_ignore_ascii_case("partial") {
            return TranscriptKind::Partial;
        }
    }

    if value
        .get("is_final")
        .or_else(|| value.get("final"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        TranscriptKind::Final
    } else {
        TranscriptKind::Partial
    }
}

fn extract_text_from_data(data: &Value) -> Option<String> {
    if let Some(utterance) = data.get("utterance") {
        if let Some(text) = utterance.get("text").and_then(|v| v.as_str()) {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    if let Some(text) = data.get("text").and_then(|v| v.as_str()) {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    if let Some(full_transcript) = data.get("full_transcript").and_then(|v| v.as_str()) {
        let trimmed = full_transcript.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    if let Some(transcription) = data.get("transcription") {
        if let Some(full_transcript) = transcription
            .get("full_transcript")
            .and_then(|v| v.as_str())
        {
            let trimmed = full_transcript.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    if let Some(transcript) = data.get("transcript").and_then(|v| v.as_str()) {
        let trimmed = transcript.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    data.get("alternatives")
        .and_then(|alts| alts.as_array())
        .and_then(|alts| alts.first())
        .and_then(|first| first.get("transcript"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
}

fn is_error_payload(payload: &str) -> bool {
    if let Ok(value) = serde_json::from_str::<Value>(payload) {
        return value.get("type").and_then(|t| t.as_str()) == Some("error");
    }
    false
}

async fn create_live_session(
    api_key: &str,
    sample_rate: u32,
    language: Option<&str>,
) -> Result<String, String> {
    let client = Client::new();
    let model = env::var("GLADIA_MODEL").unwrap_or_else(|_| "solaria-1".to_string());
    let language_config = language
        .filter(|lang| !lang.is_empty())
        .map(|lang| LanguageConfig {
            languages: vec![lang.to_string()],
            code_switching: false,
        });

    let request_body = LiveSessionRequest {
        encoding: "wav/pcm",
        bit_depth: 16,
        sample_rate,
        channels: 1,
        endpointing: 0.4,
        maximum_duration_without_endpointing: 60,
        model,
        language_config,
        messages_config: MessagesConfig {
            receive_partial_transcripts: RECEIVE_PARTIAL_TRANSCRIPTS,
            receive_final_transcripts: RECEIVE_FINAL_TRANSCRIPTS,
        },
    };

    let response = client
        .post("https://api.gladia.io/v2/live")
        .header("x-gladia-key", api_key)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Failed to create Gladia live session: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        return Err(format!(
            "Gladia live session creation failed ({status}): {body}"
        ));
    }

    let parsed: LiveSessionResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Gladia live session response: {e}"))?;

    dbg!(&parsed.url);
    Ok(parsed.url)
}

#[derive(Serialize)]
struct LiveSessionRequest {
    encoding: &'static str,
    bit_depth: u8,
    sample_rate: u32,
    channels: u8,
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    language_config: Option<LanguageConfig>,
    messages_config: MessagesConfig,
    maximum_duration_without_endpointing: u32,
    endpointing: f32,
}

#[derive(Serialize)]
struct LanguageConfig {
    languages: Vec<String>,
    code_switching: bool,
}

#[derive(Serialize)]
struct MessagesConfig {
    receive_partial_transcripts: bool,
    receive_final_transcripts: bool,
}

#[allow(unused)]
#[derive(Deserialize)]
struct LiveSessionResponse {
    created_at: String,
    id: String,
    url: String,
}
