use tauri::{App, Manager};
use window_vibrancy::*;

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

pub fn set_acrylic_theme(app: &mut App) {
    let window = app.get_webview_window("main").unwrap();
    #[cfg(target_os = "windows")]
    apply_acrylic(&window, None).unwrap();
}
pub fn set_acrylic_theme_extra(app: &mut App) {
    let window = app.get_webview_window("main").unwrap();
    #[cfg(target_os = "windows")]
    apply_acrylic(&window, Some((18, 18, 18, 125))).unwrap();
}

pub fn set_vibrancy_theme(app: &mut App) {
    let window = app.get_webview_window("main").unwrap();
    apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None)
        .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![show_window])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            set_vibrancy_theme(app);
            #[cfg(target_os = "windows")]
            set_acrylic_theme_extra(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
