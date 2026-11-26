mod adb;
mod commands;
mod error;
mod monitor;
mod scrcpy;
mod settings;
mod state;
mod tools;
mod tray;
mod updater;

use settings::Settings;
use state::AppState;
use std::sync::atomic::Ordering;
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;

fn default_output_dir(app: &tauri::AppHandle) -> String {
    let base = app
        .path()
        .download_dir()
        .or_else(|_| app.path().home_dir())
        .unwrap_or_default();
    base.join("ScrcpyGUI").to_string_lossy().into_owned()
}

pub fn run() {
    tauri::Builder::default()
        // Launching the app a second time just brings the existing window back.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .setup(|app| {
            let path = app.path().app_config_dir()?.join("settings.json");
            let mut settings = Settings::load(&path);
            if settings.output_dir.trim().is_empty() {
                settings.output_dir = default_output_dir(app.handle());
            }
            let auto_update = settings.auto_update_scrcpy && settings.scrcpy_path.trim().is_empty();
            app.manage(AppState::new(settings, path));

            // Pick the newest of the bundled and previously downloaded scrcpy before anything
            // runs, then look for a newer release in the background.
            updater::refresh_current(app.handle());
            if auto_update {
                tauri::async_runtime::spawn(updater::check_and_install(app.handle().clone()));
            }

            match tray::setup(app.handle()) {
                Ok(()) => app
                    .state::<AppState>()
                    .tray_ok
                    .store(true, Ordering::SeqCst),
                Err(e) => eprintln!("system tray unavailable: {e}"),
            }

            monitor::spawn(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the window while mirroring keeps sessions alive in the tray.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<AppState>();
                let keep = state.settings().keep_in_tray;
                if keep && state.has_sessions() && state.tray_ok.load(Ordering::SeqCst) {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::check_tools,
            commands::list_devices,
            commands::start_session,
            commands::stop_session,
            commands::list_sessions,
            commands::wifi_enable,
            commands::wifi_connect,
            commands::wifi_disconnect,
            commands::wifi_pair,
            commands::take_screenshot,
            commands::reveal_path,
            commands::get_autostart,
            commands::set_autostart,
            commands::get_update_status,
            commands::check_scrcpy_update,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Scrcpy GUI")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                scrcpy::shutdown(app);
            }
        });
}
