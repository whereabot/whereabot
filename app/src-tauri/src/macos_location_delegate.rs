//! CLLocationManager delegate: turns the first callback into one answer on a channel.

use std::sync::{Mutex, mpsc};

use objc2::{DefinedClass, MainThreadOnly, define_class};
use objc2_core_location::{CLAuthorizationStatus, CLError, CLLocation, CLLocationManager, CLLocationManagerDelegate};
use objc2_foundation::{NSArray, NSError, NSObject, NSObjectProtocol};
use serde_json::Value;

use crate::macos_location_json::{position, state};

pub type Answer = Result<Value, String>;

#[derive(Clone, Copy, PartialEq)]
pub enum Op {
    Check,
    Request,
    Fix,
}

pub struct Ivars {
    pub op: Op,
    pub tx: Mutex<Option<mpsc::Sender<Answer>>>,
}

define_class!(
    // SAFETY: NSObject has no subclassing requirements and Delegate does not implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = Ivars]
    pub struct Delegate;

    unsafe impl NSObjectProtocol for Delegate {}

    unsafe impl CLLocationManagerDelegate for Delegate {
        // Also called once right after the delegate is set, with the current status.
        #[unsafe(method(locationManagerDidChangeAuthorization:))]
        fn did_change_authorization(&self, manager: &CLLocationManager) {
            let status = unsafe { manager.authorizationStatus() };
            if self.ivars().op == Op::Request && status != CLAuthorizationStatus::NotDetermined {
                self.send(Ok(state(status).into()));
            }
        }

        #[unsafe(method(locationManager:didUpdateLocations:))]
        fn did_update_locations(&self, _manager: &CLLocationManager, locations: &NSArray<CLLocation>) {
            if let Some(l) = locations.lastObject() {
                self.send(Ok(position(&l)));
            }
        }

        #[unsafe(method(locationManager:didFailWithError:))]
        fn did_fail(&self, _manager: &CLLocationManager, error: &NSError) {
            let fail = if error.code() == CLError::Denied.0 { "permission" } else { "no_fix" };
            self.send(Err(fail.into()));
        }
    }
);

impl Delegate {
    /// First answer wins; later callbacks are ignored.
    fn send(&self, answer: Answer) {
        if let Some(tx) = self.ivars().tx.lock().ok().and_then(|mut t| t.take()) {
            let _ = tx.send(answer);
        }
    }

    pub fn pending(&self) -> bool {
        self.ivars().tx.lock().is_ok_and(|t| t.is_some())
    }
}

