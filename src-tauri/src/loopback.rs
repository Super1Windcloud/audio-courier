use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
static RECORDING: AtomicBool = AtomicBool::new(true);

#[derive(Default)]
pub struct RecordParams {
    pub device: String,
    pub file_name: String,
    pub only_pcm: bool,
    pub pcm_callback: Option<Box<dyn Fn(Vec<f32>) + Send + Sync>>,
}

pub fn start_record_audio_with_writer(params: RecordParams) -> Result<(), String> {
    let host = cpal::default_host();

    let device = match params.device.as_str() {
        "default" => host.default_input_device(),
        "default_input" => host.default_input_device(),
        name => host
            .input_devices()
            .unwrap()
            .find(|x| x.name().map(|y| y == name).unwrap_or(false)),
    }
    .ok_or_else(|| "failed to find input device".to_string())?;

    println!("Input device: {}", device.name().unwrap());

    let config = if device.supports_input() {
        device.default_input_config()
    } else {
        device.default_output_config()
    }
    .map_err(|_| "Failed to get default input/output config".to_string())?;
    println!("Default input/output config: {config:?}");

    //
    let path = if params.file_name.trim().is_empty() {
        concat!(env!("CARGO_MANIFEST_DIR"), "/recorded.wav")
    } else {
        params.file_name.as_str()
    };
    let spec = wav_spec_from_config(&config);

    let writer = if params.file_name.trim().is_empty() {
        // empty file name,
        Arc::new(Mutex::new(None))
    } else {
        let path = Path::new(params.file_name.as_str());
        let writer = hound::WavWriter::create(path, spec)
            .map_err(|e| format!("Failed to create WAV writer: {e}"))?;
        Arc::new(Mutex::new(Some(writer)))
    };

    let writer_clone = writer.clone();
    let err_fn = move |err| {
        eprintln!("An error occurred on stream: {err}");
    };

    // 创建音频输入流
    let stream = match config.sample_format() {
        cpal::SampleFormat::I8 => device
            .build_input_stream(
                &config.into(),
                move |data, _: &_| {
                    handle_input_data::<i8, i8>(data, &writer, params.pcm_callback.as_ref())
                },
                err_fn,
                None,
            )
            .unwrap(),
        cpal::SampleFormat::I16 => device
            .build_input_stream(
                &config.into(),
                move |data, _: &_| {
                    handle_input_data::<i16, i16>(data, &writer, params.pcm_callback.as_ref())
                },
                err_fn,
                None,
            )
            .unwrap(),
        cpal::SampleFormat::I32 => device
            .build_input_stream(
                &config.into(),
                move |data, _: &_| {
                    handle_input_data::<i32, i32>(data, &writer, params.pcm_callback.as_ref())
                },
                err_fn,
                None,
            )
            .unwrap(),
        cpal::SampleFormat::F32 => device
            .build_input_stream(
                &config.into(),
                move |data, _: &_| {
                    handle_input_data::<f32, f32>(data, &writer, params.pcm_callback.as_ref())
                },
                err_fn,
                None,
            )
            .unwrap(),
        sample_format => return Err(format!("Unsupported sample format '{sample_format}'")),
    };

    stream
        .play()
        .map_err(|e| format!("Failed to play stream: {e}"))?;

    // 监听停止条件：例如外部通过信号来停止录音
    while RECORDING.load(Ordering::SeqCst) {
        thread::sleep(std::time::Duration::from_millis(100)); // 每100ms检查一次是否停止录音
    }

    // 录音结束，停止流
    drop(stream);

    // 如果是写文件，finalize WAV 文件
    if !params.only_pcm {
        writer_clone
            .lock()
            .unwrap()
            .take()
            .unwrap()
            .finalize()
            .map_err(|e| format!("Failed to finalize WAV file: {e}"))?;
        println!("Recording complete! Saved to {path}");
    }

    Ok(())
}

// 处理 PCM 数据，支持回调
fn handle_input_data<T, U>(
    input: &[T],
    writer: &WavWriterHandle,
    pcm_callback: Option<&Box<dyn Fn(Vec<f32>) + Send + Sync>>,
) where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T>,
{
    if let Some(callback) = pcm_callback {
        // let input_f32 = input
        //     .iter()
        //     .map(|&x| x.to_float_sample())
        //     .collect::<Vec<f32>>();
        //
        // callback(input_f32);
    } else if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                let sample: U = U::from_sample(sample);
                writer.write_sample(sample).ok();
            }
        }
    }
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

// 停止录音的函数
pub fn stop_recording() {
    RECORDING.store(false, Ordering::SeqCst); // 设置停止录音标志
}

#[test]
fn test_record_audio_with_writer() {
    let params = RecordParams {
        device: String::from("default"),
        file_name: "".to_string(),
        only_pcm: true, // 设置为 true，处理 PCM 数据而不写入文件
        pcm_callback: Some(Box::new(|pcm_data| {
            println!("Received PCM data: {:?}", pcm_data);
            // 这里你可以进一步处理 PCM 数据，例如保存到数据库、分析数据等
        })),
    };

    if let Err(e) = start_record_audio_with_writer(params) {
        eprintln!("Error: {e}");
    }

    println!("Enter to stop recording...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    stop_recording();
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
