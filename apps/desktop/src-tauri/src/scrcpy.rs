//! Launching, supervising and stopping scrcpy processes.

use crate::{
    error::{AppError, Result},
    settings::LaunchOptions,
    state::{AppState, Session, SessionInfo},
    tools, updater,
};
use chrono::Local;
use serde::Serialize;
use std::{
    path::Path,
    process::Stdio,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

#[derive(Serialize, Clone)]
struct LogLine {
    serial: String,
    stream: &'static str,
    line: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SessionEnded {
    serial: String,
    code: Option<i32>,
    recording: Option<String>,
}

/// Translates UI options into scrcpy (>= 3.0) command line arguments.
pub fn build_args(
    serial: &str,
    o: &LaunchOptions,
    extra: &str,
    record: Option<&Path>,
) -> Result<Vec<String>> {
    // scrcpy must never wait for a key press on exit: there is no console to press it in.
    let mut a: Vec<String> = vec![format!("--serial={serial}"), "--pause-on-exit=false".to_string()];

    if o.max_size > 0 {
        a.push(format!("--max-size={}", o.max_size));
    }
    if o.video_bit_rate > 0 {
        a.push(format!("--video-bit-rate={}M", o.video_bit_rate));
    }
    if o.max_fps > 0 {
        a.push(format!("--max-fps={}", o.max_fps));
    }
    if matches!(o.video_codec.as_str(), "h264" | "h265" | "av1") {
        a.push(format!("--video-codec={}", o.video_codec));
    }
    if matches!(o.orientation.as_str(), "0" | "90" | "180" | "270") {
        a.push(format!("--orientation={}", o.orientation));
    }
    let title = o.window_title.trim();
    if !title.is_empty() {
        a.push(format!("--window-title={title}"));
    }

    let flags = [
        (o.turn_screen_off, "--turn-screen-off"),
        (o.stay_awake, "--stay-awake"),
        (o.always_on_top, "--always-on-top"),
        (o.fullscreen, "--fullscreen"),
        (o.borderless, "--window-borderless"),
        (o.no_audio, "--no-audio"),
        (o.no_control, "--no-control"),
        (o.show_touches, "--show-touches"),
    ];
    for (enabled, flag) in flags {
        if enabled {
            a.push(flag.to_string());
        }
    }

    if let Some(path) = record {
        a.push(format!("--record={}", path.display()));
    }

    if !extra.trim().is_empty() {
        let parts = shell_words::split(extra)
            .map_err(|e| AppError::msg(format!("Invalid extra arguments: {e}")))?;
        a.extend(parts);
    }
    Ok(a)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn list_sessions(state: &AppState) -> Vec<SessionInfo> {
    let mut v: Vec<SessionInfo> = state
        .sessions
        .lock()
        .unwrap()
        .iter()
        .map(|(serial, s)| SessionInfo {
            serial: serial.clone(),
            pid: s.pid,
            started_at: s.started_at,
            recording: s.recording.clone(),
        })
        .collect();
    v.sort_by(|a, b| a.serial.cmp(&b.serial));
    v
}

fn emit_sessions(app: &AppHandle) {
    let state = app.state::<AppState>();
    let list = list_sessions(&state);
    let _ = app.emit("sessions-changed", list);
}

fn pump<R>(app: AppHandle, serial: String, stream: &'static str, reader: R)
where
    R: AsyncRead + Unpin + Send + 'static,
{
    tauri::async_runtime::spawn(async move {
        let mut reader = BufReader::new(reader);
        let mut buf = Vec::new();
        loop {
            buf.clear();
            match reader.read_until(b'\n', &mut buf).await {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    let line = String::from_utf8_lossy(&buf).trim_end().to_string();
                    if !line.is_empty() {
                        let _ = app.emit(
                            "log",
                            LogLine {
                                serial: serial.clone(),
                                stream,
                                line,
                            },
                        );
                    }
                }
            }
        }
    });
}

/// Must run inside the tokio runtime (hence `async`), because it spawns a child process.
pub async fn start(app: &AppHandle, serial: String, options: LaunchOptions) -> Result<()> {
    let state = app.state::<AppState>();
    let settings = state.settings();

    if state.sessions.lock().unwrap().contains_key(&serial) {
        return Err(AppError::msg("This device is already being mirrored."));
    }

    let scrcpy = tools::resolve_scrcpy(&settings)?;
    let adb = tools::resolve_adb(&settings)?;

    let record_path = if options.record {
        let dir = std::path::PathBuf::from(&settings.output_dir);
        std::fs::create_dir_all(&dir)?;
        Some(dir.join(format!(
            "{}-{}.mkv",
            tools::sanitize(&serial),
            Local::now().format("%Y%m%d-%H%M%S")
        )))
    } else {
        None
    };

    let args = build_args(&serial, &options, &settings.extra_args, record_path.as_deref())?;

    let mut cmd = tools::command(&scrcpy);
    cmd.args(&args)
        .env("ADB", &adb)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let mut child = cmd
        .spawn()
        .map_err(|e| AppError::msg(format!("Could not start scrcpy: {e}")))?;
    let pid = child
        .id()
        .ok_or_else(|| AppError::msg("scrcpy exited immediately."))?;
    let recording = record_path.map(|p| p.display().to_string());

    if let Some(out) = child.stdout.take() {
        pump(app.clone(), serial.clone(), "stdout", out);
    }
    if let Some(err) = child.stderr.take() {
        pump(app.clone(), serial.clone(), "stderr", err);
    }

    state.sessions.lock().unwrap().insert(
        serial.clone(),
        Session {
            pid,
            started_at: now_secs(),
            recording: recording.clone(),
        },
    );
    emit_sessions(app);

    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        let code = child.wait().await.ok().and_then(|s| s.code());
        {
            let st = app2.state::<AppState>();
            let mut sessions = st.sessions.lock().unwrap();
            if sessions.get(&serial).map(|s| s.pid) == Some(pid) {
                sessions.remove(&serial);
            }
        }
        let _ = app2.emit(
            "session-ended",
            SessionEnded {
                serial,
                code,
                recording,
            },
        );
        emit_sessions(&app2);
        // A scrcpy update downloaded mid-session is only activated once nothing is running,
        // because switching adb versions would restart the adb server under a live session.
        if !app2.state::<AppState>().has_sessions() {
            updater::apply_pending(&app2);
        }
    });

    Ok(())
}

/// Sends a termination request. Graceful by default so scrcpy can finalize recordings;
/// `force` kills the process tree.
pub fn terminate(pid: u32, force: bool) {
    let pid_s = pid.to_string();
    #[cfg(unix)]
    {
        let _ = std::process::Command::new("kill")
            .arg(if force { "-KILL" } else { "-TERM" })
            .arg(&pid_s)
            .status();
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let mut c = std::process::Command::new("taskkill");
        c.args(["/PID", pid_s.as_str(), "/T"]);
        if force {
            c.arg("/F");
        }
        c.creation_flags(0x0800_0000);
        let _ = c.status();
    }
}

pub fn stop(app: &AppHandle, serial: &str) -> Result<()> {
    let state = app.state::<AppState>();
    let pid = state
        .sessions
        .lock()
        .unwrap()
        .get(serial)
        .map(|s| s.pid)
        .ok_or_else(|| AppError::msg("No active session for this device."))?;

    terminate(pid, false);

    // If scrcpy ignores the polite request, force it after a short grace period.
    let app2 = app.clone();
    let serial = serial.to_string();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(3)).await;
        let st = app2.state::<AppState>();
        let still_running = st.sessions.lock().unwrap().get(&serial).map(|s| s.pid) == Some(pid);
        if still_running {
            terminate(pid, true);
        }
    });
    Ok(())
}

pub fn stop_all(app: &AppHandle) {
    let state = app.state::<AppState>();
    let serials: Vec<String> = state.sessions.lock().unwrap().keys().cloned().collect();
    for s in serials {
        let _ = stop(app, &s);
    }
}

/// Called when the app exits: stop every scrcpy so none is left orphaned.
pub fn shutdown(app: &AppHandle) {
    let state = app.state::<AppState>();
    let pids: Vec<u32> = state.sessions.lock().unwrap().values().map(|s| s.pid).collect();
    if pids.is_empty() {
        return;
    }
    for p in &pids {
        terminate(*p, false);
    }
    for _ in 0..20 {
        if state.sessions.lock().unwrap().is_empty() {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    for p in &pids {
        terminate(*p, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options_only_select_device() {
        let a = build_args("abc", &LaunchOptions::default(), "", None).unwrap();
        assert_eq!(a, vec!["--serial=abc", "--pause-on-exit=false"]);
    }

    #[test]
    fn maps_options_to_flags() {
        let o = LaunchOptions {
            max_size: 1024,
            video_bit_rate: 12,
            max_fps: 60,
            video_codec: "h265".into(),
            orientation: "90".into(),
            turn_screen_off: true,
            no_audio: true,
            window_title: "My phone".into(),
            ..Default::default()
        };
        let a = build_args("abc", &o, "--crop=1224:1440:0:0 --disable-screensaver", None).unwrap();
        assert!(a.contains(&"--max-size=1024".to_string()));
        assert!(a.contains(&"--video-bit-rate=12M".to_string()));
        assert!(a.contains(&"--max-fps=60".to_string()));
        assert!(a.contains(&"--video-codec=h265".to_string()));
        assert!(a.contains(&"--orientation=90".to_string()));
        assert!(a.contains(&"--turn-screen-off".to_string()));
        assert!(a.contains(&"--no-audio".to_string()));
        assert!(a.contains(&"--window-title=My phone".to_string()));
        assert!(a.contains(&"--disable-screensaver".to_string()));
    }

    #[test]
    fn ignores_unknown_codec_and_rejects_bad_extra() {
        let o = LaunchOptions {
            video_codec: "bogus".into(),
            ..Default::default()
        };
        assert_eq!(build_args("s", &o, "", None).unwrap().len(), 2);
        assert!(build_args("s", &o, "\"unterminated", None).is_err());
    }
}
