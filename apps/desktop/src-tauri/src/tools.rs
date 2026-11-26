//! Locating and inspecting the external `adb` and `scrcpy` binaries.

use crate::{
    error::{AppError, Result},
    settings::Settings,
    updater::{self, Source},
};
use serde::Serialize;
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

/// Builds a tokio command that never flashes a console window on Windows.
pub fn command(program: &Path) -> tokio::process::Command {
    #[allow(unused_mut)]
    let mut c = tokio::process::Command::new(program);
    #[cfg(windows)]
    c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    c
}

pub fn exe(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

/// Resolves a user supplied path. A directory is accepted and joined with `name`.
fn from_custom(custom: &str, name: &str) -> Result<PathBuf> {
    let mut p = PathBuf::from(custom);
    if p.is_dir() {
        p = p.join(exe(name));
    }
    if p.is_file() {
        Ok(p)
    } else {
        Err(AppError::msg(format!(
            "Configured {name} path does not exist: {custom}"
        )))
    }
}

/// Order: configured path, the bundled / auto-updated copy, `scrcpy` on PATH.
/// Also returns where it came from: custom | downloaded | bundled | system.
pub fn resolve_scrcpy_src(s: &Settings) -> Result<(PathBuf, &'static str)> {
    let custom = s.scrcpy_path.trim();
    if !custom.is_empty() {
        return from_custom(custom, "scrcpy").map(|p| (p, "custom"));
    }
    if let Some(home) = updater::current() {
        let p = home.dir.join(exe("scrcpy"));
        if p.is_file() {
            let src = match home.source {
                Source::Bundled => "bundled",
                Source::Downloaded => "downloaded",
            };
            return Ok((p, src));
        }
    }
    which::which("scrcpy")
        .map(|p| (p, "system"))
        .map_err(|_| AppError::ScrcpyNotFound)
}

pub fn resolve_scrcpy(s: &Settings) -> Result<PathBuf> {
    resolve_scrcpy_src(s).map(|(p, _)| p)
}

/// Order: configured path, the `adb` shipped next to the scrcpy in use, `adb` on PATH.
/// Using the sibling keeps scrcpy and adb on the same version. The result is also exported
/// to scrcpy as `ADB` so both always talk to the same adb server.
pub fn resolve_adb(s: &Settings) -> Result<PathBuf> {
    let custom = s.adb_path.trim();
    if !custom.is_empty() {
        return from_custom(custom, "adb");
    }
    if let Ok(scrcpy) = resolve_scrcpy(s) {
        if let Some(dir) = scrcpy.parent() {
            let sibling = dir.join(exe("adb"));
            if sibling.is_file() {
                return Ok(sibling);
            }
        }
    }
    which::which("adb").map_err(|_| AppError::AdbNotFound)
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ToolInfo {
    pub path: Option<String>,
    /// custom | downloaded | bundled | system
    pub source: Option<String>,
    pub version: Option<String>,
    pub error: Option<String>,
}

pub async fn inspect(resolved: Result<PathBuf>, source: Option<&str>, version_arg: &str) -> ToolInfo {
    match resolved {
        Err(e) => ToolInfo {
            path: None,
            source: None,
            version: None,
            error: Some(e.to_string()),
        },
        Ok(p) => {
            let version = version_of(&p, version_arg).await;
            ToolInfo {
                path: Some(p.display().to_string()),
                source: source.map(str::to_string),
                version,
                error: None,
            }
        }
    }
}

async fn version_of(program: &Path, arg: &str) -> Option<String> {
    let mut cmd = command(program);
    cmd.arg(arg).stdin(Stdio::null()).kill_on_drop(true);
    let out = tokio::time::timeout(Duration::from_secs(5), cmd.output())
        .await
        .ok()?
        .ok()?;
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
}

/// Makes a device serial safe to use inside a file name.
pub fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_replaces_unsafe_chars() {
        assert_eq!(sanitize("192.168.1.5:5555"), "192_168_1_5_5555");
        assert_eq!(sanitize("R58M-abc_1"), "R58M-abc_1");
    }
}
