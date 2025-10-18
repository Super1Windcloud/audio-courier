use crate::loopback::{start_record_audio_with_writer, stop_recording, RecordParams};
use crate::loopback_resample::start_record_audio_with_writer_resampled;
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
    capture_interval: i32,
    use_big_model: bool,
    use_remote_model: bool,
    use_resampled: bool,
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

    // with_channel_communication(device, capture_interval, app);
    let params = RecordParams {
        device: device.to_string(),
        file_name: "".to_string(),
        capture_interval: capture_interval as u32,
        only_pcm: true,
        pcm_callback: Some(Arc::new(move |chunk: &str| {
            app.emit("transcription_result", chunk).unwrap();
        })),
        use_drain_chunk_buffer: true,
        use_big_model,
        use_remote_model,
        xunfei_tx: None,
    };

    if use_resampled {
        if let Ok(handle) = start_record_audio_with_writer_resampled(params) {
            let mut guard = get_record_handle().lock().unwrap();
            *guard = Some(handle);
            println!("Resample 录音识别已开始 ✅");
        } else {
            eprintln!("Resample 录音线程启动失败 ❌");
        }
    } else if let Ok(handle) = start_record_audio_with_writer(params) {
        let mut guard = get_record_handle().lock().unwrap();
        *guard = Some(handle);
        println!("录音识别已开始 ✅");
    } else {
        eprintln!("录音线程启动失败 ❌");
    }
}

#[allow(unused)]
fn with_channel_communication(device: &str, capture_interval: i32, app: AppHandle) {
    use tauri::async_runtime::channel;
    let (tx, mut rx) = channel::<String>(100);

    let params = RecordParams {
        device: device.to_string(),
        file_name: "".to_string(),
        capture_interval: capture_interval as u32,
        only_pcm: true,
        pcm_callback: Some(Arc::new(move |chunk: &str| {
            let _ = tx.blocking_send(chunk.to_string());
        })),
        use_drain_chunk_buffer: true,
        use_big_model: true,
        use_remote_model: false,
        xunfei_tx: None,
    };

    if let Ok(handle) = start_record_audio_with_writer(params) {
        let mut guard = get_record_handle().lock().unwrap();
        *guard = Some(handle);
        println!("录音识别已开始 ✅");
    } else {
        eprintln!("录音线程启动失败 ❌");
    }

    // ---- 主线程异步转发 ----
    tauri::async_runtime::spawn({
        let app = app.clone();
        async move {
            while let Some(text) = rx.recv().await {
                if let Err(e) = app.emit("transcription_result", text) {
                    eprintln!("emit失败: {e}");
                }
            }
        }
    });
}

pub fn find_model_path(big_model: bool, use_remote_model: bool) -> Option<String> {
    let possible_paths = ["vosk-model-cn-0.22", "vosk-model-small-cn-0.22"];

    for path in &possible_paths {
        if !std::path::Path::new(path).exists() {
            eprintln!("找不到模型文件：{}", path);
            return None;
        }
    }
    if big_model && !use_remote_model {
        println!("使用大模型 vosk-model-cn-0.22");
        Some(possible_paths[0].to_string())
    } else {
        Some(possible_paths[1].to_string())
    }
}
