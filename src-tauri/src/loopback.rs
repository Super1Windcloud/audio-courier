#![allow(clippy::tabs_in_doc_comments)]

use crate::audio_stream::find_model_path;
use crate::utils::{is_dev, write_some_log};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use dasp::sample::ToSample;
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::thread::JoinHandle;
use vosk::{Model, Recognizer};

/** static 全局变量，用于控制录音线程的状态
名称    中文含义	常用于	作用
Relaxed	无序	计数器等简单情况	只保证原子性，不保证顺序
Acquire	获取（读时）	加锁、读取信号	保证之后的操作不能被重排到它前面
Release	释放（写时）	解锁、发信号	保证之前的操作不能被重排到它后面
AcqRel	获取+释放	原子交换等	同时具有 Acquire 和 Release 效果
SeqCst	顺序一致	强制全局顺序	所有线程都按统一顺序观察所有原子操作
 */
static RECORDING: AtomicBool = AtomicBool::new(true);
static CLEAR_RECORDING: AtomicBool = AtomicBool::new(false);

#[derive(Default)]
pub struct RecordParams {
    pub device: String,
    pub file_name: String,
    pub only_pcm: bool,
    pub capture_interval: u32,
    pub pcm_callback: Option<PcmCallback>,
    pub use_drain_chunk_buffer: bool,
    pub use_big_model: bool,
}

pub type PcmCallback = Box<dyn Fn(&str) + Send + Sync + 'static>;

pub fn record_audio_worker(params: RecordParams) -> Result<(), String> {
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

    let config = if device.supports_input() && params.device.contains("input") {
        device.default_input_config()
    } else {
        device.default_output_config()
    }
    .map_err(|_| "Failed to get default input/output config".to_string())?;
    if is_dev() {
        write_some_log(format!("Output device: {}", device.name().unwrap()).as_str());
        write_some_log(format!("Output config: {:#?}", config).as_str());
    };

    let model_path = find_model_path(params.use_big_model).ok_or("未找到Vosk模型文件")?;
    if is_dev() {
        write_some_log(format!("Model path: {}", model_path).as_str());
        if model_path == "未找到Vosk模型文件" {
            std::process::exit(0);
        }
    }

    let model = Model::new(model_path).expect("Could not create the model");
    let mut recognizer = Recognizer::new(&model, config.sample_rate().0 as f32)
        .expect("Could not create the Recognizer");

    recognizer.set_max_alternatives(1);
    recognizer.set_partial_words(false);
    recognizer.set_words(false);
    println!("模型加载成功");

    let recognizer = Arc::new(Mutex::new(recognizer));
    let recognizer_clone = recognizer.clone();

    let spec_f32_stereo = wav_spec_from_config(&config);

    let spec_i16_mono = hound::WavSpec {
        channels: 1,
        sample_rate: spec_f32_stereo.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let path = if params.file_name.trim().is_empty() {
        concat!(env!("CARGO_MANIFEST_DIR"), "/recorded.wav")
    } else {
        params.file_name.as_str()
    };

    let writer = if params.file_name.trim().is_empty() {
        Arc::new(Mutex::new(None))
    } else {
        let path = Path::new(params.file_name.as_str());
        let writer = hound::WavWriter::create(path, spec_i16_mono)
            .map_err(|e| format!("Failed to create WAV writer: {e}"))?;
        Arc::new(Mutex::new(Some(writer)))
    };

    let writer_clone = writer.clone();
    let err_fn = move |err| {
        eprintln!("An error occurred on stream: {err}");
        write_some_log(format!("An error occurred on stream: {err}").as_str())
    };
    let chunk_size = (config.sample_rate().0 * params.capture_interval) as usize;

    let stream = match config.sample_format() {
        cpal::SampleFormat::I16 => device
            .build_input_stream(
                &config.into(),
                move |data, _: &_| {
                    handle_input_data::<i16, i16>(
                        data,
                        &writer,
                        params.pcm_callback.as_ref(),
                        params.only_pcm,
                        &mut recognizer.lock().unwrap(),
                        chunk_size,
                        params.use_drain_chunk_buffer,
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
                    handle_input_data::<f32, f32>(
                        data,
                        &writer,
                        params.pcm_callback.as_ref(),
                        params.only_pcm,
                        &mut recognizer.lock().unwrap(),
                        chunk_size,
                        params.use_drain_chunk_buffer,
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
    println!(
        "final result {:#?}",
        recognizer_clone.lock().unwrap().final_result()
    );

    Ok(())
}

thread_local! {
    static PCM_BUFFER: Mutex<Vec<i16>> = const { Mutex::new(Vec::new()) };
}
use std::sync::LazyLock;
pub static TOTAL_SAMPLES_WRITTEN: LazyLock<Mutex<i32>> = LazyLock::new(|| Mutex::new(0));

fn handle_input_data<T, U>(
    input: &[T],
    writer: &WavWriterHandle,
    pcm_callback: Option<&PcmCallback>,
    only_pcm: bool,
    recognizer: &mut MutexGuard<Recognizer>,
    chunk_size: usize,
    use_drain_chunk_buffer: bool,
) where
    T: Sample + ToSample<i16> + ToSample<f32> + FromSample<i16>,
    U: Sample + hound::Sample + FromSample<T> + FromSample<i16>,
{
    let input = input
        .iter()
        .map(|&x| x.to_sample::<i16>())
        .collect::<Vec<i16>>();
    let input_mono = stereo_to_mono_i16(&input);

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

    PCM_BUFFER.with(|buf_cell| {
        let mut buf = buf_cell.lock().unwrap();
        buf.extend(input_mono);
        if use_drain_chunk_buffer {
            drain_chunk_buffer_to_writer(&mut buf, chunk_size, pcm_callback, only_pcm, recognizer)
        } else {
            use_collected_pcm_to_writer(&mut buf, chunk_size, pcm_callback, only_pcm, recognizer)
        }
    });
}

fn drain_chunk_buffer_to_writer(
    buf: &mut MutexGuard<Vec<i16>>,
    chunk_size: usize,
    pcm_callback: Option<&PcmCallback>,
    only_pcm: bool,
    recognizer: &mut MutexGuard<Recognizer>,
) {
    while buf.len() >= chunk_size {
        let chunk: Vec<i16> = buf.drain(..chunk_size).collect();
        if is_dev() {
            *TOTAL_SAMPLES_WRITTEN.lock().unwrap() += chunk.len() as i32;
            let title = *TOTAL_SAMPLES_WRITTEN.lock().unwrap();
            let used_kb = title as f64 / 1024.0;
            let used_mb = used_kb / 1024.0;

            println!(
                "缓冲区当前使用: {} 个样本, {:.2} KB, {:.2} MB",
                buf.len(),
                used_kb,
                used_mb
            );
        }
        if only_pcm {
            if let Some(callback) = pcm_callback {
                recognizer.accept_waveform(&chunk).unwrap();
                if CLEAR_RECORDING.load(Ordering::SeqCst) {
                    recognizer.reset();
                    *TOTAL_SAMPLES_WRITTEN.lock().unwrap() = 0;
                    CLEAR_RECORDING.store(false, Ordering::SeqCst);
                }
                let partial = recognizer.partial_result().partial;
                if is_dev() {
                    write_some_log(format!("Partial result: {partial}").as_str())
                }
                callback(partial);
            }
        }
    }
}

fn use_collected_pcm_to_writer(
    buf: &mut MutexGuard<Vec<i16>>,
    chunk_size: usize,
    pcm_callback: Option<&PcmCallback>,
    only_pcm: bool,
    recognizer: &mut MutexGuard<Recognizer>,
) {
    if buf.len() >= chunk_size + *TOTAL_SAMPLES_WRITTEN.lock().unwrap() as usize {
        if is_dev() {
            *TOTAL_SAMPLES_WRITTEN.lock().unwrap() = buf.len() as i32;
            let title = *TOTAL_SAMPLES_WRITTEN.lock().unwrap();
            let used_kb = title as f64 / 1024.0;
            let used_mb = used_kb / 1024.0;

            println!(
                "缓冲区当前使用: {} 个样本, {:.2} KB, {:.2} MB",
                buf.len(),
                used_kb,
                used_mb
            );
        }
        if only_pcm {
            if let Some(callback) = pcm_callback {
                recognizer.reset();
                recognizer.accept_waveform(buf).unwrap();
                if CLEAR_RECORDING.load(Ordering::SeqCst) {
                    recognizer.reset();
                    *TOTAL_SAMPLES_WRITTEN.lock().unwrap() = 0;
                    buf.clear();
                    CLEAR_RECORDING.store(false, Ordering::SeqCst);
                }
                let partial = recognizer.partial_result().partial;
                callback(partial);
            }
        }
    }
}
fn stereo_to_mono_i16(samples: &[i16]) -> Vec<i16> {
    if samples.len() < 2 {
        return samples.to_vec();
    }
    samples
        .chunks_exact(2)
        .map(|c| ((c[0] as i32 + c[1] as i32) / 2) as i16)
        .collect()
}

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    if format.is_float() {
        hound::SampleFormat::Float
    } else {
        hound::SampleFormat::Int
    }
}

fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> hound::WavSpec {
    hound::WavSpec {
        channels: config.channels() as _,
        sample_rate: config.sample_rate().0 as _,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: sample_format(config.sample_format()),
    }
}

type WavWriterHandle = Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>;

pub fn stop_recording(handle: JoinHandle<()>) {
    RECORDING.store(false, Ordering::SeqCst);
    println!("停止信号已发送，等待录音线程退出...");
    handle.join().expect("无法 join 录音线程");
    println!("录音线程已退出 ✅");
}

#[tauri::command]
pub fn clear_vosk_accept_buffer() {
    CLEAR_RECORDING.store(true, Ordering::SeqCst);
    println!("清空 Vosk 接受缓存");
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
fn test_record_audio_with_writer() {
    let params = RecordParams {
        device: String::from("default"),
        file_name: "".to_string(),
        only_pcm: true,
        capture_interval: 2,
        pcm_callback: Some(Box::new(|_pcm_data| {})),
        use_drain_chunk_buffer: true,
        use_big_model: true,
    };

    if let Err(e) = start_record_audio_with_writer(params) {
        eprintln!("Error: {e}");
    }
    println!("Enter to stop recording...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
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
