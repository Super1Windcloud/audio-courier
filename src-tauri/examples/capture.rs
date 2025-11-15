#![allow(clippy::collapsible_if)]

use cpal::Sample;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dasp::sample::ToSample;
use hound::WavWriter;
use rubato::{
    ResampleError, Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType,
    WindowFunction,
};
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use tauri_courier_ai_lib::RESAMPLE_RATE;

pub struct RecordParams {
    pub device: String,
    pub duration: u64,
}

fn select_output_config(use_resample: bool) -> Result<cpal::SupportedStreamConfig, String> {
    let device = cpal::default_host()
        .default_output_device()
        .ok_or("没有可用的输出设备")?;

    let supported_configs = device
        .supported_output_configs()
        .map_err(|_| "无法获取输出设备配置".to_string())?;

    let desired_sample_rate = cpal::SampleRate(RESAMPLE_RATE);

    for range in supported_configs {
        if range.min_sample_rate() <= desired_sample_rate
            && range.max_sample_rate() >= desired_sample_rate
        {
            let selected = range.with_sample_rate(desired_sample_rate);
            println!("选择输出设备配置：{:?}", selected);
            return Ok(selected);
        }
    }

    if !use_resample {
        let supported = device.supported_output_configs().unwrap();
        for range in supported {
            if range.sample_format() == cpal::SampleFormat::I16 {
                let rate = range.min_sample_rate(); // 选该范围的最低采样率
                let sel = range.with_sample_rate(rate);
                println!("⚙️ 没有16kHz，选择 i16 配置: {:?}", sel);
                return Ok(sel);
            }
        }
    }

    let fallback = device
        .default_output_config()
        .map_err(|_| "没有可用的输出配置".to_string())?;

    println!("使用默认输出配置：{:?}", fallback);
    Ok(fallback)
}

pub fn record_audio(params: RecordParams) -> Result<(), String> {
    let host = cpal::default_host();

    let device = match params.device.as_str() {
        "default" => host.default_output_device(),
        "default_input" => host.default_input_device(),
        name => host
            .output_devices()
            .unwrap()
            .find(|x| x.name().map(|y| y == name).unwrap_or(false)),
    }
    .ok_or_else(|| "无法找到输出设备".to_string())?;
    let config = select_output_config(true)?;

    const PATH_I16_MONO: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/recorded_i16_mono.wav");

    const PATH_I16_MONO_RESAMPLE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/recorded_i16_mono_resample.wav"
    );

    let spec_f32_stereo = wav_spec_from_config(&config);

    let spec_i16_mono = hound::WavSpec {
        channels: 1,
        sample_rate: spec_f32_stereo.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let spec_i16_mono_resample = hound::WavSpec {
        channels: 1,
        sample_rate: RESAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let i16_mono = Arc::new(Mutex::new(Some(
        hound::WavWriter::create(PATH_I16_MONO, spec_i16_mono)
            .map_err(|e| format!("创建文件失败: {e}"))?,
    )));

    let i16_resample_mono = Arc::new(Mutex::new(Some(
        hound::WavWriter::create(PATH_I16_MONO_RESAMPLE, spec_i16_mono_resample)
            .map_err(|e| format!("创建文件失败: {e}"))?,
    )));

    let err_fn = |err| eprintln!("🎧 音频流错误: {err}");
    let channels = config.channels() as usize;
    let sample_rate = config.sample_rate().0 as usize;

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            {
                let i16_mono = i16_mono.clone();
                let i16_resample_mono = i16_resample_mono.clone();
                let mut audio_buffer = AudioBuffer::new(channels, sample_rate);

                move |data: &[f32], _: &_| {
                    handle_audio_input(
                        data,
                        &mut audio_buffer,
                        &i16_mono,
                        &i16_resample_mono,
                        sample_rate,
                    );
                }
            },
            err_fn,
            None,
        ),
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            {
                let i16_mono = i16_mono.clone();
                let mut audio_buffer = AudioBuffer::new(channels, sample_rate);

                move |data: &[i16], _: &_| {
                    handle_audio_input(
                        data,
                        &mut audio_buffer,
                        &i16_mono,
                        &i16_resample_mono,
                        sample_rate,
                    );
                }
            },
            err_fn,
            None,
        ),
        other => return Err(format!("暂不支持的采样格式: {other:?}")),
    }
    .map_err(|e| format!("创建音频输出流失败: {e}"))?;

    println!("▶️ 开始录制 {:#?} 秒...", params.duration);
    stream.play().unwrap();

    std::thread::sleep(std::time::Duration::from_secs(params.duration));
    drop(stream);

    finalize_all(&[&i16_mono]);

    println!(
        "✅ 录音完成:
  - {PATH_I16_MONO}
  - {PATH_I16_MONO_RESAMPLE} "
    );
    Ok(())
}

fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> hound::WavSpec {
    hound::WavSpec {
        channels: config.channels() as _,
        sample_rate: config.sample_rate().0 as _,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: if config.sample_format().is_float() {
            hound::SampleFormat::Float
        } else {
            hound::SampleFormat::Int
        },
    }
}

type WavWriterHandle = Arc<Mutex<Option<WavWriter<BufWriter<File>>>>>;

#[allow(clippy::too_many_arguments)]
fn handle_audio_input<T>(
    input: &[T],
    buffer: &mut AudioBuffer<T>,
    i16_mono: &WavWriterHandle,
    i16_mono_resample: &Arc<Mutex<Option<WavWriter<BufWriter<File>>>>>,
    sample_rate: usize,
) where
    T: Sample + ToSample<f32> + ToSample<i16>,
{
    buffer.push_samples(input);

    if buffer.is_full() {
        let chunk = buffer.drain_chunk();
        write_all_formats(&chunk, i16_mono, i16_mono_resample, sample_rate);
    }
}

/// 写入所有四种格式（f32/i16 单/双通道）
fn write_all_formats<T>(
    input: &[T],
    i16_mono: &WavWriterHandle,
    i16_mono_resample: &Arc<Mutex<Option<WavWriter<BufWriter<File>>>>>,
    sample_rate: usize,
) where
    T: Sample + ToSample<f32> + ToSample<i16>,
{
    let stereo_f32: Vec<f32> = input.iter().map(|s| s.to_sample::<f32>()).collect();
    let stereo_i16: Vec<i16> = input.iter().map(|s| s.to_sample::<i16>()).collect();
    let mono_f32 = stereo_to_mono_f32(&stereo_f32);
    let mono_i16 = stereo_to_mono_i16(&stereo_i16);

    let output_rate = RESAMPLE_RATE as usize;
    let resampled_mono_i16 = resample_audio_rubato(&mono_f32, sample_rate, output_rate, 1).unwrap();

    write_samples(&mono_i16, i16_mono);
    write_samples(&resampled_mono_i16, i16_mono_resample);
}

fn resample_audio_rubato(
    input: &[f32],
    input_rate: usize,
    output_rate: usize,
    channels: usize,
) -> Result<Vec<i16>, ResampleError> {
    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };
    let mut resampler = SincFixedIn::<f32>::new(
        output_rate as f64 / input_rate as f64,
        2.0,
        params,
        input.len() / channels,
        channels,
    )
    .unwrap();
    let split: Vec<Vec<f32>> = (0..channels)
        .map(|ch| {
            input
                .iter()
                .skip(ch)
                .step_by(channels)
                .cloned()
                .collect::<Vec<f32>>()
        })
        .collect();

    let waves_out = resampler.process(&split, None)?;
    let resampled = waves_out
        .iter()
        .flat_map(|w| w.iter().map(|&s| s.to_sample::<i16>()))
        .collect::<Vec<i16>>();

    Ok(resampled)
}

/// 写入任意类型数据
fn write_samples<T>(data: &[T], writer: &WavWriterHandle)
where
    T: hound::Sample + Copy,
{
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(w) = guard.as_mut() {
            for &s in data {
                w.write_sample(s).ok();
            }
        }
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

fn stereo_to_mono_i16(samples: &[i16]) -> Vec<i16> {
    if samples.len() < 2 {
        return samples.to_vec();
    }
    samples
        .chunks_exact(2)
        .map(|c| ((c[0] as i32 + c[1] as i32) / 2) as i16)
        .collect()
}

fn finalize_all(writers: &[&WavWriterHandle]) {
    for w in writers {
        if let Ok(mut guard) = w.lock() {
            if let Some(w) = guard.take() {
                let _ = w.finalize();
            }
        }
    }
}

/// 用于缓存音频块
struct AudioBuffer<T> {
    data: Vec<T>,
    sample_rate: usize,
    channels: usize,
}

impl<T> AudioBuffer<T>
where
    T: Sample + Clone + ToSample<f32> + ToSample<i16>,
{
    fn new(sample_rate: usize, channels: usize) -> Self {
        Self {
            data: Vec::with_capacity(sample_rate * channels),
            sample_rate,
            channels,
        }
    }

    fn push_samples(&mut self, samples: &[T]) {
        self.data.extend_from_slice(samples);
    }

    fn is_full(&self) -> bool {
        self.data.len() >= self.sample_rate * self.channels
    }

    fn drain_chunk(&mut self) -> Vec<T> {
        self.data
            .drain(..self.sample_rate * self.channels)
            .collect()
    }
}

fn main() {
    let params = RecordParams {
        device: "default".into(),
        duration: 10,
    };

    if let Err(e) = record_audio(params) {
        eprintln!("❌ 错误: {e}");
    }
}
