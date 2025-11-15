#![allow(clippy::collapsible_if)]

use crate::transcript_vendors::{PcmCallback, StreamingTranscriber};
use bytes::{BufMut, Bytes, BytesMut};
use deepgram::{
    Deepgram,
    common::{
        options::{Encoding, Endpointing, Language, Options},
        stream_response::StreamResponse,
    },
};
use futures::channel::mpsc as futures_mpsc;
use futures_util::{SinkExt, StreamExt};
use std::env;
use std::fmt;
use std::thread::{self, JoinHandle};
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, oneshot};

pub struct DeepgramTranscriber {
    sender: mpsc::Sender<Vec<i16>>,
    shutdown: Option<oneshot::Sender<()>>,
    handle: Option<JoinHandle<()>>,
}

impl DeepgramTranscriber {
    pub fn start(sample_rate: u32, callback: PcmCallback) -> Result<Self, String> {
        let api_key = env::var("DEEPGRAM_API_KEY")
            .map_err(|e| format!("Missing DEEPGRAM_API_KEY environment variable: {e}"))?;

        let (sender, receiver) = mpsc::channel::<Vec<i16>>(64);
        let (shutdown, shutdown_rx) = oneshot::channel::<()>();
        let callback_clone = callback.clone();

        let handle = thread::spawn(move || {
            let runtime = Runtime::new().expect("Failed to build Tokio runtime");
            if let Err(err) =
                runtime.block_on(run_stream(api_key, sample_rate, callback_clone, receiver, shutdown_rx))
            {
                eprintln!("Deepgram streaming error: {err}");
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
            .map_err(|e| format!("Failed to queue PCM chunk for Deepgram: {e}"))
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

impl Drop for DeepgramTranscriber {
    fn drop(&mut self) {
        self.stop();
    }
}

impl StreamingTranscriber for DeepgramTranscriber {
    fn queue_chunk(&self, chunk: Vec<i16>) -> Result<(), String> {
        self.enqueue_chunk(chunk)
    }
}

async fn run_stream(
    api_key: String,
    sample_rate: u32,
    callback: PcmCallback,
    mut audio_rx: mpsc::Receiver<Vec<i16>>,
    mut shutdown_rx: oneshot::Receiver<()>,
) -> Result<(), String> {
    let deepgram = Deepgram::new(&api_key)
        .map_err(|e| format!("Failed to construct Deepgram client: {e}"))?;

    let transcription = deepgram.transcription();
    let mut builder = transcription
        .stream_request_with_options(build_stream_options())
        .keep_alive()
        .encoding(Encoding::Linear16)
        .sample_rate(sample_rate)
        .channels(1)
        .interim_results(parse_bool_env("DEEPGRAM_INTERIM_RESULTS").unwrap_or(true))
        .no_delay(parse_bool_env("DEEPGRAM_NO_DELAY").unwrap_or(true))
        .vad_events(parse_bool_env("DEEPGRAM_VAD_EVENTS").unwrap_or(true));

    if let Some(ms) = parse_u32_env("DEEPGRAM_ENDPOINTING_MS") {
        builder = builder.endpointing(Endpointing::CustomDurationMs(ms));
    } else {
        builder = builder.endpointing(Endpointing::CustomDurationMs(300));
    }

    if let Some(ms) = parse_u16_env("DEEPGRAM_UTTERANCE_END_MS") {
        builder = builder.utterance_end_ms(ms);
    } else {
        builder = builder.utterance_end_ms(1000);
    }

    let emit_partials = parse_bool_env("DEEPGRAM_EMIT_PARTIALS").unwrap_or(false);
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
                channel,
                is_final,
                ..
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

    let smart_format = parse_bool_env("DEEPGRAM_SMART_FORMAT").unwrap_or(true);
    builder = builder.smart_format(smart_format);

    let punctuate = parse_bool_env("DEEPGRAM_PUNCTUATE").unwrap_or(true);
    builder = builder.punctuate(punctuate);

    if let Some(value) = env::var("DEEPGRAM_QUERY_PARAMS").ok().filter(|v| !v.trim().is_empty()) {
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
    let cleaned = value
        .trim()
        .replace('-', "_")
        .replace(' ', "_")
        .to_lowercase();

    match cleaned.as_str() {
        "bg" => Language::bg,
        "ca" => Language::ca,
        "cs" => Language::cs,
        "da" => Language::da,
        "de" => Language::de,
        "de_ch" => Language::de_CH,
        "el" => Language::el,
        "en" => Language::en,
        "en_au" => Language::en_AU,
        "en_gb" => Language::en_GB,
        "en_in" => Language::en_IN,
        "en_nz" => Language::en_NZ,
        "en_us" => Language::en_US,
        "es" => Language::es,
        "es_419" | "es419" => Language::es_419,
        "es_latam" | "eslatam" => Language::es_LATAM,
        "et" => Language::et,
        "fi" => Language::fi,
        "fr" => Language::fr,
        "fr_ca" | "frca" => Language::fr_CA,
        "hi" => Language::hi,
        "hi_latn" | "hilatn" => Language::hi_Latn,
        "hu" => Language::hu,
        "id" => Language::id,
        "it" => Language::it,
        "ja" => Language::ja,
        "ko" => Language::ko,
        "ko_kr" | "kokr" => Language::ko_KR,
        "lv" => Language::lv,
        "lt" => Language::lt,
        "ms" => Language::ms,
        "multi" => Language::multi,
        "nl" => Language::nl,
        "nl_be" | "nlbe" => Language::nl_BE,
        "no" => Language::no,
        "pl" => Language::pl,
        "pt" => Language::pt,
        "pt_br" | "ptbr" => Language::pt_BR,
        "ro" => Language::ro,
        "ru" => Language::ru,
        "sk" => Language::sk,
        "sv" => Language::sv,
        "sv_se" | "svse" => Language::sv_SE,
        "ta" => Language::ta,
        "taq" => Language::taq,
        "th" => Language::th,
        "th_th" | "thth" => Language::th_TH,
        "tr" => Language::tr,
        "uk" => Language::uk,
        "vi" => Language::vi,
        "zh" => Language::zh,
        "zh_cn" | "zhcn" => Language::zh_CN,
        "zh_hans" => Language::zh_Hans,
        "zh_hant" => Language::zh_Hant,
        "zh_tw" | "zhtw" => Language::zh_TW,
        _ => Language::Other(value.trim().to_string()),
    }
}

fn parse_bool_env(key: &str) -> Option<bool> {
    let raw = env::var(key).ok()?;
    let normalized = raw.trim().to_lowercase();
    match normalized.as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_u32_env(key: &str) -> Option<u32> {
    env::var(key).ok()?.trim().parse().ok()
}

fn parse_u16_env(key: &str) -> Option<u16> {
    env::var(key).ok()?.trim().parse().ok()
}

#[derive(Debug)]
struct StreamBridgeError;

impl fmt::Display for StreamBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("stream bridge error")
    }
}

impl std::error::Error for StreamBridgeError {}
