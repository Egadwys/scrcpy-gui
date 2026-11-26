//! Background device polling. Emits `devices-changed` only when the list actually changes.

use crate::{
    adb::{self, DeviceSnapshot},
    state::AppState,
};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

pub fn spawn(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut last: Option<DeviceSnapshot> = None;
        loop {
            let settings = app.state::<AppState>().settings();
            let snap = adb::snapshot(&settings).await;
            if last.as_ref() != Some(&snap) {
                let _ = app.emit("devices-changed", &snap);
                last = Some(snap);
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
}
