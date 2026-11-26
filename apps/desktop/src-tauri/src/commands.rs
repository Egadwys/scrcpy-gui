use crate::{
    adb::{self, DeviceSnapshot},
    error::{AppError, Result},
    scrcpy,
    settings::{LaunchOptions, Settings},
    state::{AppState, SessionInfo},
    tools::{self, ToolInfo},
    updater::{self, UpdateStatus},
};
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, State};

#[derive(Serialize)]
pub struct ToolStatus {
    pub adb: ToolInfo,
    pub scrcpy: ToolInfo,
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings()
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: Settings) -> Result<()> {
    let mut settings = settings;
    if settings.wifi_port == 0 {
        settings.wifi_port = 5555;
    }
    settings.save(&state.settings_path)?;
    *state.settings.lock().unwrap() = settings;
    Ok(())
}

#[tauri::command]
pub async fn check_tools(state: State<'_, AppState>) -> Result<ToolStatus> {
    let s = state.settings();
    let (scrcpy_path, scrcpy_source) = match tools::resolve_scrcpy_src(&s) {
        Ok((p, src)) => (Ok(p), Some(src)),
        Err(e) => (Err(e), None),
    };
    Ok(ToolStatus {
        adb: tools::inspect(tools::resolve_adb(&s), None, "version").await,
        scrcpy: tools::inspect(scrcpy_path, scrcpy_source, "--version").await,
    })
}

#[tauri::command]
pub async fn list_devices(state: State<'_, AppState>) -> Result<DeviceSnapshot> {
    Ok(adb::snapshot(&state.settings()).await)
}

#[tauri::command]
pub async fn start_session(app: AppHandle, serial: String, options: LaunchOptions) -> Result<()> {
    scrcpy::start(&app, serial, options).await
}

#[tauri::command]
pub fn stop_session(app: AppHandle, serial: String) -> Result<()> {
    scrcpy::stop(&app, &serial)
}

#[tauri::command]
pub fn list_sessions(state: State<'_, AppState>) -> Vec<SessionInfo> {
    scrcpy::list_sessions(&state)
}

#[tauri::command]
pub async fn wifi_enable(state: State<'_, AppState>, serial: String) -> Result<String> {
    let s = state.settings();
    let adb_path = tools::resolve_adb(&s)?;
    adb::enable_wifi(&adb_path, &serial, s.wifi_port).await
}

#[tauri::command]
pub async fn wifi_connect(state: State<'_, AppState>, address: String) -> Result<()> {
    let adb_path = tools::resolve_adb(&state.settings())?;
    adb::connect(&adb_path, address.trim()).await
}

#[tauri::command]
pub async fn wifi_disconnect(state: State<'_, AppState>, address: String) -> Result<()> {
    let adb_path = tools::resolve_adb(&state.settings())?;
    adb::disconnect(&adb_path, address.trim()).await
}

#[tauri::command]
pub async fn wifi_pair(state: State<'_, AppState>, address: String, code: String) -> Result<()> {
    let adb_path = tools::resolve_adb(&state.settings())?;
    adb::pair(&adb_path, address.trim(), code.trim()).await
}

#[tauri::command]
pub async fn take_screenshot(state: State<'_, AppState>, serial: String) -> Result<String> {
    let s = state.settings();
    let adb_path = tools::resolve_adb(&s)?;
    let file = adb::screenshot(&adb_path, &serial, &PathBuf::from(&s.output_dir)).await?;
    Ok(file.display().to_string())
}

/// Opens a folder (or the folder containing a file) in the system file manager.
#[tauri::command]
pub fn reveal_path(app: AppHandle, path: String) -> Result<()> {
    use tauri_plugin_opener::OpenerExt;
    let p = PathBuf::from(&path);
    let target = if p.is_dir() {
        p
    } else {
        p.parent().map(|d| d.to_path_buf()).unwrap_or(p)
    };
    app.opener()
        .open_path(target.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| AppError::msg(e.to_string()))
}

#[tauri::command]
pub fn get_autostart(app: AppHandle) -> Result<bool> {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch()
        .is_enabled()
        .map_err(|e| AppError::msg(e.to_string()))
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<()> {
    use tauri_plugin_autostart::ManagerExt;
    let launcher = app.autolaunch();
    let result = if enabled { launcher.enable() } else { launcher.disable() };
    result.map_err(|e| AppError::msg(e.to_string()))
}

#[tauri::command]
pub fn get_update_status() -> Option<UpdateStatus> {
    updater::status_now()
}

/// Starts a scrcpy update check in the background; progress arrives as `scrcpy-update` events.
#[tauri::command]
pub fn check_scrcpy_update(app: AppHandle, state: State<'_, AppState>) -> Result<()> {
    if !state.settings().scrcpy_path.trim().is_empty() {
        return Err(AppError::msg(
            "A custom scrcpy path is set in Preferences. Updates only manage the built-in copy.",
        ));
    }
    tauri::async_runtime::spawn(updater::check_and_install(app));
    Ok(())
}
