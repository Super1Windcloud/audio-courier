use crate::RESAMPLE_RATE;
use cpal::Sample;
use cpal::traits::{DeviceTrait, HostTrait};
use rubato::ResampleError;
use std::collections::HashMap;
use std::env;
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::Write;
use std::path::{Path, PathBuf};

pub const APP_LOG_FILE_NAME: &str = "Audio Courier.log";
const LEGACY_APP_LOG_FILE_NAME: &str = "app.log";
const APP_IDENTIFIER: &str = "com.superwindcloud.audio-courier";
const APP_PRODUCT_NAME: &str = "Audio Courier";

pub fn is_dev() -> bool {
    cfg!(debug_assertions)
}

pub fn reset_app_log_files() {
    truncate_log_file(&primary_plain_log_path());

    for path in legacy_log_paths() {
        remove_log_file_if_exists(&path);
    }
}

pub fn write_some_log(msg: &str) {
    let path = primary_plain_log_path();
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            let _ = create_dir_all(parent);
        }
    }

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(file, "{}", msg);
    } else {
        // 如果文件打开失败，至少尝试在标准错误中输出，以便开发者查看
        eprintln!("Failed to write log to {:?}: {}", path, msg);
    }
}

fn truncate_log_file(path: &Path) {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            let _ = create_dir_all(parent);
        }
    }

    let _ = File::create(path);
}

fn primary_plain_log_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(local_app_data) = env::var_os("LOCALAPPDATA") {
            return PathBuf::from(local_app_data)
                .join(APP_IDENTIFIER)
                .join("logs")
                .join(APP_LOG_FILE_NAME);
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Logs")
                .join(APP_IDENTIFIER)
                .join(APP_LOG_FILE_NAME);
        }
    }

    PathBuf::from(APP_LOG_FILE_NAME)
}

fn legacy_log_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "windows")]
    {
        if let Some(local_app_data) = env::var_os("LOCALAPPDATA") {
            let local_app_data = PathBuf::from(local_app_data);
            paths.push(
                local_app_data
                    .join(APP_PRODUCT_NAME)
                    .join(APP_LOG_FILE_NAME),
            );
            paths.push(
                local_app_data
                    .join(APP_PRODUCT_NAME)
                    .join(LEGACY_APP_LOG_FILE_NAME),
            );
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = env::var_os("HOME") {
            let home = PathBuf::from(home);
            paths.push(
                home.join("Library")
                    .join("Application Support")
                    .join(APP_IDENTIFIER)
                    .join(APP_LOG_FILE_NAME),
            );
        }
    }

    paths.push(PathBuf::from(LEGACY_APP_LOG_FILE_NAME));
    paths
}

fn remove_log_file_if_exists(path: &Path) {
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
}

pub fn load_env_variables() {
    const ENV_CONTENT: &str = include_str!("../.env.local");

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
        unsafe {
            env::set_var(key, value);
        }
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

pub fn select_output_config(use_resample: bool) -> Result<cpal::SupportedStreamConfig, String> {
    let device = cpal::default_host()
        .default_output_device()
        .ok_or("没有可用的输出设备")?;

    let supported_configs = device
        .supported_output_configs()
        .map_err(|_| "无法获取输出设备配置".to_string())?;

    let desired_sample_rate = RESAMPLE_RATE;

    for range in supported_configs {
        if range.min_sample_rate() <= desired_sample_rate
            && range.max_sample_rate() >= desired_sample_rate
            && range.sample_format() == cpal::SampleFormat::I16
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

#[allow(unused)]
pub fn resample_audio_with_rubato(
    input: &[f32],
    input_rate: usize,
    output_rate: usize,
    channels: usize,
) -> Result<Vec<i16>, ResampleError> {
    if input.is_empty() || input_rate == 0 || output_rate == 0 || channels == 0 {
        return Ok(Vec::new());
    }

    if input_rate == output_rate {
        return Ok(input.iter().map(|&s| s.to_sample::<i16>()).collect());
    }

    let input_frames = input.len() / channels;
    if input_frames == 0 {
        return Ok(Vec::new());
    }

    let output_frames =
        ((input_frames as u128 * output_rate as u128) / input_rate as u128).max(1) as usize;
    let ratio = input_rate as f64 / output_rate as f64;
    let mut resampled = Vec::with_capacity(output_frames * channels);

    for frame_idx in 0..output_frames {
        let src_pos = frame_idx as f64 * ratio;
        let src_idx = src_pos.floor() as usize;
        let next_idx = (src_idx + 1).min(input_frames.saturating_sub(1));
        let frac = (src_pos - src_idx as f64) as f32;

        for ch in 0..channels {
            let base = src_idx * channels + ch;
            let next = next_idx * channels + ch;
            let sample = input[base] * (1.0 - frac) + input[next] * frac;
            resampled.push(sample.to_sample::<i16>());
        }
    }

    Ok(resampled)
}
