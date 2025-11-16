#![allow(clippy::collapsible_if)]

use crate::transcript_vendors::{PcmCallback, StatusCallback, StreamingTranscriber};
use futures_util::{SinkExt, StreamExt, future::try_join};
#[cfg(target_os = "windows")]
use native_tls::TlsConnector;
use serde_json::Value;
use std::env;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use tauri::http::Uri;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, oneshot, watch};
#[cfg(target_os = "windows")]
use tokio_tungstenite::{Connector, connect_async_tls_with_config};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{
        Error as WsError,
        client::{ClientRequestBuilder, IntoClientRequest},
        handshake::client::{Request as WsRequest, Response as WsResponse},
        protocol::Message,
    },
};

pub struct RevAiTranscriber {
    sender: mpsc::Sender<Vec<i16>>,
    shutdown: Mutex<Option<oneshot::Sender<()>>>,
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl RevAiTranscriber {
    pub fn start(
        sample_rate: u32,
        callback: PcmCallback,
        status_callback: Option<StatusCallback>,
    ) -> Result<Self, String> {
        let api_key = env::var("REVAI_API_KEY")
            .map_err(|e| format!("Missing REVAI_API_KEY environment variable: {e}"))?;
        let metadata = env::var("REVAI_METADATA").ok();
        let language = env::var("REVAI_LANGUAGE").ok();

        let (sender, receiver) = mpsc::channel::<Vec<i16>>(64);
        let (shutdown, shutdown_rx) = oneshot::channel::<()>();

        let handle = thread::spawn(move || {
            let runtime = Runtime::new().expect("Failed to build Tokio runtime");
            if let Err(err) = runtime.block_on(run_stream(
                api_key,
                metadata,
                language,
                sample_rate,
                callback,
                receiver,
                shutdown_rx,
            )) {
                if let Some(cb) = status_callback.as_ref() {
                    cb(format!("revai: {err}"));
                }
                eprintln!("RevAI streaming error: {err}");
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
            .map_err(|e| format!("Failed to queue PCM chunk for RevAI: {e}"))
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

impl Drop for RevAiTranscriber {
    fn drop(&mut self) {
        self.stop();
    }
}

impl StreamingTranscriber for RevAiTranscriber {
    fn queue_chunk(&self, chunk: Vec<i16>) -> Result<(), String> {
        self.enqueue_chunk(chunk)
    }

    fn get_vendor_name(&self) -> String {
        "RevAI".to_string()
    }

    fn shutdown(&self) {
        self.stop();
        println!("RevAI websocket shutdown invoked");
    }
}

async fn run_stream(
    api_key: String,
    metadata: Option<String>,
    language: Option<String>,
    sample_rate: u32,
    callback: PcmCallback,
    mut audio_rx: mpsc::Receiver<Vec<i16>>,
    mut shutdown_rx: oneshot::Receiver<()>,
) -> Result<(), String> {
    const BASE_URL: &str = "wss://api.rev.ai/speechtotext/v1/stream";

    let content_type = format!(
        "audio/x-raw;layout=interleaved;rate={sample_rate};format=S16LE;channels=1;max_connection_wait_seconds=540"
    );

    let mut params = vec![
        ("access_token".to_string(), api_key),
        ("content_type".to_string(), content_type),
        ("sample_rate".to_string(), sample_rate.to_string()),
    ];

    if let Some(language) = language.as_ref() {
        if !language.trim().is_empty() {
            params.push(("language".to_string(), language.clone()));
        }
    }

    if let Some(metadata) = metadata.as_ref() {
        if !metadata.trim().is_empty() {
            params.push(("metadata".to_string(), metadata.clone()));
        }
    }

    let query = params
        .into_iter()
        .map(|(key, value)| format!("{key}={}", encode_component(&value)))
        .collect::<Vec<_>>()
        .join("&");
    let url = format!("{BASE_URL}?{query}");

    let uri: Uri = url
        .parse()
        .map_err(|e| format!("Failed to parse RevAI streaming URI: {e}"))?;
    dbg!(&uri);
    let builder = ClientRequestBuilder::new(uri);
    let client_request = builder
        .into_client_request()
        .map_err(|e| format!("Failed to build RevAI websocket request: {e}"))?;

    #[cfg(target_os = "windows")]
    let (ws_stream, _) = connect_revai_socket(client_request)
        .await
        .map_err(|e| format!("Failed to connect to RevAI: {e}"))?;
    #[cfg(not(target_os = "windows"))]
    let (ws_stream, _) = connect_async(client_request)
        .await
        .map_err(|e| format!("Failed to connect to RevAI: {e}"))?;

    let (mut sink, mut stream) = ws_stream.split();
    let (termination_tx, mut termination_rx) = watch::channel(false);

    let send_audio = async {
        let mut should_send_stop = true;

        loop {
            tokio::select! {
                _ = &mut shutdown_rx => break,
                result = termination_rx.changed() => {
                    if result.is_err() || *termination_rx.borrow() {
                        eprintln!("RevAI signaled termination; halting audio upload");
                        should_send_stop = false;
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
                            .map_err(|e| format!("Failed to send audio chunk to RevAI: {e}"))?;
                    }
                    None => break,
                },
            }
        }

        if should_send_stop {
            sink.send(Message::Text("EOS".into()))
                .await
                .map_err(|e| format!("Failed to send RevAI stop message: {e}"))?;
        }

        sink.close()
            .await
            .map_err(|e| format!("Failed to close RevAI socket: {e}"))?;

        println!("Revai websocket streaming stop completely!");

        Ok::<(), String>(())
    };

    let receive_events = {
        let callback = callback.clone();
        let termination_tx = termination_tx.clone();

        async move {
            while let Some(message) = stream.next().await {
                let message = match message {
                    Ok(msg) => msg,
                    Err(err) => {
                        let _ = termination_tx.send(true);
                        return Err(format!("RevAI receive error: {err}"));
                    }
                };

                match message {
                    Message::Text(payload) => {
                        if let Some((kind, result)) = parse_transcript(&payload) {
                            if kind == TranscriptKind::Partial && !result.is_empty() {
                                callback(result.as_str());
                            }
                        } else if is_revai_error(&payload) {
                            eprintln!("RevAI error payload: {payload}");
                            let _ = termination_tx.send(true);
                            return Err(format!("RevAI returned error payload: {payload}"));
                        }
                    }
                    Message::Close(frame) => {
                        if let Some(frame) = frame {
                            eprintln!(
                                "RevAI closed websocket: code={:?}, reason={}",
                                frame.code, frame.reason
                            );
                        } else {
                            eprintln!("RevAI closed websocket without close frame data");
                        }
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

#[cfg(target_os = "windows")]
async fn connect_revai_socket(
    request: WsRequest,
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, WsResponse), WsError> {
    match TlsConnector::builder().build() {
        Ok(connector) => {
            match connect_async_tls_with_config(
                request.clone(),
                None,
                false,
                Some(Connector::NativeTls(connector)),
            )
            .await
            {
                Ok(res) => return Ok(res),
                Err(err) => {
                    eprintln!("RevAI native TLS connect failed, retrying with rustls: {err}");
                }
            }
        }
        Err(err) => eprintln!("Failed to build native TLS connector: {err}"),
    }

    connect_async(request).await
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TranscriptKind {
    Partial,
    Final,
}

fn parse_transcript(payload: &str) -> Option<(TranscriptKind, String)> {
    let value: Value = serde_json::from_str(payload).ok()?;
    let kind = value.get("type").and_then(|v| v.as_str())?;
    let kind = match kind {
        "partial" => TranscriptKind::Partial,
        "final" => TranscriptKind::Final,
        _ => return None,
    };

    let transcript = match kind {
        TranscriptKind::Partial => extract_partial_text(&value),
        TranscriptKind::Final => extract_final_text(&value),
    }?;

    Some((kind, transcript))
}

fn extract_final_text(value: &Value) -> Option<String> {
    value
        .get("elements")
        .and_then(|elements| elements.as_array())
        .and_then(|elements| collect_elements_text(elements))
}

fn extract_partial_text(value: &Value) -> Option<String> {
    if let Some(text) = value
        .get("elements")
        .and_then(|elements| elements.as_array())
        .and_then(|elements| collect_elements_text(elements))
    {
        if !text.is_empty() {
            return Some(text);
        }
    }

    for field in ["value", "text", "transcript"] {
        if let Some(text) = value
            .get(field)
            .and_then(|entry| entry.as_str())
            .map(str::trim)
            .filter(|text| !text.is_empty())
        {
            return Some(text.to_string());
        }
    }

    None
}

fn collect_elements_text(elements: &[Value]) -> Option<String> {
    let mut buffer = String::new();
    for element in elements {
        let text = element
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        buffer.push_str(text);
    }

    let trimmed = buffer.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn is_revai_error(payload: &str) -> bool {
    if let Ok(value) = serde_json::from_str::<Value>(payload) {
        if let Some(kind) = value.get("type").and_then(|v| v.as_str()) {
            return kind.eq_ignore_ascii_case("error");
        }
    }
    false
}

fn encode_component(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}
