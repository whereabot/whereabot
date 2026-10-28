//! Native iOS half of the background answer: notification actions on the locate push,
//! a one-shot location read, and the POST to the relay. See docs/architecture.md.

use tauri::{
    Runtime,
    plugin::{Builder, TauriPlugin},
};

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_background_answer);

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::<R, ()>::new("background-answer")
        .setup(|_app, _api| {
            // The Swift init installs the notification delegate, so it must run at launch.
            #[cfg(target_os = "ios")]
            _api.register_ios_plugin(init_plugin_background_answer)?;
            Ok(())
        })
        .build()
}
