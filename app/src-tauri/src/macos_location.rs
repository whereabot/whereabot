//! Mac app location. tauri-plugin-geolocation is a stub on desktop and WKWebView denies
//! navigator.geolocation, so the Mac app asks CoreLocation directly: one fix per call.
//! Errors reach JS as "permission" or "no_fix" (app/src/macos-location.ts).

use std::cell::RefCell;
use std::sync::{Mutex, mpsc};
use std::time::Duration;

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{MainThreadMarker, MainThreadOnly, msg_send};
use objc2_core_location::{CLLocationManager, kCLLocationAccuracyBest};
use tauri::{AppHandle, Runtime};

use crate::macos_location_delegate::{Answer, Delegate, Ivars, Op};
use crate::macos_location_json::state;

thread_local! {
    // CLLocationManager holds its delegate weakly: keep both alive until they answer.
    static ACTIVE: RefCell<Vec<(Retained<CLLocationManager>, Retained<Delegate>)>> = const { RefCell::new(Vec::new()) };
}

fn start(mtm: MainThreadMarker, op: Op, tx: mpsc::Sender<Answer>) {
    let manager = unsafe { CLLocationManager::new() };
    if op == Op::Check {
        let _ = tx.send(Ok(state(unsafe { manager.authorizationStatus() }).into()));
        return;
    }
    let delegate: Retained<Delegate> = {
        let this = Delegate::alloc(mtm).set_ivars(Ivars { op, tx: Mutex::new(Some(tx)) });
        unsafe { msg_send![super(this), init] }
    };
    unsafe {
        manager.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
        if op == Op::Fix {
            // requestLocation waits for this accuracy or its own timeout (about 10 s), then answers once.
            manager.setDesiredAccuracy(kCLLocationAccuracyBest);
            manager.requestLocation();
        } else {
            manager.requestWhenInUseAuthorization();
        }
    }
    ACTIVE.with_borrow_mut(|a| {
        a.retain(|(_, d)| d.pending());
        a.push((manager, delegate));
    });
}

async fn ask<R: Runtime>(app: AppHandle<R>, op: Op, wait: Duration) -> Answer {
    let (tx, rx) = mpsc::channel();
    app.run_on_main_thread(move || start(MainThreadMarker::new().expect("main thread"), op, tx))
        .map_err(|e| e.to_string())?;
    let got = tauri::async_runtime::spawn_blocking(move || rx.recv_timeout(wait)).await;
    got.map_err(|e| e.to_string())?.unwrap_or_else(|_| Err("no_fix".into()))
}

/// "granted", "denied" or "prompt". With `request`, shows the macOS prompt when not decided yet.
#[tauri::command]
pub async fn macos_location_permission<R: Runtime>(app: AppHandle<R>, request: bool) -> Answer {
    let (op, wait) = if request { (Op::Request, 120) } else { (Op::Check, 5) };
    ask(app, op, Duration::from_secs(wait)).await
}

/// One fix: { timestamp, coords }.
#[tauri::command]
pub async fn macos_location<R: Runtime>(app: AppHandle<R>) -> Answer {
    ask(app, Op::Fix, Duration::from_secs(15)).await
}
