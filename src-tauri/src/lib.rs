extern crate core;

mod audio_stream;
mod constant;
pub mod license;
mod llm;
mod loopback;
mod transcript_vendors;
mod utils;
pub use audio_stream::*;
use chrono::{DateTime, Utc};
pub use constant::*;
use dotenv::{dotenv, from_filename};
use license::{
    build_activation_request, ensure_signer_access, load_license_status, persist_license,
    sign_license_from_request_json, signer_status,
};
pub use llm::*;
pub use loopback::*;
use std::path::PathBuf;
use tauri::{LogicalSize, Manager, WebviewUrl, WebviewWindowBuilder};
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
        .set_size(LogicalSize::<i32>::from((800, 900)))
        .unwrap();
    window
        .set_focus()
        .map_err(|e| format!("Failed to set focus: {}", e))?;
    window
        .show()
        .map_err(|e| format!("Failed to show window: {}", e))?;

    Ok(())
}

#[tauri::command]
fn get_activation_request(user_id: Option<String>) -> Result<license::ActivationRequest, String> {
    build_activation_request(user_id)
}

#[tauri::command]
fn get_license_status(app: tauri::AppHandle) -> Result<license::LicenseStatus, String> {
    load_license_status(&app)
}

#[tauri::command]
fn import_license(
    app: tauri::AppHandle,
    raw_license: String,
) -> Result<license::LicenseStatus, String> {
    persist_license(&app, &raw_license)
}

#[tauri::command]
fn get_signer_status() -> license::SignerStatus {
    signer_status()
}

#[tauri::command]
fn sign_activation_license(
    raw_request: String,
    user_id: String,
    expires_at: String,
    max_version: String,
    features: Vec<String>,
) -> Result<license::SignedLicense, String> {
    let expires_at = expires_at
        .parse::<DateTime<Utc>>()
        .map_err(|err| format!("expiresAt 解析失败: {err}"))?;
    sign_license_from_request_json(&raw_request, user_id, expires_at, max_version, features)
}

#[tauri::command]
fn open_license_signer(app: tauri::AppHandle) -> Result<(), String> {
    ensure_signer_access()?;

    if let Some(window) = app.get_webview_window("license-signer") {
        window
            .show()
            .map_err(|err| format!("显示签名窗口失败: {err}"))?;
        window
            .set_focus()
            .map_err(|err| format!("聚焦签名窗口失败: {err}"))?;
        return Ok(());
    }

    WebviewWindowBuilder::new(
        &app,
        "license-signer",
        WebviewUrl::App("index.html?mode=license-signer".into()),
    )
    .title("License Signer")
    .inner_size(980.0, 820.0)
    .min_inner_size(860.0, 720.0)
    .center()
    .resizable(true)
    .visible(true)
    .build()
    .map_err(|err| format!("创建签名窗口失败: {err}"))?;

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
        .setup(|app| {
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())
                .map_err(|err| -> Box<dyn std::error::Error> { Box::new(err) })?;

            Ok(())
        })
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let main = app.get_webview_window("main").expect("no main window");
            main.set_focus().unwrap();
            main.show().unwrap();
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            show_window,
            get_activation_request,
            get_license_status,
            import_license,
            get_signer_status,
            sign_activation_license,
            open_license_signer,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
