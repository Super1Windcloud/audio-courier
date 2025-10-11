use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
use dasp::sample::ToSample;
use hound::WavWriter;
use rubato::{FftFixedIn, Resampler};
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};

pub struct RecordParams {
    pub device: String,
    pub duration: u64,
}

fn select_output_config() -> Result<cpal::SupportedStreamConfig, String> {
    let device = cpal::default_host()
        .default_output_device()
        .ok_or("没有可用的输出设备")?;

    let supported_configs = device
        .supported_output_configs()
        .map_err(|_| "无法获取输出设备配置".to_string())?;

    let desired_sample_rate = cpal::SampleRate(16000);

    for range in supported_configs {
        if range.min_sample_rate() <= desired_sample_rate
            && range.max_sample_rate() >= desired_sample_rate
        {
            let selected = range.with_sample_rate(desired_sample_rate);
            println!("选择输出设备配置：{:?}", selected);
            return Ok(selected);
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
    let config = select_output_config().unwrap();

    const PATH_F32_STEREO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/recorded_f32_stereo.wav");
    const PATH_F32_MONO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/recorded_f32_mono.wav");
    const PATH_I16_STEREO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/recorded_i16_stereo.wav");
    const PATH_I16_MONO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/recorded_i16_mono.wav");
    const PATH_I16_STEREO_RESAMPLE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/recorded_i16_stereo_resample.wav"
    );
    const PATH_I16_MONO_RESAMPLE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/recorded_i16_mono_resample.wav"
    );

    // ---- WAV 规格 ----
    let spec_f32_stereo = wav_spec_from_config(&config);
    let spec_f32_mono = hound::WavSpec {
        channels: 1,
        sample_rate: spec_f32_stereo.sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let spec_i16_stereo = hound::WavSpec {
        channels: spec_f32_stereo.channels,
        sample_rate: spec_f32_stereo.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let spec_i16_mono = hound::WavSpec {
        channels: 1,
        sample_rate: spec_f32_stereo.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let spec_i16_stereo_resample = hound::WavSpec {
        channels: spec_i16_stereo.channels,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let spec_i16_mono_resample = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    // ---- 创建写入器 ----
    let f32_stereo = Arc::new(Mutex::new(Some(
        hound::WavWriter::create(PATH_F32_STEREO, spec_f32_stereo)
            .map_err(|e| format!("创建文件失败: {e}"))?,
    )));
    let f32_mono = Arc::new(Mutex::new(Some(
        hound::WavWriter::create(PATH_F32_MONO, spec_f32_mono)
            .map_err(|e| format!("创建文件失败: {e}"))?,
    )));
    let i16_stereo = Arc::new(Mutex::new(Some(
        hound::WavWriter::create(PATH_I16_STEREO, spec_i16_stereo)
            .map_err(|e| format!("创建文件失败: {e}"))?,
    )));
    let i16_mono = Arc::new(Mutex::new(Some(
        hound::WavWriter::create(PATH_I16_MONO, spec_i16_mono)
            .map_err(|e| format!("创建文件失败: {e}"))?,
    )));
    let i16_resample_stereo = Arc::new(Mutex::new(Some(
        hound::WavWriter::create(PATH_I16_STEREO_RESAMPLE, spec_i16_stereo_resample)
            .map_err(|e| format!("创建文件失败: {e}"))?,
    )));
    let i16_resample_mono = Arc::new(Mutex::new(Some(
        hound::WavWriter::create(PATH_I16_MONO_RESAMPLE, spec_i16_mono_resample)
            .map_err(|e| format!("创建文件失败: {e}"))?,
    )));

    let err_fn = |err| eprintln!("🎧 音频流错误: {err}");

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            {
                let f32_stereo = f32_stereo.clone();
                let f32_mono = f32_mono.clone();
                let i16_stereo = i16_stereo.clone();
                let i16_mono = i16_mono.clone();
                let i16_resample_stereo = i16_resample_stereo.clone();
                let i16_resample_mono = i16_resample_mono.clone();

                move |data: &[f32], _: &_| {
                    write_all_formats(
                        data,
                        &f32_stereo,
                        &f32_mono,
                        &i16_stereo,
                        &i16_mono,
                        &i16_resample_stereo,
                        &i16_resample_mono,
                    );
                }
            },
            err_fn,
            None,
        ),
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            {
                let f32_stereo = f32_stereo.clone();
                let f32_mono = f32_mono.clone();
                let i16_stereo = i16_stereo.clone();
                let i16_mono = i16_mono.clone();

                move |data: &[i16], _: &_| {
                    write_all_formats(
                        data,
                        &f32_stereo,
                        &f32_mono,
                        &i16_stereo,
                        &i16_mono,
                        &i16_resample_stereo,
                        &i16_resample_mono,
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

    finalize_all(&[&f32_stereo, &f32_mono, &i16_stereo, &i16_mono]);

    println!(
        "✅ 录音完成:
  - {PATH_F32_STEREO}
  - {PATH_F32_MONO}
  - {PATH_I16_STEREO}
  - {PATH_I16_MONO}"
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

/// 写入所有四种格式（f32/i16 单/双通道）
fn write_all_formats<T>(
    input: &[T],
    f32_stereo: &WavWriterHandle,
    f32_mono: &WavWriterHandle,
    i16_stereo: &WavWriterHandle,
    i16_mono: &WavWriterHandle,
    i16_stereo_resample: &Arc<Mutex<Option<WavWriter<BufWriter<File>>>>>,
    i16_mono_resample: &Arc<Mutex<Option<WavWriter<BufWriter<File>>>>>,
) where
    T: Sample + ToSample<f32> + ToSample<i16>,
{
    // 转换
    let stereo_f32: Vec<f32> = input.iter().map(|s| s.to_sample::<f32>()).collect();
    let stereo_i16: Vec<i16> = input.iter().map(|s| s.to_sample::<i16>()).collect();
    let mono_f32 = stereo_to_mono_f32(&stereo_f32);
    let mono_i16 = stereo_to_mono_i16(&stereo_i16);
    let input_rate = 44100;
    let output_rate = 16000;
    let resampled_stereo_f32 = resample_audio(&stereo_f32, input_rate, output_rate, 2);
    let resampled_mono_f32 = resample_audio(&mono_f32, input_rate, output_rate, 1);

    let resampled_stereo_i16: Vec<i16> = resampled_stereo_f32
        .iter()
        .map(|s| (*s * i16::MAX as f32) as i16)
        .collect();
    let resampled_mono_i16: Vec<i16> = resampled_mono_f32
        .iter()
        .map(|s| (*s * i16::MAX as f32) as i16)
        .collect();

    write_samples(&stereo_f32, f32_stereo);
    write_samples(&mono_f32, f32_mono);
    write_samples(&stereo_i16, i16_stereo);
    write_samples(&mono_i16, i16_mono);
    write_samples(&resampled_stereo_i16, i16_stereo_resample);
    write_samples(&resampled_mono_i16, i16_mono_resample);
}

fn resample_audio(
    input: &[f32],
    input_rate: usize,
    output_rate: usize,
    channels: usize,
) -> Vec<f32> {
    if input_rate == output_rate {
        return input.to_vec();
    }

    // 将 interleaved 格式转换为 per-channel
    let mut separated: Vec<Vec<f32>> = vec![Vec::new(); channels];
    for frame in input.chunks(channels) {
        for (ch, &sample) in frame.iter().enumerate() {
            separated[ch].push(sample);
        }
    }

    let mut resampler = FftFixedIn::<f32>::new(
        input_rate,
        output_rate,
        separated[0].len(),
        separated[0].len(),
        channels,
    )
    .unwrap();

    let output = resampler.process(&separated, None).unwrap();

    // 再次 interleave 成 [L, R, L, R, ...]
    let mut interleaved = Vec::with_capacity(output[0].len() * channels);
    for i in 0..output[0].len() {
        for ch in 0..channels {
            interleaved.push(output[ch][i]);
        }
    }

    interleaved
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

fn main() {
    let params = RecordParams {
        device: "default".into(),
        duration: 10,
    };

    if let Err(e) = record_audio(params) {
        eprintln!("❌ 错误: {e}");
    }
}
