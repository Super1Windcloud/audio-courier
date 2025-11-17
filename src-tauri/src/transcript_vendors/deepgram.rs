use crate::transcript_vendors::{PcmCallback, StatusCallback, StreamingTranscriber};
use bytes::{BufMut, Bytes, BytesMut};
use deepgram::{
    Deepgram,
    common::{
        options::{Encoding, Language, Options},
        stream_response::StreamResponse,
    },
};
use futures::channel::mpsc as futures_mpsc;
use futures_util::{SinkExt, StreamExt};
use std::env;
use std::fmt;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, oneshot};

pub struct DeepgramTranscriber {
    sender: mpsc::Sender<Vec<i16>>,
    shutdown: Mutex<Option<oneshot::Sender<()>>>,
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl DeepgramTranscriber {
    pub fn start(
        sample_rate: u32,
        callback: PcmCallback,
        status_callback: Option<StatusCallback>,
    ) -> Result<Self, String> {
        let api_key = env::var("DEEPGRAM_API_KEY")
            .map_err(|e| format!("Missing DEEPGRAM_API_KEY environment variable: {e}"))?;

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
                    cb(format!("deepgram: {err}"));
                }
                eprintln!("Deepgram streaming error: {err}");
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
            .map_err(|e| format!("Failed to queue PCM chunk for Deepgram: {e}"))
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

impl Drop for DeepgramTranscriber {
    fn drop(&mut self) {
        self.stop();
    }
}

impl StreamingTranscriber for DeepgramTranscriber {
    fn queue_chunk(&self, chunk: Vec<i16>) -> Result<(), String> {
        self.enqueue_chunk(chunk)
    }

    fn get_vendor_name(&self) -> String {
        "Deepgram".to_string()
    }

    fn shutdown(&self) {
        self.stop();
        println!("Deepgram websocket shutdown invoked");
    }
}

async fn run_stream(
    api_key: String,
    sample_rate: u32,
    callback: PcmCallback,
    mut audio_rx: mpsc::Receiver<Vec<i16>>,
    mut shutdown_rx: oneshot::Receiver<()>,
) -> Result<(), String> {
    let deepgram =
        Deepgram::new(&api_key).map_err(|e| format!("Failed to construct Deepgram client: {e}"))?;

    let transcription = deepgram.transcription();

    let builder = transcription
        .stream_request_with_options(build_stream_options())
        .keep_alive()
        .encoding(Encoding::Linear16)
        .sample_rate(sample_rate)
        .channels(1);

    let emit_partials = true;
    let (mut stream_tx, stream_rx) = futures_mpsc::channel::<Result<Bytes, StreamBridgeError>>(32);

    let bridge = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => {
                    break;
                }
                chunk = audio_rx.recv() => {
                    match chunk {
                        Some(samples) => {
                            if stream_tx.send(Ok(pcm_chunk_to_bytes(&samples))).await.is_err() {
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    });

    let mut responses = builder
        .stream(stream_rx)
        .await
        .map_err(|e| format!("Deepgram websocket failed: {e}"))?;

    while let Some(message) = responses.next().await {
        match message {
            Ok(StreamResponse::TranscriptResponse {
                channel, is_final, ..
            }) => {
                if let Some(entry) = channel.alternatives.first() {
                    let transcript = entry.transcript.trim();
                    if transcript.is_empty() {
                        continue;
                    }

                    if is_final || emit_partials {
                        callback(transcript);
                    }
                }
            }
            Ok(_) => {}
            Err(err) => return Err(format!("Deepgram stream response error: {err}")),
        }
    }

    let _ = bridge.await;
    println!("Deepgram websocket closed");
    Ok(())
}

fn pcm_chunk_to_bytes(chunk: &[i16]) -> Bytes {
    let mut buffer = BytesMut::with_capacity(chunk.len() * 2);
    for sample in chunk {
        buffer.put_i16_le(*sample);
    }
    buffer.freeze()
}

fn build_stream_options() -> Options {
    let mut builder = Options::builder();

    if let Some(language) = language_from_env() {
        builder = builder.language(language);
    }

    builder = builder.smart_format(true);
    builder = builder.dictation(true);

    builder = builder.punctuate(true);

    if let Some(value) = env::var("DEEPGRAM_QUERY_PARAMS")
        .ok()
        .filter(|v| !v.trim().is_empty())
    {
        let params = value
            .split('&')
            .filter_map(|pair| pair.split_once('='))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<Vec<_>>();

        if !params.is_empty() {
            builder = builder.query_params(params);
        }
    }

    builder.build()
}

fn language_from_env() -> Option<Language> {
    let raw = env::var("DEEPGRAM_LANGUAGE").ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(match_language(trimmed))
}

fn match_language(value: &str) -> Language {
    let cleaned = value.trim().replace(['-', ' '], "_").to_lowercase();

    match cleaned.as_str() {
        "en" => Language::en,
        "zh" => Language::zh,
        "zh_cn" | "zhcn" => Language::zh_CN,
        _ => Language::Other(value.trim().to_string()),
    }
}

#[derive(Debug)]
struct StreamBridgeError;

impl fmt::Display for StreamBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("stream bridge error")
    }
}

impl std::error::Error for StreamBridgeError {}
