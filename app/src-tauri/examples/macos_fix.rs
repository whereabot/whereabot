//! One real CoreLocation fix through the app's own `macos_location` command, without sign-in.
//! Prints coordinates rounded to 2 decimals only. `--request` first shows the macOS prompt.
//! macOS grants location per app bundle: run this binary as a bundle's executable, launched with `open`.

#[cfg(target_os = "macos")]
#[path = "../src/macos_location.rs"]
mod macos_location;
#[cfg(target_os = "macos")]
#[path = "../src/macos_location_delegate.rs"]
mod macos_location_delegate;
#[cfg(target_os = "macos")]
#[path = "../src/macos_location_json.rs"]
mod macos_location_json;

#[cfg(target_os = "macos")]
fn main() {
    let mut context = tauri::generate_context!();
    context.config_mut().app.windows.clear();
    tauri::Builder::default()
        .setup(|app| {
            let app = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let request = std::env::args().any(|a| a == "--request");
                let state = macos_location::macos_location_permission(app.clone(), request).await;
                println!("permission: {state:?}");
                match macos_location::macos_location(app.clone()).await {
                    Ok(p) => {
                        let c = &p["coords"];
                        let round = |k: &str| c[k].as_f64().map(|v| format!("{v:.2}"));
                        println!(
                            "fix: lat {:?} lon {:?} accuracy {} m timestamp {}",
                            round("latitude"),
                            round("longitude"),
                            c["accuracy"],
                            p["timestamp"]
                        );
                    }
                    Err(e) => println!("fix error: {e}"),
                }
                app.exit(0);
            });
            Ok(())
        })
        .run(context)
        .expect("tauri runtime");
}

#[cfg(not(target_os = "macos"))]
fn main() {}
