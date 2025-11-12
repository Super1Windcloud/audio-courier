use crate::transcript_vendors::PcmCallback;
use futures_util::{SinkExt, StreamExt, future::try_join};
use serde_json::{Value, json};
use std::env;
use std::thread::{self, JoinHandle};
use tauri::http::Uri;
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tungstenite::client::{ClientRequestBuilder, IntoClientRequest};

#[derive(Clone)]
pub enum AssemblyAudioChunk {
    Int16(Vec<i16>),
    Float32(Vec<f32>),
}

pub struct AssemblyAiTranscriber {
    sender: mpsc::Sender<AssemblyAudioChunk>,
    shutdown: Option<oneshot::Sender<()>>,
    handle: Option<JoinHandle<()>>,
}

impl AssemblyAiTranscriber {
    pub fn start(sample_rate: u32, callback: PcmCallback) -> Result<Self, String> {
        let api_key = env::var("ASSEMBLY_API_KEY")
            .map_err(|e| format!("Missing ASSEMBLY_API_KEY environment variable: {e}"))?;

        let (sender, receiver) = mpsc::channel::<AssemblyAudioChunk>(64);
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

    pub fn send_chunk(&self, chunk: AssemblyAudioChunk) -> Result<(), String> {
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
    mut audio_rx: mpsc::Receiver<AssemblyAudioChunk>,
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

    let send_audio = async {
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => break,
                chunk = audio_rx.recv() => match chunk {
                    Some(chunk) => {
                        let audio_bytes = match chunk {
                            AssemblyAudioChunk::Int16(samples) => samples
                                .iter()
                                .flat_map(|sample| sample.to_le_bytes())
                                .collect::<Vec<u8>>(),
                            AssemblyAudioChunk::Float32(samples) => samples
                                .iter()
                                .flat_map(|sample| sample.to_le_bytes())
                                .collect::<Vec<u8>>(),
                        };

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

    let receive_events = async {
        let callback = callback.clone();

        while let Some(message) = stream.next().await {
            let message = message.map_err(|e| format!("AssemblyAI receive error: {e}"))?;
            if let Message::Text(payload) = message {
                if let Ok(value) = serde_json::from_str::<Value>(&payload) {
                    if let Some(kind) = value.get("type").and_then(|v| v.as_str()) {
                        match kind {
                            "Turn" => {
                                if let Some(text) = value.get("transcript").and_then(|t| t.as_str())
                                {
                                    if !text.trim().is_empty() {
                                        callback(text);
                                    }
                                }
                            }
                            "Termination" => break,
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok::<(), String>(())
    };

    try_join(send_audio, receive_events).await?;
    Ok(())
}
