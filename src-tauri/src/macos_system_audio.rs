#![cfg(target_os = "macos")]

use crate::RECORDING;
use crate::provider_config::TranscriptRuntimeConfig;
use crate::transcript_vendors::{
    PcmCallback, SelectedDeepgramTranscriber, StatusCallback, StreamingTranscriber,
    TranscriptVendors, assemblyai::AssemblyAiTranscriber, gladia::GladiaTranscriber,
    revai::RevAiTranscriber, speechmatics::SpeechmaticsTranscriber,
};
use crate::utils::write_some_log;
use hex::encode as hex_encode;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};

const MACOS_CAPTURE_SAMPLE_RATE: u32 = 16_000;
const MACOS_CAPTURE_SAMPLE_WIDTH_BYTES: usize = 2;
const AUDIO_DUMP_SWIFT_SOURCE: &str = include_str!("../examples/audio_dump.swift");

static MACOS_CAPTURE_CHILD: OnceLock<Mutex<Option<Child>>> = OnceLock::new();

fn macos_capture_child() -> &'static Mutex<Option<Child>> {
    MACOS_CAPTURE_CHILD.get_or_init(|| Mutex::new(None))
}

pub fn stop_macos_system_audio_capture() {
    if let Some(mut child) = macos_capture_child().lock().unwrap().take() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

pub fn start_macos_system_audio_transcription(
    capture_interval: u32,
    selected_asr_vendor: String,
    pcm_callback: PcmCallback,
    status_callback: Option<StatusCallback>,
    transcript_config: Option<TranscriptRuntimeConfig>,
) -> Result<JoinHandle<()>, String> {
    let helper_binary = ensure_audio_dump_binary()?;
    let transcript_config = transcript_config.unwrap_or_default();
    let transcriber = build_transcriber(
        &selected_asr_vendor,
        pcm_callback,
        status_callback.clone(),
        transcript_config,
    )?;

    RECORDING.store(true, std::sync::atomic::Ordering::SeqCst);

    Ok(thread::spawn(move || {
        if let Err(err) = run_capture_loop(
            &helper_binary,
            capture_interval,
            transcriber,
        ) {
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
    helper_binary: &PathBuf,
    capture_interval: u32,
    transcriber: Arc<dyn StreamingTranscriber>,
) -> Result<(), String> {
    let mut command = Command::new(helper_binary);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|err| format!("Failed to start macOS audio helper: {err}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "macOS audio helper stdout unavailable".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "macOS audio helper stderr unavailable".to_string())?;

    *macos_capture_child().lock().unwrap() = Some(child);

    let stderr_lines = Arc::new(Mutex::new(Vec::<String>::new()));
    let stderr_lines_for_thread = stderr_lines.clone();
    let stderr_handle = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            write_some_log(format!("macOS helper: {line}").as_str());
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
            transcriber
                .queue_chunk(chunk)
                .map_err(|err| format!("macOS helper streaming chunk send failed: {err}"))?;
        }
    }

    if !pcm_samples.is_empty() {
        transcriber
            .queue_chunk(pcm_samples)
            .map_err(|err| format!("macOS helper final chunk send failed: {err}"))?;
    }

    transcriber.shutdown();
    let _ = stderr_handle.join();

    if let Some(mut child) = macos_capture_child().lock().unwrap().take() {
        if RECORDING.load(std::sync::atomic::Ordering::SeqCst) {
            let _ = child.kill();
        }
        let status = child
            .wait()
            .map_err(|err| format!("Failed waiting for macOS audio helper: {err}"))?;
        if !status.success()
            && RECORDING.load(std::sync::atomic::Ordering::SeqCst)
        {
            let stderr_output = stderr_lines.lock().unwrap().join("\n");
            let message = if stderr_output.trim().is_empty() {
                format!("macOS audio helper exited with status {status}")
            } else {
                format!("macOS audio helper exited with status {status}: {stderr_output}")
            };
            return Err(message);
        }
    }

    Ok(())
}

fn ensure_audio_dump_binary() -> Result<PathBuf, String> {
    let mut hasher = Sha256::new();
    hasher.update(AUDIO_DUMP_SWIFT_SOURCE.as_bytes());
    let digest = hex_encode(hasher.finalize());
    let helper_dir = std::env::temp_dir().join("audio-courier-screen-capture");
    let source_path = helper_dir.join(format!("audio_dump_{}.swift", &digest[..12]));
    let binary_path = helper_dir.join(format!("audio_dump_{}", &digest[..12]));

    if binary_path.exists() {
        return Ok(binary_path);
    }

    fs::create_dir_all(&helper_dir)
        .map_err(|err| format!("Failed to create macOS helper directory: {err}"))?;
    fs::write(&source_path, AUDIO_DUMP_SWIFT_SOURCE)
        .map_err(|err| format!("Failed to write macOS helper source: {err}"))?;

    let output = Command::new("xcrun")
        .args([
            "swiftc",
            "-parse-as-library",
            source_path
                .to_str()
                .ok_or_else(|| "Failed to encode macOS helper source path".to_string())?,
            "-o",
            binary_path
                .to_str()
                .ok_or_else(|| "Failed to encode macOS helper binary path".to_string())?,
            "-framework",
            "ScreenCaptureKit",
            "-framework",
            "AVFoundation",
            "-framework",
            "CoreMedia",
        ])
        .output()
        .map_err(|err| format!("Failed to compile macOS audio helper with xcrun: {err}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!(
            "Failed compiling macOS audio helper: {}",
            if stderr.is_empty() {
                "unknown swiftc error".to_string()
            } else {
                stderr
            }
        ));
    }

    Ok(binary_path)
}
