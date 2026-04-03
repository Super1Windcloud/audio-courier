#![cfg(target_os = "macos")]

use crate::RECORDING;
use crate::provider_config::TranscriptRuntimeConfig;
use crate::transcript_vendors::{
    PcmCallback, SelectedDeepgramTranscriber, StatusCallback, StreamingTranscriber,
    TranscriptVendors, assemblyai::AssemblyAiTranscriber, gladia::GladiaTranscriber,
    revai::RevAiTranscriber, speechmatics::SpeechmaticsTranscriber,
};
use crate::utils::write_some_log;
use macos_audio_capture::{
    CaptureBackend, CaptureSession, selected_backend_name, spawn_system_audio_capture,
};
use std::io::{BufRead, BufReader, Read};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};

const MACOS_CAPTURE_SAMPLE_RATE: u32 = 16_000;
const MACOS_CAPTURE_SAMPLE_WIDTH_BYTES: usize = 2;

static MACOS_CAPTURE_SESSION: OnceLock<Mutex<Option<CaptureSession>>> = OnceLock::new();

fn macos_capture_session() -> &'static Mutex<Option<CaptureSession>> {
    MACOS_CAPTURE_SESSION.get_or_init(|| Mutex::new(None))
}

pub fn stop_macos_system_audio_capture() {
    if let Some(mut session) = macos_capture_session().lock().unwrap().take() {
        session.stop();
        let _ = session.wait();
    }
}

pub fn start_macos_system_audio_transcription(
    capture_interval: u32,
    selected_asr_vendor: String,
    pcm_callback: PcmCallback,
    status_callback: Option<StatusCallback>,
    transcript_config: Option<TranscriptRuntimeConfig>,
) -> Result<JoinHandle<()>, String> {
    let transcript_config = transcript_config.unwrap_or_default();
    let capture_backend = resolve_capture_backend(&transcript_config);
    let backend_name = selected_backend_name(capture_backend).to_string();
    let transcriber = build_transcriber(
        &selected_asr_vendor,
        pcm_callback,
        status_callback.clone(),
        transcript_config,
    )?;

    RECORDING.store(true, std::sync::atomic::Ordering::SeqCst);

    Ok(thread::spawn(move || {
        write_some_log(
            format!(
                "Starting macOS system audio capture with backend: {}",
                backend_name
            )
            .as_str(),
        );
        if let Err(err) = run_capture_loop(capture_interval, capture_backend, transcriber) {
            write_some_log(format!("macOS system audio capture failed: {err}").as_str());
            if let Some(callback) = status_callback.as_ref() {
                callback(err);
            }
        }
        stop_macos_system_audio_capture();
    }))
}

fn build_transcriber(
    selected_asr_vendor: &str,
    pcm_callback: PcmCallback,
    status_callback: Option<StatusCallback>,
    transcript_config: TranscriptRuntimeConfig,
) -> Result<Arc<dyn StreamingTranscriber>, String> {
    let vendor: TranscriptVendors = selected_asr_vendor.parse()?;
    let transcriber: Arc<dyn StreamingTranscriber> = match vendor {
        TranscriptVendors::AssemblyAI => Arc::new(
            AssemblyAiTranscriber::start(
                MACOS_CAPTURE_SAMPLE_RATE,
                pcm_callback,
                status_callback,
                transcript_config,
            )
            .map_err(|err| format!("Failed to start AssemblyAI stream: {err}"))?,
        ),
        TranscriptVendors::RevAI => Arc::new(
            RevAiTranscriber::start(
                MACOS_CAPTURE_SAMPLE_RATE,
                pcm_callback,
                status_callback,
                transcript_config,
            )
            .map_err(|err| format!("Failed to start RevAI stream: {err}"))?,
        ),
        TranscriptVendors::DeepGram => Arc::new(
            SelectedDeepgramTranscriber::start(
                MACOS_CAPTURE_SAMPLE_RATE,
                pcm_callback,
                status_callback,
                transcript_config,
            )
            .map_err(|err| format!("Failed to start Deepgram stream: {err}"))?,
        ),
        TranscriptVendors::SpeechMatics => Arc::new(
            SpeechmaticsTranscriber::start(
                MACOS_CAPTURE_SAMPLE_RATE,
                pcm_callback,
                status_callback,
                transcript_config,
            )
            .map_err(|err| format!("Failed to start Speechmatics stream: {err}"))?,
        ),
        TranscriptVendors::GlaDia => Arc::new(
            GladiaTranscriber::start(
                MACOS_CAPTURE_SAMPLE_RATE,
                pcm_callback,
                status_callback,
                transcript_config,
            )
            .map_err(|err| format!("Failed to start Gladia stream: {err}"))?,
        ),
    };

    Ok(transcriber)
}

fn run_capture_loop(
    capture_interval: u32,
    capture_backend: CaptureBackend,
    transcriber: Arc<dyn StreamingTranscriber>,
) -> Result<(), String> {
    let backend_name = selected_backend_name(capture_backend);
    let mut session = spawn_system_audio_capture(capture_backend)?;
    let stdout = session.take_stdout()?;
    let stderr = session.take_stderr()?;

    *macos_capture_session().lock().unwrap() = Some(session);

    let stderr_lines = Arc::new(Mutex::new(Vec::<String>::new()));
    let stderr_lines_for_thread = stderr_lines.clone();
    let stderr_handle = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            write_some_log(format!("macOS system audio [{backend_name}]: {line}").as_str());
            stderr_lines_for_thread.lock().unwrap().push(line);
        }
    });

    let chunk_samples = ((MACOS_CAPTURE_SAMPLE_RATE * capture_interval.max(1)) / 10) as usize;
    let mut stdout = BufReader::new(stdout);
    let mut read_buffer = [0_u8; 4096];
    let mut pcm_bytes = Vec::<u8>::new();
    let mut pcm_samples = Vec::<i16>::new();

    while RECORDING.load(std::sync::atomic::Ordering::SeqCst) {
        let bytes_read = stdout
            .read(&mut read_buffer)
            .map_err(|err| format!("Failed reading macOS audio helper output: {err}"))?;
        if bytes_read == 0 {
            break;
        }

        pcm_bytes.extend_from_slice(&read_buffer[..bytes_read]);
        let complete_bytes_len =
            pcm_bytes.len() - (pcm_bytes.len() % MACOS_CAPTURE_SAMPLE_WIDTH_BYTES);
        if complete_bytes_len == 0 {
            continue;
        }

        for chunk in pcm_bytes[..complete_bytes_len].chunks_exact(2) {
            pcm_samples.push(i16::from_le_bytes([chunk[0], chunk[1]]));
        }
        pcm_bytes.drain(..complete_bytes_len);

        while pcm_samples.len() >= chunk_samples {
            let chunk = pcm_samples.drain(..chunk_samples).collect::<Vec<_>>();
            transcriber.queue_chunk(chunk).map_err(|err| {
                format!("macOS system audio [{backend_name}] chunk send failed: {err}")
            })?;
        }
    }

    if !pcm_samples.is_empty() {
        transcriber.queue_chunk(pcm_samples).map_err(|err| {
            format!("macOS system audio [{backend_name}] final chunk send failed: {err}")
        })?;
    }

    transcriber.shutdown();
    let _ = stderr_handle.join();

    if let Some(mut session) = macos_capture_session().lock().unwrap().take() {
        if RECORDING.load(std::sync::atomic::Ordering::SeqCst) {
            session.stop();
        }
        if let Err(err) = session.wait() {
            if RECORDING.load(std::sync::atomic::Ordering::SeqCst) {
                let stderr_output = stderr_lines.lock().unwrap().join("\n");
                let message = if stderr_output.trim().is_empty() {
                    err
                } else {
                    format!("{err}: {stderr_output}")
                };
                return Err(message);
            }
        }
    }

    Ok(())
}

fn resolve_capture_backend(transcript_config: &TranscriptRuntimeConfig) -> CaptureBackend {
    match transcript_config
        .macos_system_audio_backend
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some("rust-native") => CaptureBackend::RustNative,
        Some("swift-helper") => CaptureBackend::SwiftHelper,
        _ => CaptureBackend::SwiftHelper,
    }
}
