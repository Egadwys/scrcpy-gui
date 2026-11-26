# Architecture

Scrcpy GUI is a [Tauri 2](https://tauri.app) app: a Rust backend that owns every external process, and a Svelte 5 + TypeScript frontend that only renders state and calls commands.

```
apps/desktop
├── src/                    Svelte frontend
│   ├── App.svelte          single-device screen: status text, pill button, panel routing
│   └── lib/
│       ├── store.svelte.ts reactive state + event listeners (single source of truth)
│       ├── api.ts          typed wrappers around Tauri commands
│       ├── PhoneStage.svelte  phone illustration (none / ready / live / warn)
│       ├── Modal.svelte, OptionsPanel.svelte, WifiPanel.svelte,
│       │   LogPanel.svelte (bottom sheet), Preferences.svelte, Toasts.svelte
└── src-tauri/src           Rust backend
    ├── lib.rs              app setup, plugins, window-close behaviour, exit cleanup
    ├── commands.rs         #[tauri::command] surface exposed to the frontend
    ├── adb.rs              adb wrapper: device list parsing, Wi-Fi, pairing, screenshot
    ├── scrcpy.rs           argument building, spawning, supervising, stopping scrcpy
    ├── tools.rs            locating/inspecting adb and scrcpy binaries
    ├── updater.rs          bundled + downloaded scrcpy, GitHub release check, install
    ├── monitor.rs          polls adb every 2 s, emits `devices-changed` on change
    ├── tray.rs             system tray menu
    ├── settings.rs         Settings / LaunchOptions, JSON persistence
    ├── state.rs            AppState: settings + running sessions
    └── error.rs            AppError (serialized to the UI as a plain string)
```

## Data flow

- **UI**: one device is shown at a time (like gnirehtet-gui). With several devices a picker appears; the device that is mirroring is preferred. Log, Preferences, Options and Wireless open as overlays.
- **Devices**: `monitor` runs `adb devices -l` every 2 seconds and emits `devices-changed` only when the parsed snapshot differs. The store replaces its device list; cards are keyed by serial so per-card option edits survive refreshes.
- **Sessions**: `start_session` builds the argument list from `LaunchOptions`, spawns scrcpy with `ADB` set to the resolved adb, registers the PID in `AppState.sessions` and emits `sessions-changed`. Two tasks forward stdout/stderr lines as `log` events; a third awaits the process, removes the session and emits `session-ended` plus `sessions-changed`.
- **Stopping**: `terminate` sends SIGTERM (Unix) or `taskkill /T` (Windows) so scrcpy can close and finalize recordings. If the process is still alive after 3 seconds it is force killed.
- **Exit**: on `RunEvent::Exit` every remaining session is terminated (graceful, then forced after 2 s), so no scrcpy is orphaned.
- **Window close**: when sessions are running, tray is available and *keep in tray* is on, the window hides instead of closing; otherwise the app exits. A second launch just shows the existing window (single-instance plugin).

## Bundled scrcpy and auto-update

`scripts/fetch-scrcpy.mjs` downloads the official scrcpy archive for the build platform (version pinned in `scripts/scrcpy-version.txt`, verified against the release's `SHA256SUMS.txt`) into `src-tauri/resources/scrcpy`, and `tauri.conf.json` bundles that folder as a resource. It runs automatically before `tauri dev` and `tauri build`.

At runtime `updater.rs` knows two kinds of copies: the bundled one and downloaded ones under `<app local data>/scrcpy/<tag>/`. `refresh_current` picks the newest by version and stores it in a global; `tools::resolve_scrcpy` returns, in order: the custom path from Preferences, that copy, `scrcpy` on PATH. `resolve_adb` prefers the adb next to the scrcpy in use, then PATH.

At startup (unless disabled or a custom scrcpy path is set) `check_and_install` runs in the background:

1. `GET /repos/Genymobile/scrcpy/releases/latest`; stop if the tag is not newer than the current copy.
2. Pick `scrcpy-<platform>-<tag>.zip|tar.gz` and `SHA256SUMS.txt`; both must come from `github.com/Genymobile/scrcpy/releases/download/`.
3. Stream the archive to disk while hashing; discard it on mismatch.
4. Unpack into `.tmp-<tag>`, check that the executable is there, write a `.complete` marker, and rename the folder to `<tag>`. A copy without the marker is ignored, so an interrupted install can never be selected.
5. Switch to it with `refresh_current`, or, if sessions are running, set a pending flag. `scrcpy.rs` calls `apply_pending` when the last session ends, because replacing adb under a live session would restart the adb server and drop it.
6. Keep the two newest downloaded copies, delete the rest.

Progress and results are sent as `scrcpy-update` events; the last status is also available through `get_update_status`.

Not done: the GPG signature of `SHA256SUMS.txt` is not verified. The checksum protects against corruption and tampering in transit, not against a compromised GitHub release. Turn automatic updates off in Preferences if that matters to you.

## Design choices

- The app drives external binaries instead of linking scrcpy, so any scrcpy 3.0+ works (orientation flags). The bundled and auto-updated copy keeps users on a current one.
- Settings live in `settings.json` in the OS config directory (not browser storage); unknown/missing fields fall back to defaults, so older files keep working.
- Rust errors are plain strings in the UI; the frontend never parses them.
- Everything that talks to adb is `async` with a timeout, and the console window is suppressed on Windows (`CREATE_NO_WINDOW`).

## Adding a scrcpy option

1. Add the field to `LaunchOptions` in `settings.rs`.
2. Map it in `build_args` in `scrcpy.rs` and add a test.
3. Add it to the `LaunchOptions` interface in `types.ts`.
4. Add the control to `OptionsPanel.svelte`.
