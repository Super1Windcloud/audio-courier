extern crate core;

mod audio_stream;
mod constant;
mod llm;
mod loopback;
mod transcript_vendors;
mod utils;
pub use audio_stream::*;
pub use constant::*;
use dotenv::{dotenv, from_filename};
pub use llm::*;
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
