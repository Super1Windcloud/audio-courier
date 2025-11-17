#![allow(clippy::tabs_in_doc_comments)]
#![allow(clippy::collapsible_if)]

use crate::RESAMPLE_RATE;
use crate::transcript_vendors::{
    PcmCallback, StatusCallback, StreamingTranscriber, TranscriptVendors,
    assemblyai::AssemblyAiTranscriber, deepgram::DeepgramTranscriber, gladia::GladiaTranscriber,
    revai::RevAiTranscriber, speechmatics::SpeechmaticsTranscriber,
};
use crate::utils::{is_dev, resample_audio_with_rubato, select_output_config, write_some_log};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{ChannelCount, FromSample, Sample};
use dasp::sample::ToSample;
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::thread::JoinHandle;

pub static TOTAL_SAMPLES_WRITTEN: LazyLock<Mutex<i32>> = LazyLock::new(|| Mutex::new(0));

/** static 全局变量，用于控制录音线程的状态
名称    中文含义	常用于	作用
Relaxed	无序	计数器等简单情况	只保证原子性，不保证顺序
Acquire	获取（读时）	加锁、读取信号	保证之后的操作不能被重排到它前面
Release	释放（写时）	解锁、发信号	保证之前的操作不能被重排到它后面
AcqRel	获取+释放	原子交换等	同时具有 Acquire 和 Release 效果
SeqCst	顺序一致	强制全局顺序	所有线程都按统一顺序观察所有原子操作
 */
pub static RECORDING: AtomicBool = AtomicBool::new(true);

thread_local! {
    static PCM_BUFFER: Mutex<Vec<f32>> = const { Mutex::new(Vec::new()) };
}

#[derive(Default)]
pub struct RecordParams {
    pub device: String,
    pub file_name: String,
    pub only_pcm: bool,
    pub capture_interval: u32,
    pub use_resampled: bool,
    pub pcm_callback: Option<PcmCallback>,
    pub auto_chunk_buffer: bool,
    pub selected_asr_vendor: String,
    pub status_callback: Option<StatusCallback>,
}

pub fn record_audio_worker(mut params: RecordParams) -> Result<(), String> {
    let host = cpal::default_host();
    RECORDING.store(true, Ordering::SeqCst);

    let device = match params.device.as_str() {
        "default" => host.default_output_device(),
        "default_input" => host.default_input_device(),
        name => host
            .output_devices()
            .unwrap()
            .find(|x| x.name().map(|y| y == name).unwrap_or(false)),
    }
    .ok_or_else(|| "failed to find input device".to_string())?;

    if is_dev() {
        write_some_log(format!("Input device: {}", device.name().unwrap()).as_str());
    }

    if !params.only_pcm {
        params.use_resampled = false;
        params.auto_chunk_buffer = true;
    }

    let config = if device.supports_input() && params.device.contains("input") {
        device.default_input_config().unwrap()
    } else {
        select_output_config(params.use_resampled)?
    };

    write_some_log(format!("Output selected config: {:#?}", config).as_str());

    let asr_vendor: TranscriptVendors = params.selected_asr_vendor.parse()?;

    let config_sample_rate = config.sample_rate().0;
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: config_sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let path = if params.file_name.trim().is_empty() {
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/transfer_recorded.wav")
    } else {
        params.file_name.as_str()
    };

    let writer = hound::WavWriter::create(path, spec)
        .map_err(|e| format!("Failed to create WAV writer: {e}"))?;
    let writer = Arc::new(Mutex::new(Some(writer)));

    let writer_clone = writer.clone();
    let err_fn = move |err| {
        eprintln!("An error occurred on stream: {err}");
        write_some_log(format!("An error occurred on stream: {err}").as_str())
    };
    let sample_rate_u32 = config_sample_rate;
    let sample_rate = sample_rate_u32 as usize;
    // 100ms best effect
    let chunk_size = (sample_rate_u32 * params.capture_interval / 10) as usize;
    let channels = config.channels();
    let stream_sample_rate = if params.use_resampled {
        RESAMPLE_RATE
    } else {
        sample_rate_u32
    };

    let callback = params.pcm_callback.clone();
    let status_callback = params.status_callback.clone();
    let asr_transcriber: Option<Arc<dyn StreamingTranscriber>> = match (asr_vendor, callback) {
        (TranscriptVendors::AssemblyAI, Some(callback)) => {
            let transcriber =
                AssemblyAiTranscriber::start(stream_sample_rate, callback, status_callback.clone())
                    .map_err(|e| format!("Failed to start AssemblyAI stream: {e}"))?;
            let transcriber: Arc<dyn StreamingTranscriber> = Arc::new(transcriber);
            Some(transcriber)
        }
        (TranscriptVendors::RevAI, Some(callback)) => {
            let transcriber =
                RevAiTranscriber::start(stream_sample_rate, callback, status_callback.clone())
                    .map_err(|e| format!("Failed to start RevAI stream: {e}"))?;
            let transcriber: Arc<dyn StreamingTranscriber> = Arc::new(transcriber);
            Some(transcriber)
        }
        (TranscriptVendors::DeepGram, Some(callback)) => {
            let transcriber =
                DeepgramTranscriber::start(stream_sample_rate, callback, status_callback.clone())
                    .map_err(|e| format!("Failed to start Deepgram stream: {e}"))?;
            let transcriber: Arc<dyn StreamingTranscriber> = Arc::new(transcriber);
            Some(transcriber)
        }
        (TranscriptVendors::SpeechMatics, Some(callback)) => {
            let transcriber = SpeechmaticsTranscriber::start(
                stream_sample_rate,
                callback,
                status_callback.clone(),
            )
            .map_err(|e| format!("Failed to start Speechmatics stream: {e}"))?;
            let transcriber: Arc<dyn StreamingTranscriber> = Arc::new(transcriber);
            Some(transcriber)
        }
        (TranscriptVendors::GlaDia, Some(callback)) => {
            let transcriber =
                GladiaTranscriber::start(stream_sample_rate, callback, status_callback.clone())
                    .map_err(|e| format!("Failed to start Gladia stream: {e}"))?;
            let transcriber: Arc<dyn StreamingTranscriber> = Arc::new(transcriber);
            Some(transcriber)
        }
        _ => None,
    };
    let stream = match config.sample_format() {
        cpal::SampleFormat::I16 => device
            .build_input_stream(
                &config.into(),
                move |data, _: &_| {
                    handle_input_data::<i16, i16>(
                        data,
                        &writer,
                        params.only_pcm,
                        chunk_size,
                        channels,
                        params.auto_chunk_buffer,
                        asr_transcriber.clone(),
                        params.use_resampled,
                        sample_rate,
                    )
                },
                err_fn,
                None,
            )
            .unwrap(),
        cpal::SampleFormat::F32 => device
            .build_input_stream(
                &config.into(),
                move |data, _: &_| {
                    handle_input_data::<f32, i16>(
                        data,
                        &writer,
                        params.only_pcm,
                        chunk_size,
                        channels,
                        params.auto_chunk_buffer,
                        asr_transcriber.clone(),
                        params.use_resampled,
                        sample_rate,
                    )
                },
                err_fn,
                None,
            )
            .unwrap(),
        sample_format => {
            write_some_log(format!("Unsupported sample format '{sample_format}'").as_str());
            return Err(format!("Unsupported sample format '{sample_format}'"));
        }
    };

    stream
        .play()
        .map_err(|e| format!("Failed to play stream: {e}"))?;

    while RECORDING.load(Ordering::SeqCst) {
        thread::sleep(std::time::Duration::from_millis(100));
    }

    drop(stream);

    if !params.only_pcm {
        writer_clone
            .lock()
            .unwrap()
            .take()
            .unwrap()
            .finalize()
            .map_err(|e| format!("Failed to finalize WAV file: {e}"))?;

        write_some_log(format!("Recording complete! Saved to {path}").as_str());
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn handle_input_data<T, U>(
    input: &[T],
    writer: &WavWriterHandle,
    only_pcm: bool,
    chunk_size: usize,
    channels: ChannelCount,
    auto_chunk_buffer: bool,
    transcriber: Option<Arc<dyn StreamingTranscriber>>,
    use_resampled: bool,
    input_sample_rate: usize,
) where
    T: Sample + ToSample<i16> + ToSample<f32> + FromSample<i16> + FromSample<f32>,
    U: Sample + hound::Sample + FromSample<T> + FromSample<i16> + FromSample<f32>,
{
    let input = input
        .iter()
        .map(|&x| x.to_sample::<f32>())
        .collect::<Vec<f32>>();

    let input_mono = if channels == 1 {
        input
    } else {
        stereo_to_mono_f32(&input)
    };

    if !only_pcm {
        if let Ok(mut guard) = writer.try_lock() {
            if let Some(writer) = guard.as_mut() {
                for &sample in input_mono.iter() {
                    let s: U = sample.to_sample();
                    writer.write_sample(s).ok();
                }
            }
        }
        return;
    }
    if auto_chunk_buffer {
        if let Some(transcriber) = transcriber.as_ref() {
            let chunk_i16 =
                prepare_chunk_for_transcriber(&input_mono, use_resampled, input_sample_rate);
            if let Err(err) = transcriber.queue_chunk(chunk_i16) {
                write_some_log(format!("Streaming chunk send failed: {err}").as_str());
            }
        }
    } else {
        PCM_BUFFER.with(|buf_cell| {
            let mut buf = buf_cell.lock().unwrap();
            buf.extend(input_mono);
            drain_chunk_buffer_to_writer(
                &mut buf,
                chunk_size,
                transcriber.clone(),
                use_resampled,
                input_sample_rate,
            )
        });
    }
}

fn drain_chunk_buffer_to_writer(
    buf: &mut MutexGuard<Vec<f32>>,
    chunk_size: usize,
    transcriber: Option<Arc<dyn StreamingTranscriber>>,
    use_resampled: bool,
    input_sample_rate: usize,
) {
    while buf.len() >= chunk_size {
        let chunk = buf.drain(..chunk_size).collect::<Vec<f32>>();

        if is_dev() {
            *TOTAL_SAMPLES_WRITTEN.lock().unwrap() += chunk.len() as i32;
            let title = *TOTAL_SAMPLES_WRITTEN.lock().unwrap();
            let used_kb = title as f64 / 1024.0;
            let used_mb = used_kb / 1024.0;
            let title = title as usize;

            if title.is_multiple_of(input_sample_rate) {
                println!(
                    "缓冲区当前使用: {} 个样本, {:.2} KB, {:.2} MB",
                    title, used_kb, used_mb
                );
            }
        }

        if let Some(transcriber) = transcriber.as_ref() {
            let vendor = transcriber.get_vendor_name();
            let chunk_i16 = prepare_chunk_for_transcriber(&chunk, use_resampled, input_sample_rate);
            if let Err(err) = transcriber.queue_chunk(chunk_i16) {
                write_some_log(format!("{vendor:?} Streaming chunk send failed: {err}").as_str());
            }
        }
    }
}

fn prepare_chunk_for_transcriber(
    input: &[f32],
    use_resampled: bool,
    input_sample_rate: usize,
) -> Vec<i16> {
    if use_resampled {
        match resample_audio_with_rubato(input, input_sample_rate, RESAMPLE_RATE as usize, 1) {
            Ok(resampled) => resampled,
            Err(err) => {
                write_some_log(format!("Resample failed, fallback to raw chunk: {err}").as_str());
                input.iter().map(|&x| x.to_sample::<i16>()).collect()
            }
        }
    } else {
        input.iter().map(|&x| x.to_sample::<i16>()).collect()
    }
}

fn stereo_to_mono_f32(samples: &[f32]) -> Vec<f32> {
    if samples.len() < 2 {
        return samples.to_vec();
    }
    samples
        .chunks_exact(2)
        .map(|c| (c[0] + c[1]) / 2.0)
        .collect()
}

type WavWriterHandle = Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>;

///! 停止录音线程可能死锁卡住,暂时还没解决

pub fn stop_recording(handle: JoinHandle<()>) {
    RECORDING.store(false, Ordering::SeqCst);
    println!("停止信号已发送，等待录音线程退出...");
    handle.join().expect("无法 join 录音线程");
    println!("录音线程已退出 ✅");
}

pub fn start_record_audio_with_writer(params: RecordParams) -> Result<JoinHandle<()>, String> {
    let handle = thread::spawn(move || {
        if let Err(e) = record_audio_worker(params) {
            eprintln!("录音线程出错: {e}");
        }
    });
    Ok(handle)
}

#[test]
fn test_output_path() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/recorded.wav");
    let path = if cfg!(windows) {
        path.replace("/", "\\")
    } else {
        path.to_string()
    };

    println!("Output path: {}", path);
}

#[test]
fn output_format_config() {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let input_device = host.default_input_device().unwrap();
    let config = device.default_output_config().unwrap();
    let input_config = input_device.default_input_config().unwrap();
    println!("Default output config: {config:?}");
    println!("Default input config: {input_config:?}");
}
