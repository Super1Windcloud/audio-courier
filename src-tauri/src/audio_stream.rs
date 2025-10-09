use crate::loopback::{start_record_audio_with_writer, stop_recording, RecordParams};
use crate::transcript::TranscriptionManager;
use cpal::traits::{DeviceTrait, HostTrait};
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn get_audio_stream_devices_names() -> Result<Vec<String>, String> {
    let host = cpal::default_host();
    let mut device_names = Vec::new();

    let output_devices = host
        .output_devices()
        .map_err(|e| format!("获取输出设备列表失败: {}", e))?;

    for device in output_devices {
        if let Ok(name) = device.name() {
            device_names.push(format!("{} [输出]", name));
        }
    }

    let input_devices = host
        .input_devices()
        .map_err(|e| format!("获取输入设备列表失败: {}", e))?;

    for device in input_devices {
        if let Ok(name) = device.name() {
            device_names.push(format!("{} [输入]", name));
        }
    }

    if let Some(default_device) = host.default_output_device() {
        if let Ok(default_name) = default_device.name() {
            let default_display_name = format!("{} [输出] (默认)", default_name);
            device_names.retain(|name| !name.starts_with(&format!("{} [输入]", default_name)));
            device_names.insert(0, default_display_name);
        }
    }

    Ok(device_names)
}

#[tauri::command]
pub fn stop_recognize_audio_stream_from_speaker_loopback() {
    stop_recording();
}

#[tauri::command]
pub fn start_recognize_audio_stream_from_speaker_loopback(
    app: AppHandle,
    device_name: Option<String>,
) -> Result<String, String> {
    let model_path = find_model_path().ok_or("未找到 Vosk 模型文件")?;
    println!("使用模型文件: {}", model_path);

    let mut manager = TranscriptionManager::new_vosk(model_path.clone());
    manager
        .initialize()
        .map_err(|e| format!("初始化转录模型失败: {:?}", e))?;
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
        eprintln!("Error: {}", e);
        Err(format!("Error: {e}"))
    } else {
        Ok(format!("音频流识别已启动，使用设备: {}", ""))
    }
}

fn find_model_path() -> Option<String> {
    let possible_paths = [
        "vosk-model-small-cn-0.22",
        "../vosk-model-small-cn-0.22",
        "../../vosk-model-small-cn-0.22",
        "./vosk-model-small-cn-0.22",
    ];

    for path in &possible_paths {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    None
}
