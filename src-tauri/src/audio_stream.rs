use crate::loopback::{start_record_audio_with_writer, stop_recording, RecordParams};
use crate::utils::write_some_log;
use cpal::traits::{DeviceTrait, HostTrait};
use std::sync::{Mutex, OnceLock};
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
    capture_interval: i32,
) {
    let device = if device_name.is_some() {
        let device = device_name.unwrap();
        if device.contains("输入") {
            "default_input"
        } else {
            "default"
        }
    } else {
        "default"
    };

    let params = RecordParams {
        device: device.to_string(),
        file_name: "".to_string(),
        capture_interval: capture_interval as u32,
        only_pcm: true,
        pcm_callback: Some(Box::new({
            let app = app.clone();
            move |chunk: &str| {
                let app = app.clone();
                let chunk = chunk.to_string();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = app.emit("transcription_result", chunk) {
                        eprintln!("emit失败: {e}");
                    }
                });
            }
        })),
        use_drain_chunk_buffer: true,
        use_big_model: true,
    };

    if let Ok(handle) = start_record_audio_with_writer(params) {
        let mut guard = get_record_handle().lock().unwrap();
        *guard = Some(handle);
        println!("录音识别已开始 ✅");
    } else {
        eprintln!("录音线程启动失败 ❌");
    }
}

pub fn find_model_path(big_model: bool) -> Option<String> {
    let possible_paths = ["vosk-model-cn-0.22", "vosk-model-small-cn-0.22"];

    for path in &possible_paths {
        if !std::path::Path::new(path).exists() {
            eprintln!("找不到模型文件：{}", path);
            write_some_log(format!("找不到模型文件：{}", path).as_str());
            return None;
        }
    }
    if big_model {
        Some(possible_paths[0].to_string())
    } else {
        Some(possible_paths[1].to_string())
    }
}
