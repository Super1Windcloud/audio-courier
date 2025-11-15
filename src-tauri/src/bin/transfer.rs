#![allow(unused_imports)]

use cpal::traits::{DeviceTrait, HostTrait};
use inquire::Select;
use std::fs::OpenOptions;
use std::io::Write;
use std::iter::Rev;
use std::sync::{Arc, Mutex};
use tauri_courier_ai_lib::{
    RESAMPLE_RATE, RecordParams, get_audio_stream_devices_names, get_record_handle,
    start_record_audio_with_writer, stop_recording,
};

fn main() {
    dotenv::dotenv().ok();
    let device = "default";
    let vendor = Select::new(
        "请选择语音服务提供商:",
        vec!["revai", "assemblyai", "deepgram", "speechmatics", "gladia"],
    )
    .prompt()
    .expect("选择失败");

    let last_result = Arc::new(Mutex::new(String::new()));
    let auto_chunk_buffer = if vendor == "assemblyai" {
        false
    } else if vendor == "revai" {
        false
    } else if vendor == "speechmatics" {
        false
    } else if vendor == "gladia" {
        false
    } else if vendor == "deepgram" {
        true
    } else {
        false
    };
    let capture_interval = if vendor == "assemblyai" { 1 } else { 10 };
    let params = RecordParams {
        device: device.to_string(),
        file_name: "".to_string(),
        only_pcm: true,
        capture_interval,
        pcm_callback: Some(Arc::new(move |chunk: &str| {
            if !chunk.is_empty() && *last_result.lock().unwrap() != chunk {
                *last_result.lock().unwrap() += chunk;
                println!("partial result :{:?}", *last_result.lock().unwrap());
                write_log(chunk);
            }
        })),
        auto_chunk_buffer,
        use_resampled: true,
        selected_asr_vendor: vendor.to_string(),
    };

    if let Ok(handle) = start_record_audio_with_writer(params) {
        let mut guard = get_record_handle().lock().unwrap();
        *guard = Some(handle);
        println!("录音识别已开始 ✅");
    } else {
        eprintln!("录音线程启动失败 ❌");
    }

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    if let Some(handle) = get_record_handle().lock().unwrap().take() {
        stop_recording(handle);
    } else {
        println!("没有正在运行的录音线程");
    }
    println!("录音识别已停止");
}

fn write_log(msg: &str) {
    let mut file = OpenOptions::new()
        .create(true) // 文件不存在则创建
        .append(true) // 追加写入
        .open("app.log") // 日志文件名
        .unwrap();

    writeln!(file, "{}", msg).unwrap(); // 写入一行
}
#[test]
fn output_device_config() {
    fn select_input_config() -> Result<cpal::StreamConfig, String> {
        let names = get_audio_stream_devices_names()?;
        for (i, name) in names.iter().enumerate() {
            println!("{}: {}", i, name);
        }
        let device = cpal::default_host()
            .default_output_device()
            .ok_or("没有可用的输出设备")?;
        let input_device = cpal::default_host()
            .default_input_device()
            .ok_or("没有可用的输入设备")?;

        let supported_configs = device
            .supported_output_configs()
            .map_err(|_| "无法获取输入设备配置".to_string())?;
        {
            println!("默认输出");
            println!("{:?}", device.default_output_config().unwrap());
            println!("默认输入");
            println!("{:?}", input_device.default_input_config().unwrap());
        }
        println!("输出设备支持的配置：");

        let desired_sample_rate = cpal::SampleRate(RESAMPLE_RATE);

        let mut best_config = None;
        for range in supported_configs {
            println!("{:?}", range);
            if range.min_sample_rate() <= desired_sample_rate
                && range.max_sample_rate() >= desired_sample_rate
            {
                best_config = Some(range.with_sample_rate(desired_sample_rate).config());
                break;
            } else if range.sample_format() == cpal::SampleFormat::I16 {
            }
        }

        let support_input = input_device
            .supported_input_configs()
            .map_err(|_| "无法获取输入设备配置".to_string())?;
        println!("输入设备支持的配置：");

        for range in support_input {
            println!("{:?}", range);
        }

        if let Some(config) = best_config {
            println!("选择输出设备配置：{:?}", config);
            Ok(config)
        } else {
            let fallback = device
                .default_output_config()
                .map_err(|_| "没有可用的输入配置妈的".to_string())?;
            Ok(fallback.config())
        }
    }

    select_input_config().unwrap();
}
