#![allow(clippy::needless_bool)]

use crate::loopback::{RecordParams, start_record_audio_with_writer, stop_recording};
use cpal::traits::{DeviceTrait, HostTrait};
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{AppHandle, Emitter};

pub static RECORD_HANDLE: OnceLock<Mutex<Option<std::thread::JoinHandle<()>>>> = OnceLock::new();

pub fn get_record_handle() -> &'static Mutex<Option<std::thread::JoinHandle<()>>> {
    RECORD_HANDLE.get_or_init(|| Mutex::new(None))
}

#[tauri::command]
pub fn get_audio_stream_devices_names() -> Result<Vec<String>, String> {
    let host = cpal::default_host();
    let mut device_names = Vec::new();

    let default_output_name = host.default_output_device().and_then(|d| d.name().ok());

    if let Ok(output_devices) = host.output_devices() {
        for device in output_devices {
            if let Ok(name) = device.name() {
                if Some(&name) == default_output_name.as_ref() {
                    continue;
                }
                device_names.push(format!("{} [输出]", name));
            }
        }
    }

    if let Ok(input_devices) = host.input_devices() {
        for device in input_devices {
            if let Ok(name) = device.name() {
                device_names.push(format!("{} [输入]", name));
            }
        }
    }

    if let Some(name) = default_output_name {
        device_names.insert(0, format!("{} [输出] (默认)", name));
    }

    Ok(device_names)
}

#[tauri::command]
pub fn stop_recognize_audio_stream_from_speaker_loopback() {
    if let Some(handle) = get_record_handle().lock().unwrap().take() {
        stop_recording(handle);
    } else {
        println!("没有正在运行的录音线程");
    }
}

#[tauri::command]
pub fn start_recognize_audio_stream_from_speaker_loopback(
    app: AppHandle,
    device_name: Option<String>,
    selected_asr_vendor: String,
) {
    let device = if let Some(name) = device_name {
        if name.contains("输入") {
            "default_input"
        } else {
            "default"
        }
    } else {
        "default"
    };

    let capture_interval = if selected_asr_vendor == "assemblyai" {
        1
    } else {
        10
    };

    let last_result = Arc::new(Mutex::new(String::new()));

    let params = RecordParams {
        device: device.to_string(),
        file_name: "".to_string(),
        capture_interval,
        only_pcm: true,
        pcm_callback: Some(Arc::new(move |chunk: &str| {
            if !chunk.is_empty() && *last_result.lock().unwrap() != chunk {
                *last_result.lock().unwrap() = chunk.to_string();
                app.emit("transcription_result", chunk).unwrap();
            }
        })),

        use_resampled: true,
        auto_chunk_buffer: false,
        selected_asr_vendor,
    };

    if let Ok(handle) = start_record_audio_with_writer(params) {
        let mut guard = get_record_handle().lock().unwrap();
        *guard = Some(handle);
        println!("录音识别已开始 ✅");
    } else {
        eprintln!("录音线程启动失败 ❌");
    }
}
