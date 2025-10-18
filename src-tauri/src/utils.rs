use cpal::traits::{DeviceTrait, HostTrait};
use cpal::Sample;
use rubato::ResampleError;
use samplerate_rs::{convert, ConverterType};
use std::collections::HashMap;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;

pub fn is_dev() -> bool {
    cfg!(debug_assertions)
}

pub fn write_some_log(msg: &str) {
    #[cfg(target_os = "macos")]
    {
        let mut file = OpenOptions::new()
            .create(true) // 文件不存在则创建
            .append(true) // 追加写入
            .open("app.log") // 日志文件名
            .unwrap();

        writeln!(file, "{}", msg).unwrap(); // 写入一行
    }

    #[cfg(target_os = "windows")]
    {
        let mut file = OpenOptions::new()
            .create(true) // 文件不存在则创建
            .append(true) // 追加写入
            .open("app.log") // 日志文件名
            .unwrap();

        writeln!(file, "{}", msg).unwrap(); // 写入一行
    }
}

pub fn load_env_variables() {
    const ENV_CONTENT: &str = include_str!("../.env");

    let mut vars: HashMap<String, String> = HashMap::new();

    for line in ENV_CONTENT.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = parse_line(line) {
            vars.insert(key, value);
        }
    }

    for (key, value) in vars {
        env::set_var(key, value);
    }
}

fn parse_line(line: &str) -> Option<(String, String)> {
    if let Some(eq_pos) = line.find('=') {
        let key = line[0..eq_pos].trim().to_string();
        let value = line[eq_pos + 1..].trim().to_string();
        Some((key, value))
    } else {
        None
    }
}

pub fn select_output_config() -> Result<cpal::SupportedStreamConfig, String> {
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

#[allow(unused)]
pub fn resample_audio_by_samplerate(
    input: &[f32],
    from_rate: usize,
    target_rate: usize,
    channels: usize,
    chunk_size: usize,
) -> Result<Vec<i16>, ResampleError> {
    if input.len() < chunk_size {
        return Ok(vec![]);
    }
    let resampled = convert(
        from_rate as u32,
        target_rate as u32,
        channels,
        ConverterType::Linear,
        input,
    )
    .unwrap();

    if is_dev() {
        println!(
            "Original len: {}, Resampled len: {}, Expected len: {}",
            input.len(),
            resampled.len(),
            (input.len() as f32 / from_rate as f32 * target_rate as f32) as usize
        );
    }

    let resampled = input.iter().map(|&x| x.to_sample::<i16>()).collect();

    Ok(resampled)
}
