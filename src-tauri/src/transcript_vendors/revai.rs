#![allow(clippy::collapsible_if)]

///https://docs.rev.ai/api/streaming/requests
///https://docs.rev.ai/api/streaming/responses
use crate::provider_config::{
    TranscriptRuntimeConfig, resolve_optional_string, resolve_required_string,
};
use crate::transcript_vendors::{
    PcmCallback, StatusCallback, StreamingTranscriber, emit_commit, emit_draft,
};
use futures_util::{SinkExt, StreamExt, future::try_join};
#[cfg(target_os = "windows")]
use native_tls::TlsConnector;
use serde_json::Value;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use tauri::http::Uri;
#[cfg(target_os = "windows")]
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::{Mutex as AsyncMutex, mpsc, oneshot, watch};
#[cfg(target_os = "windows")]
use tokio_tungstenite::tungstenite::{
    Error as WsError,
    handshake::client::{Request as WsRequest, Response as WsResponse},
};
#[cfg(target_os = "windows")]
use tokio_tungstenite::{
    Connector, MaybeTlsStream, WebSocketStream, connect_async_tls_with_config,
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        client::{ClientRequestBuilder, IntoClientRequest},
        protocol::Message,
    },
};

pub struct RevAiTranscriber {
    sender: mpsc::Sender<Vec<i16>>,
    shutdown: Mutex<Option<oneshot::Sender<()>>>,
    handle: Mutex<Option<JoinHandle<()>>>,
    stop_requested: Arc<AtomicBool>,
}

impl RevAiTranscriber {
    pub fn start(
        sample_rate: u32,
        callback: PcmCallback,
        status_callback: Option<StatusCallback>,
        transcript_config: TranscriptRuntimeConfig,
    ) -> Result<Self, String> {
        let api_key = resolve_required_string(
            transcript_config.revai_api_key.as_deref(),
            &["REVAI_API_KEY"],
            "REVAI_API_KEY",
        )?;
        let metadata = resolve_optional_string(
            transcript_config.revai_metadata.as_deref(),
            &["REVAI_METADATA"],
        );
        let language = resolve_optional_string(
            transcript_config.revai_language.as_deref(),
            &["REVAI_LANGUAGE"],
        );

        let (sender, receiver) = mpsc::channel::<Vec<i16>>(64);
        let (shutdown, shutdown_rx) = oneshot::channel::<()>();
        let stop_requested = Arc::new(AtomicBool::new(false));
        let stop_requested_for_thread = stop_requested.clone();

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
                stop_requested_for_thread,
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
            stop_requested,
        })
    }

    pub fn enqueue_chunk(&self, chunk: Vec<i16>) -> Result<(), String> {
        self.sender
            .blocking_send(chunk)
            .map_err(|e| format!("Failed to queue PCM chunk for RevAI: {e}"))
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
    stop_requested: Arc<AtomicBool>,
) -> Result<(), String> {
    const BASE_URL: &str = "wss://api.rev.ai/speechtotext/v1/stream";
    const MAX_CONNECTION_WAIT_SECONDS: u32 = 60;

    let content_type =
        format!("audio/x-raw;layout=interleaved;rate={sample_rate};format=S16LE;channels=1");

    let mut params = vec![
        ("access_token".to_string(), api_key),
        ("content_type".to_string(), content_type),
        (
            "max_connection_wait_seconds".to_string(),
            MAX_CONNECTION_WAIT_SECONDS.to_string(),
        ),
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
    let (connected_tx, mut connected_rx) = watch::channel(false);
    let eos_sent = Arc::new(AtomicBool::new(false));
    let last_partial = Arc::new(AsyncMutex::new(None::<String>));
    let saw_final = Arc::new(AtomicBool::new(false));

    let send_audio = async {
        let mut should_send_stop = true;
        let eos_sent = eos_sent.clone();

        loop {
            if *connected_rx.borrow() {
                break;
            }

            tokio::select! {
                _ = &mut shutdown_rx => return Ok::<(), String>(()),
                result = termination_rx.changed() => {
                    if result.is_err() || *termination_rx.borrow() {
                        return Ok::<(), String>(());
                    }
                },
                result = connected_rx.changed() => {
                    if result.is_err() {
                        return Err("RevAI connection readiness watcher closed unexpectedly".into());
                    }
                },
            }
        }

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
            eos_sent.store(true, Ordering::SeqCst);
        }

        loop {
            if *termination_rx.borrow() {
                break;
            }

            if termination_rx.changed().await.is_err() {
                break;
            }
        }

        println!("RevAI websocket streaming stop completed");

        Ok::<(), String>(())
    };

    let receive_events = {
        let callback = callback.clone();
        let termination_tx = termination_tx.clone();
        let connected_tx = connected_tx.clone();
        let eos_sent = eos_sent.clone();
        let last_partial = last_partial.clone();
        let saw_final = saw_final.clone();

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
                        if is_connected_payload(&payload) {
                            let _ = connected_tx.send(true);
                        } else if let Some((kind, result)) = parse_transcript(&payload) {
                            if !result.is_empty() {
                                match kind {
                                    TranscriptKind::Partial => {
                                        *last_partial.lock().await = Some(result.clone());
                                        emit_draft(&callback, "RevAI", &result);
                                    }
                                    TranscriptKind::Final => {
                                        saw_final.store(true, Ordering::SeqCst);
                                        *last_partial.lock().await = None;
                                        emit_commit(&callback, "RevAI", &result);
                                    }
                                }
                            }
                        } else if is_revai_error(&payload) {
                            eprintln!("RevAI error payload: {payload}");
                            let _ = termination_tx.send(true);
                            return Err(format!("RevAI returned error payload: {payload}"));
                        }
                    }
                    Message::Close(frame) => {
                        let closed_normally = frame
                            .as_ref()
                            .map(|close| {
                                close.code
                                    == tungstenite::protocol::frame::coding::CloseCode::Normal
                            })
                            .unwrap_or(false);

                        if let Some(frame) = frame.as_ref() {
                            eprintln!(
                                "RevAI closed websocket: code={:?}, reason={}",
                                frame.code, frame.reason
                            );
                        } else {
                            eprintln!("RevAI closed websocket without close frame data");
                        }
                        let _ = termination_tx.send(true);

                        if (closed_normally && eos_sent.load(Ordering::SeqCst))
                            || stop_requested.load(Ordering::SeqCst)
                        {
                            flush_last_partial_as_final(&callback, &last_partial, &saw_final).await;
                            break;
                        }
                        return Err("RevAI websocket closed unexpectedly".into());
                    }
                    _ => {}
                }
            }

            if eos_sent.load(Ordering::SeqCst) || stop_requested.load(Ordering::SeqCst) {
                flush_last_partial_as_final(&callback, &last_partial, &saw_final).await;
                let _ = termination_tx.send(true);
                return Ok::<(), String>(());
            }

            if !stop_requested.load(Ordering::SeqCst) {
                let _ = termination_tx.send(true);
                return Err("RevAI websocket closed unexpectedly".into());
            }

            let _ = termination_tx.send(true);
            Ok::<(), String>(())
        }
    };

    try_join(send_audio, receive_events).await?;
    Ok(())
}

async fn flush_last_partial_as_final(
    callback: &PcmCallback,
    last_partial: &Arc<AsyncMutex<Option<String>>>,
    saw_final: &Arc<AtomicBool>,
) {
    if saw_final.load(Ordering::SeqCst) {
        return;
    }

    if let Some(text) = last_partial.lock().await.take() {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            emit_commit(callback, "RevAI", trimmed);
        }
    }
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

fn is_connected_payload(payload: &str) -> bool {
    if let Ok(value) = serde_json::from_str::<Value>(payload) {
        if let Some(kind) = value.get("type").and_then(|v| v.as_str()) {
            return kind.eq_ignore_ascii_case("connected");
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

#[cfg(test)]
mod tests {
    use super::{
        TranscriptKind, collect_elements_text, extract_final_text, extract_partial_text,
        is_connected_payload, parse_transcript,
    };
    use serde_json::json;

    #[test]
    fn parse_final_transcript_joins_text_and_punctuation() {
        let payload = r#"{
            "type":"final",
            "ts":1.01,
            "end_ts":3.2,
            "elements":[
                {"type":"text","value":"One","ts":1.04,"end_ts":1.55,"confidence":1.0},
                {"type":"punct","value":" "},
                {"type":"text","value":"two","ts":1.84,"end_ts":2.15,"confidence":1.0},
                {"type":"punct","value":"."}
            ]
        }"#;

        let parsed = parse_transcript(payload);
        assert_eq!(
            parsed,
            Some((TranscriptKind::Final, "One two.".to_string()))
        );
    }

    #[test]
    fn parse_partial_transcript_joins_elements() {
        let payload = r#"{
            "type":"partial",
            "ts":1.01,
            "end_ts":2.2,
            "elements":[
                {"type":"text","value":"one"},
                {"type":"text","value":" tooth"}
            ]
        }"#;

        let parsed = parse_transcript(payload);
        assert_eq!(
            parsed,
            Some((TranscriptKind::Partial, "one tooth".to_string()))
        );
    }

    #[test]
    fn parse_connected_payload_is_not_treated_as_transcript() {
        assert!(is_connected_payload(
            r#"{"type":"connected","id":"s1d24ax2fd21"}"#
        ));
        assert_eq!(
            parse_transcript(r#"{"type":"connected","id":"s1d24ax2fd21"}"#),
            None
        );
    }

    #[test]
    fn final_text_extractor_ignores_empty_elements() {
        let value = json!({
            "elements": [
                {"type": "punct", "value": " "},
                {"type": "text", "value": "Hello"},
                {"type": "punct", "value": "!"}
            ]
        });

        assert_eq!(extract_final_text(&value), Some("Hello!".to_string()));
    }

    #[test]
    fn partial_text_extractor_falls_back_to_top_level_fields() {
        let value = json!({
            "type": "partial",
            "transcript": "fallback text"
        });

        assert_eq!(
            extract_partial_text(&value),
            Some("fallback text".to_string())
        );
    }

    #[test]
    fn collect_elements_text_trims_outer_whitespace_only() {
        let elements = vec![
            json!({"type": "punct", "value": " "}),
            json!({"type": "text", "value": "Hello"}),
            json!({"type": "punct", "value": ", world"}),
            json!({"type": "punct", "value": " "}),
        ];

        assert_eq!(
            collect_elements_text(&elements),
            Some("Hello, world".to_string())
        );
    }
}
