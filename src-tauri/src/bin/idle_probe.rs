use inquire::Select;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tauri_courier_ai_lib::{
    PcmCallback, SelectedDeepgramTranscriber, StatusCallback, StreamingTranscriber,
    TranscriptEvent, TranscriptRuntimeConfig, assemblyai::AssemblyAiTranscriber,
    gladia::GladiaTranscriber, revai::RevAiTranscriber, speechmatics::SpeechmaticsTranscriber,
    transcript_runtime_config_from_env,
};

const PROBE_DURATION: Duration = Duration::from_secs(180);
const SAMPLE_RATE: u32 = 16_000;

fn main() {
    load_env();

    let vendor = Select::new(
        "请选择要测试的供应商:",
        vec!["assemblyai", "deepgram", "gladia", "revai", "speechmatics"],
    )
    .prompt()
    .expect("选择失败");

    let config = transcript_runtime_config_from_env();
    let (status_tx, status_rx) = mpsc::channel::<String>();
    let (event_tx, event_rx) = mpsc::channel::<TranscriptEvent>();

    let pcm_callback: PcmCallback = Arc::new(move |event| {
        let _ = event_tx.send(event);
    });
    let status_callback: StatusCallback = Arc::new(move |message| {
        let _ = status_tx.send(message);
    });

    let transcriber = match start_transcriber(vendor, config, pcm_callback, status_callback) {
        Ok(transcriber) => transcriber,
        Err(err) => {
            eprintln!("启动失败: {err}");
            std::process::exit(1);
        }
    };

    println!("idle probe started vendor={vendor} duration={}s", PROBE_DURATION.as_secs());
    println!("No audio chunks will be sent.");

    let started_at = Instant::now();
    let mut disconnect_reason: Option<String> = None;
    let mut event_count = 0_u32;

    loop {
        let elapsed = started_at.elapsed();
        if elapsed >= PROBE_DURATION {
            break;
        }

        let wait_for = PROBE_DURATION.saturating_sub(elapsed).min(Duration::from_secs(1));

        match status_rx.recv_timeout(wait_for) {
            Ok(message) => {
                disconnect_reason = Some(message);
                break;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                disconnect_reason = Some("status callback channel disconnected".to_string());
                break;
            }
        }

        while let Ok(event) = event_rx.try_recv() {
            event_count += 1;
            println!(
                "unexpected transcript event kind={:?} len={} text={:?}",
                event.kind,
                event.text.len(),
                event.text
            );
        }
    }

    transcriber.shutdown();

    let elapsed = started_at.elapsed();
    println!("vendor={vendor}");
    println!("elapsed_seconds={:.3}", elapsed.as_secs_f64());
    println!("probe_duration_seconds={}", PROBE_DURATION.as_secs());
    println!("transcript_event_count={event_count}");

    match disconnect_reason {
        Some(reason) => {
            println!("result=disconnected_before_timeout");
            println!("disconnect_reason={reason}");
        }
        None => {
            println!("result=alive_for_full_probe_window");
            println!("disconnect_reason=<none>");
        }
    }
}

fn load_env() {
    let env_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env");
    let _ = dotenv::from_filename(&env_path);
    let _ = dotenv::dotenv();
}

fn start_transcriber(
    vendor: &str,
    config: TranscriptRuntimeConfig,
    callback: PcmCallback,
    status_callback: StatusCallback,
) -> Result<Box<dyn StreamingTranscriber>, String> {
    match vendor {
        "assemblyai" => AssemblyAiTranscriber::start(
            SAMPLE_RATE,
            callback,
            Some(status_callback),
            config,
        )
        .map(|transcriber| Box::new(transcriber) as Box<dyn StreamingTranscriber>),
        "deepgram" => SelectedDeepgramTranscriber::start(
            SAMPLE_RATE,
            callback,
            Some(status_callback),
            config,
        )
        .map(|transcriber| Box::new(transcriber) as Box<dyn StreamingTranscriber>),
        "gladia" => GladiaTranscriber::start(
            SAMPLE_RATE,
            callback,
            Some(status_callback),
            config,
        )
        .map(|transcriber| Box::new(transcriber) as Box<dyn StreamingTranscriber>),
        "revai" => RevAiTranscriber::start(
            SAMPLE_RATE,
            callback,
            Some(status_callback),
            config,
        )
        .map(|transcriber| Box::new(transcriber) as Box<dyn StreamingTranscriber>),
        "speechmatics" => SpeechmaticsTranscriber::start(
            SAMPLE_RATE,
            callback,
            Some(status_callback),
            config,
        )
        .map(|transcriber| Box::new(transcriber) as Box<dyn StreamingTranscriber>),
        _ => Err(format!("unsupported vendor: {vendor}")),
    }
}
