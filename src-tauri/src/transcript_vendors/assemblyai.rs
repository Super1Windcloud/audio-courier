#![allow(clippy::collapsible_if)]

use crate::transcript_vendors::{PcmCallback, StreamingTranscriber};
use futures_util::{SinkExt, StreamExt, future::try_join};
use serde_json::{Value, json};
use std::env;
use std::thread::{self, JoinHandle};
use tauri::http::Uri;
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, oneshot, watch};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tungstenite::client::{ClientRequestBuilder, IntoClientRequest};

pub struct AssemblyAiTranscriber {
    sender: mpsc::Sender<Vec<i16>>,
    shutdown: Option<oneshot::Sender<()>>,
    handle: Option<JoinHandle<()>>,
}

impl AssemblyAiTranscriber {
    pub fn start(sample_rate: u32, callback: PcmCallback) -> Result<Self, String> {
        let api_key = env::var("ASSEMBLY_API_KEY")
            .map_err(|e| format!("Missing ASSEMBLY_API_KEY environment variable: {e}"))?;

        let (sender, receiver) = mpsc::channel::<Vec<i16>>(64);
        let (shutdown, shutdown_rx) = oneshot::channel::<()>();
        let callback_clone = callback.clone();

        let handle = thread::spawn(move || {
            let runtime = Runtime::new().expect("Failed to build Tokio runtime");
            if let Err(err) = runtime.block_on(run_stream(
                api_key,
                sample_rate,
                callback_clone,
                receiver,
                shutdown_rx,
            )) {
                eprintln!("AssemblyAI streaming error: {err}");
            }
        });

        Ok(Self {
            sender,
            shutdown: Some(shutdown),
            handle: Some(handle),
        })
    }

    pub fn enqueue_chunk(&self, chunk: Vec<i16>) -> Result<(), String> {
        self.sender
            .blocking_send(chunk)
            .map_err(|e| format!("Failed to queue PCM chunk for AssemblyAI: {e}"))
    }

    pub fn stop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }

        if let Some(handle) = self.handle.take() {
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
    let query = format!("sample_rate={sample_rate}&format_turns=true");
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
                        if let Some(kind) = value.get("type").and_then(|v| v.as_str()) {
                            match kind {
                                "Turn" => {
                                    let transcript = value
                                        .get("transcript")
                                        .and_then(|t| t.as_str())
                                        .map(str::trim);

                                    let turn_is_final = value
                                        .get("turn_is_final")
                                        .and_then(|flag| flag.as_bool())
                                        .or_else(|| {
                                            value
                                                .get("turn_is_formatted")
                                                .and_then(|flag| flag.as_bool())
                                        });

                                    if let (Some(text), Some(true)) = (transcript, turn_is_final) {
                                        if !text.is_empty() {
                                            callback(text);
                                        }
                                    }
                                }
                                "Termination" => {
                                    let _ = termination_tx.send(true);
                                    break;
                                }
                                _ => {}
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

impl StreamingTranscriber for AssemblyAiTranscriber {
    fn queue_chunk(&self, chunk: Vec<i16>) -> Result<(), String> {
        self.enqueue_chunk(chunk)
    }
}
