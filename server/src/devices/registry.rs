use std::collections::HashMap;

use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;

use crate::protocol::ToDevice;

pub type Tx = mpsc::Sender<ToDevice>;

const REPLACED: ToDevice = ToDevice::Close {
    code: 4000,
    reason: "replaced",
};

/// Live sockets: user -> device key -> sender. Before `hello` a socket's key is random.
#[derive(Default)]
pub struct Registry(Mutex<HashMap<Uuid, HashMap<Uuid, Tx>>>);

impl Registry {
    /// Adds a socket. The same device key replaces (and closes) the old one.
    pub async fn insert(&self, user: Uuid, device: Uuid, tx: Tx) {
        let old = self
            .0
            .lock()
            .await
            .entry(user)
            .or_default()
            .insert(device, tx);
        if let Some(old) = old {
            let _ = old.try_send(REPLACED);
        }
    }

    /// Gives a socket its real device key after `hello`.
    pub async fn rekey(&self, user: Uuid, from: Uuid, to: Uuid, tx: &Tx) {
        let mut all = self.0.lock().await;
        let devices = all.entry(user).or_default();
        if devices.get(&from).is_some_and(|c| c.same_channel(tx)) {
            devices.remove(&from);
        }
        if let Some(old) = devices.insert(to, tx.clone())
            && !old.same_channel(tx)
        {
            let _ = old.try_send(REPLACED);
        }
    }

    /// Removes the socket unless a newer one took its key.
    pub async fn remove(&self, user: Uuid, device: Uuid, tx: &Tx) {
        let mut all = self.0.lock().await;
        let Some(devices) = all.get_mut(&user) else {
            return;
        };
        if devices.get(&device).is_some_and(|c| c.same_channel(tx)) {
            devices.remove(&device);
        }
        if devices.is_empty() {
            all.remove(&user);
        }
    }

    pub async fn senders(&self, user: Uuid) -> Vec<(Uuid, Tx)> {
        let all = self.0.lock().await;
        all.get(&user)
            .map(|d| d.iter().map(|(k, v)| (*k, v.clone())).collect())
            .unwrap_or_default()
    }

    pub async fn close_all(&self, code: u16, reason: &'static str) {
        for devices in self.0.lock().await.values() {
            for tx in devices.values() {
                let _ = tx.try_send(ToDevice::Close { code, reason });
            }
        }
    }
}
