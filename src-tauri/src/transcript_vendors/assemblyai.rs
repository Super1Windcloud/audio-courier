#![allow(clippy::collapsible_if)]

use crate::transcript_vendors::{PcmCallback, StatusCallback, StreamingTranscriber};
use futures_util::{SinkExt, StreamExt, future::try_join};
use serde_json::{Value, json};
use std::env;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use tauri::http::Uri;
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, oneshot, watch};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tungstenite::client::{ClientRequestBuilder, IntoClientRequest};

pub struct AssemblyAiTranscriber {
    sender: mpsc::Sender<Vec<i16>>,
    shutdown: Mutex<Option<oneshot::Sender<()>>>,
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl AssemblyAiTranscriber {
    pub fn start(
        sample_rate: u32,
        callback: PcmCallback,
        status_callback: Option<StatusCallback>,
    ) -> Result<Self, String> {
        let api_key = env::var("ASSEMBLY_API_KEY")
            .map_err(|e| format!("Missing ASSEMBLY_API_KEY environment variable: {e}"))?;

        let (sender, receiver) = mpsc::channel::<Vec<i16>>(64);
        let (shutdown, shutdown_rx) = oneshot::channel::<()>();

        let handle = thread::spawn(move || {
            let runtime = Runtime::new().expect("Failed to build Tokio runtime");
            if let Err(err) = runtime.block_on(run_stream(
                api_key,
                sample_rate,
                callback,
                receiver,
                shutdown_rx,
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
        })
    }

    pub fn enqueue_chunk(&self, chunk: Vec<i16>) -> Result<(), String> {
        self.sender
            .blocking_send(chunk)
            .map_err(|e| format!("Failed to queue PCM chunk for AssemblyAI: {e}"))
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

impl Drop for AssemblyAiTranscriber {
    fn drop(&mut self) {
        self.stop();
    }
}

async fn run_stream(
    api_key: String,
    sample_rate: u32,
    callback: PcmCallback,
    mut audio_rx: mpsc::Receiver<Vec<i16>>,
    mut shutdown_rx: oneshot::Receiver<()>,
) -> Result<(), String> {
    const BASE_URL: &str = "wss://streaming.assemblyai.com/v3/ws";

    let query = format!(
        "sample_rate={sample_rate}&format_turns=true&min_end_of_turn_silence_when_confident=900"
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
                            .map_err(|e| format!("Failed to send audio chunk: {e}"))?;
                    }
                    None => break,
                },
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
            while let Some(message) = stream.next().await {
                let message = message.map_err(|e| {
                    let _ = termination_tx.send(true);
                    format!("AssemblyAI receive error: {e}")
                })?;
                if let Message::Text(payload) = message {
                    if let Ok(value) = serde_json::from_str::<Value>(&payload) {
                        match resolve_event_type(&value) {
                            Some("Termination") => {
                                let _ = termination_tx.send(true);
                                break;
                            }
                            _ => {
                                let transcripts = extract_final_transcripts(&value);
                                for transcript in transcripts {
                                    callback(&transcript);
                                }
                            }
                        }
                    }
                }
            }

            let _ = termination_tx.send(true);
            Ok::<(), String>(())
        }
    };

    try_join(send_audio, receive_events).await?;
    Ok(())
}

fn extract_final_transcripts(value: &Value) -> Vec<String> {
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

fn extract_turn_transcripts(value: &Value) -> Vec<String> {
    if !bool_flag(
        value,
        &["end_of_turn", "turn_is_final", "turn_is_formatted"],
        false,
    ) {
        return Vec::new();
    }

    first_non_empty_text(value, &["utterance", "transcript"])
        .map(|text| vec![text.to_string()])
        .unwrap_or_default()
}

fn extract_plain_transcripts(value: &Value, treat_type_as_final: bool) -> Vec<String> {
    let is_final = if treat_type_as_final {
        true
    } else {
        bool_flag(
            value,
            &[
                "is_final",
                "final",
                "end_of_turn",
                "turn_is_final",
                "turn_is_formatted",
            ],
            false,
        )
    };

    if !is_final {
        return Vec::new();
    }

    first_non_empty_text(value, &["text", "transcript", "utterance"])
        .map(|text| vec![text.to_string()])
        .unwrap_or_default()
}

fn extract_nested_turns(value: &Value) -> Vec<String> {
    let turns = value
        .get("conversation")
        .or_else(|| value.get("turns"))
        .and_then(|turns| turns.as_array());

    let mut transcripts = Vec::new();
    if let Some(turns) = turns {
        for turn in turns {
            if bool_flag(
                turn,
                &[
                    "turn_is_final",
                    "turn_is_formatted",
                    "is_final",
                    "end_of_turn",
                ],
                true,
            ) {
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
                    transcripts.push(text.to_string());
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

impl StreamingTranscriber for AssemblyAiTranscriber {
    fn queue_chunk(&self, chunk: Vec<i16>) -> Result<(), String> {
        self.enqueue_chunk(chunk)
    }

    fn get_vendor_name(&self) -> String {
        "AssemblyAI".to_string()
    }

    fn shutdown(&self) {
        self.stop();
        println!("AssemblyAI websocket shutdown invoked");
    }
}
