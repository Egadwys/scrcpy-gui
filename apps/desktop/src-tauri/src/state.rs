use crate::settings::Settings;
use serde::Serialize;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{atomic::AtomicBool, Mutex},
};

#[derive(Clone, Debug)]
pub struct Session {
    pub pid: u32,
    pub started_at: u64,
    pub recording: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub serial: String,
    pub pid: u32,
    pub started_at: u64,
    pub recording: Option<String>,
}

pub struct AppState {
    pub settings: Mutex<Settings>,
    pub settings_path: PathBuf,
    /// Running scrcpy processes keyed by device serial
    pub sessions: Mutex<HashMap<String, Session>>,
    /// False when the system tray could not be created (e.g. Linux without AppIndicator)
    pub tray_ok: AtomicBool,
}

impl AppState {
    pub fn new(settings: Settings, settings_path: PathBuf) -> Self {
        Self {
            settings: Mutex::new(settings),
            settings_path,
            sessions: Mutex::new(HashMap::new()),
            tray_ok: AtomicBool::new(false),
        }
    }

    pub fn settings(&self) -> Settings {
        self.settings.lock().unwrap().clone()
    }

    pub fn has_sessions(&self) -> bool {
        !self.sessions.lock().unwrap().is_empty()
    }
}
