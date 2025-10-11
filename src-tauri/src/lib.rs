mod audio_stream;
mod llm;
mod loopback;
mod loopback_crossbeam;
mod transcript;
mod utils;
pub use audio_stream::*;
use dotenv::{dotenv, from_filename};
use llm::*;
pub use loopback::*;
use std::path::PathBuf;
use tauri::{LogicalSize, Manager};
use utils::*;
#[tauri::command]
fn show_window(window: tauri::Window) -> Result<(), String> {
    if window.is_visible().unwrap() {
        return Ok(());
    }
    let splash = window.get_webview_window("splashscreen");
    if let Some(splash) = splash {
        splash.close().unwrap();
    }
    window.center().unwrap();
    window
        .set_size(LogicalSize::<i32>::from((800, 600)))
        .unwrap();
    window
        .set_focus()
        .map_err(|e| format!("Failed to set focus: {}", e))?;
    window
        .show()
        .map_err(|e| format!("Failed to show window: {}", e))?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if is_dev() {
        let env_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env");
        if from_filename(&env_path).is_err() {
            println!("未找到环境变量文件: {:?}", env_path);
            return;
        }
        println!("已加载环境变量文件: {:?}", env_path);
        dotenv().ok();
    } else {
        load_env_variables();
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            show_window,
            siliconflow_free,
            siliconflow_pro,
            doubao_lite,
            doubao_pro,
            doubao_seed_flash,
            doubao_seed,
            kimi,
            zhipu,
            deepseek_api,
            ali_qwen_2_5,
            ali_qwen_plus_latest,
            ali_qwen_max,
            get_audio_stream_devices_names,
            start_recognize_audio_stream_from_speaker_loopback,
            stop_recognize_audio_stream_from_speaker_loopback,
            clear_vosk_accept_buffer
        ])
        .setup(|_app| {
            let model_paths = ["vosk-model-small-cn-0.22", "vosk-model-cn-0.22"];

            let mut model_found = false;
            for path in &model_paths {
                if std::path::Path::new(path).exists() {
                    println!("找到模型文件: {}", path);
                    model_found = true;
                    break;
                }
            }

            if !model_found {
                println!("警告: 未找到 Vosk 模型文件");
                println!("请确保模型文件位于以下位置之一:");
                for path in &model_paths {
                    println!("  - {}", path);
                }
                std::process::exit(1);
            }

            println!("应用启动完成");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
