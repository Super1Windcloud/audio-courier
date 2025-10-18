#![allow(clippy::tabs_in_doc_comments)]

use crate::audio_stream::find_model_path;
use crate::utils::{is_dev, resample_audio_by_samplerate, select_output_config, write_some_log};
use crate::{
    PcmCallback, RTASRClient, RecordParams, ACCESS_KEY_ID, ACCESS_KEY_SECRET, APP_ID,
    CLEAR_RECORDING, RECORDING, RESAMPLE_RATE, TARGET_SAMPLE_RATE,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{ChannelCount, FromSample, Sample};
use dasp::sample::ToSample;
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::thread::JoinHandle;
use vosk::{Model, Recognizer};

thread_local! {
    static PCM_BUFFER: Mutex<Vec<f32>> = const { Mutex::new(Vec::new()) };
}

use std::sync::LazyLock;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

static TOTAL_SAMPLES_WRITTEN: LazyLock<Mutex<i32>> = LazyLock::new(|| Mutex::new(0));

pub fn record_audio_worker_resampled(mut params: RecordParams) -> Result<(), String> {
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
        device.default_input_config().unwrap()
    } else {
        select_output_config()?
        // device.default_output_config()
    };
    if is_dev() {
        write_some_log(format!("Output device: {}", device.name().unwrap()).as_str());
        write_some_log(format!("Output config: {:#?}", config).as_str());
    };

    let model_path = find_model_path(params.use_big_model, params.use_remote_model)
        .ok_or("未找到Vosk模型文件")?;
    write_some_log(format!("Model path: {}", model_path).as_str());
    if model_path == "未找到Vosk模型文件" {
        std::process::exit(0);
    }
    let use_remote_model = params.use_remote_model;
    if use_remote_model {
        //TODO 开启Websocket连接,接受通道的Sender的Buffer ,传入pcm_callback
        let (tx, rx) = mpsc::channel::<Vec<i16>>(128);
        params.xunfei_tx = Some(tx);

        let pcm_callback_clone = params.pcm_callback.clone(); // 先 clone 防止move whole struct

        let client = RTASRClient::new(APP_ID, ACCESS_KEY_ID, ACCESS_KEY_SECRET);

        thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                client.run_ws_loop(rx, pcm_callback_clone).await;
            });
        });
    }

    let model = Model::new(model_path).expect("Could not create the model");
    let mut recognizer =
        Recognizer::new(&model, RESAMPLE_RATE as f32).expect("Could not create the Recognizer");

    recognizer.set_max_alternatives(1);
    recognizer.set_partial_words(false);
    recognizer.set_words(false);
    println!("模型加载成功");

    let recognizer = Arc::new(Mutex::new(recognizer));
    let recognizer_clone = recognizer.clone();

    let spec_i16_mono = hound::WavSpec {
        channels: 1,
        sample_rate: RESAMPLE_RATE,
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
    let chunk_size = (RESAMPLE_RATE * params.capture_interval) as usize;
    let channels = config.channels();
    let stream = match config.sample_format() {
        cpal::SampleFormat::I16 => device
            .build_input_stream(
                &config.into(),
                move |data, _: &_| {
                    handle_input_data::<i16, i16>(
                        data,
                        &writer,
                        params.pcm_callback.clone(),
                        params.only_pcm,
                        &mut recognizer.lock().unwrap(),
                        chunk_size,
                        params.use_drain_chunk_buffer,
                        channels,
                        RESAMPLE_RATE,
                        params.xunfei_tx.clone(),
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
                        params.pcm_callback.clone(),
                        params.only_pcm,
                        &mut recognizer.lock().unwrap(),
                        chunk_size,
                        params.use_drain_chunk_buffer,
                        channels,
                        RESAMPLE_RATE,
                        params.xunfei_tx.clone(),
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

#[allow(clippy::too_many_arguments)]
fn handle_input_data<T, U>(
    input: &[T],
    writer: &WavWriterHandle,
    pcm_callback: Option<PcmCallback>,
    only_pcm: bool,
    recognizer: &mut MutexGuard<Recognizer>,
    chunk_size: usize,
    use_drain_chunk_buffer: bool,
    channels: ChannelCount,
    sample_rate: u32,
    xunfei_tx: Option<Sender<Vec<i16>>>,
) where
    T: Sample + ToSample<i16> + ToSample<f32> + FromSample<i16> + FromSample<f32>,
    U: Sample + hound::Sample + FromSample<T> + FromSample<i16>,
{
    if !only_pcm {
        let input = input
            .iter()
            .map(|&x| x.to_sample::<i16>())
            .collect::<Vec<i16>>();

        let input_mono = if channels == 1 {
            input
        } else {
            stereo_to_mono_i16(&input)
        };
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
        let input = input
            .iter()
            .map(|&x| x.to_sample::<f32>())
            .collect::<Vec<f32>>();

        let input_mono = if channels == 1 {
            input
        } else {
            stereo_to_mono_f32(&input)
        };
        let mut buf = buf_cell.lock().unwrap();
        buf.extend(input_mono);
        if use_drain_chunk_buffer {
            drain_chunk_buffer_to_writer(
                &mut buf,
                chunk_size,
                pcm_callback,
                only_pcm,
                recognizer,
                sample_rate,
                xunfei_tx,
            )
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn drain_chunk_buffer_to_writer(
    buf: &mut MutexGuard<Vec<f32>>,
    chunk_size: usize,
    pcm_callback: Option<PcmCallback>,
    only_pcm: bool,
    recognizer: &mut MutexGuard<Recognizer>,
    _sample_rate: u32,
    xunfei_tx: Option<Sender<Vec<i16>>>,
) {
    while buf.len() >= chunk_size {
        let chunk = buf.drain(..chunk_size).collect::<Vec<f32>>();

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

        let chunk = resample_audio_by_samplerate(
            &chunk,
            _sample_rate as usize,
            TARGET_SAMPLE_RATE.parse().unwrap(),
            1,
            chunk_size,
        )
        .unwrap();

        if only_pcm {
            if let Some(ref callback) = pcm_callback {
                if let Some(tx) = xunfei_tx.as_ref() {
                    let _ = tx.try_send(chunk); // 非阻塞发送
                } else {
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

pub fn start_record_audio_with_writer_resampled(
    params: RecordParams,
) -> Result<JoinHandle<()>, String> {
    let handle = thread::spawn(move || {
        if let Err(e) = record_audio_worker_resampled(params) {
            eprintln!("录音线程出错: {e}");
        }
    });
    Ok(handle)
}
