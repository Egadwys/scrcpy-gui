use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

/// Options passed to scrcpy for one mirroring session.
/// `0` / empty string means "leave scrcpy's default".
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LaunchOptions {
    /// --max-size (pixels, 0 = device resolution)
    pub max_size: u32,
    /// --video-bit-rate in Mbps (0 = default 8M)
    pub video_bit_rate: u32,
    /// --max-fps (0 = default)
    pub max_fps: u32,
    /// "", "h264", "h265", "av1"
    pub video_codec: String,
    /// "", "0", "90", "180", "270"
    pub orientation: String,
    pub window_title: String,
    pub turn_screen_off: bool,
    pub stay_awake: bool,
    pub always_on_top: bool,
    pub fullscreen: bool,
    pub borderless: bool,
    pub no_audio: bool,
    /// View only: --no-control
    pub no_control: bool,
    pub show_touches: bool,
    /// Record the session to a .mkv file in the output directory
    pub record: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub adb_path: String,
    pub scrcpy_path: String,
    pub output_dir: String,
    /// "system" | "light" | "dark"
    pub theme: String,
    pub keep_in_tray: bool,
    pub wifi_port: u16,
    /// Check GitHub for a newer scrcpy on every start and switch to it automatically
    pub auto_update_scrcpy: bool,
    /// Extra raw scrcpy arguments appended to every launch
    pub extra_args: String,
    pub defaults: LaunchOptions,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            adb_path: String::new(),
            scrcpy_path: String::new(),
            output_dir: String::new(),
            theme: "system".into(),
            keep_in_tray: true,
            wifi_port: 5555,
            auto_update_scrcpy: true,
            extra_args: String::new(),
            defaults: LaunchOptions::default(),
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}
