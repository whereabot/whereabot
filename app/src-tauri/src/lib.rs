#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_websocket::init())
        .plugin(tauri_plugin_geolocation::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_mobile_push::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
