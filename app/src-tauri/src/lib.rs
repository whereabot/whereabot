#[cfg(target_os = "macos")]
mod macos_location;
#[cfg(target_os = "macos")]
mod macos_location_delegate;
#[cfg(target_os = "macos")]
mod macos_location_json;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();
    // Desktop: the OAuth callback opens a second instance on Windows and Linux.
    // This forwards its whereabouts:// URL to the running one (deep-link feature). Must be first.
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
        use tauri::Manager;
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.set_focus();
        }
    }));
    // Mac: CoreLocation, since neither the plugin nor WKWebView gives a fix there.
    #[cfg(target_os = "macos")]
    let builder = builder.invoke_handler(tauri::generate_handler![
        macos_location::macos_location,
        macos_location::macos_location_permission
    ]);
    builder
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_websocket::init())
        .plugin(tauri_plugin_geolocation::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_mobile_push::init())
        .plugin(tauri_plugin_background_answer::init())
        // Desktop uses the webview's navigator.geolocation (the plugin is a stub there).
        // WebKitGTK denies it unless allowed here; the in-app approve step is the consent.
        .on_permission_request(|_, kind| match kind {
            tauri::webview::PermissionKind::Geolocation => tauri::webview::PermissionResponse::Allow,
            _ => tauri::webview::PermissionResponse::Default,
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
