mod audio_stream;
mod llm;
use audio_stream::*;
use dotenv::dotenv;
use llm::*;
use std::sync::Mutex;

#[tauri::command]
fn show_window(window: tauri::Window) -> Result<(), String> {
    if window.is_visible().unwrap() {
        return Ok(());
    }
    window.center().unwrap();
    window.show_menu().unwrap();

    window
        .show()
        .map_err(|e| format!("Failed to show window: {}", e))?;
    window
        .set_focus()
        .map_err(|e| format!("Failed to set focus: {}", e))?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenv().ok();

    tauri::Builder::default()
        .manage(AudioState {
            stream: Mutex::new(None),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            show_window,
            stop_recognize_audio_stream,
            start_recognize_audio_stream,
            siliconflow,
            doubao_lite,
            doubao_pro,
            kimi,
            zhipu,
            deepseek_api,
            ali_qwen_32b,
            ali_qwen_2_5,
            ali_qwen_plus,
            ali_qwen_max,
            doubao_deepseek,
            get_audio_stream_devices_name
        ])
        .setup(|app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
