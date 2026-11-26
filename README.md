# Scrcpy GUI: mirror and control Android from a clean desktop app

Mirror, control, record and screenshot your Android phone with [scrcpy](https://github.com/Genymobile/scrcpy), without touching the terminal. Plug in the phone (or connect over Wi-Fi), click **Start mirroring**, done. Runs on Linux, Windows and macOS.

This is an independent project, not affiliated with or endorsed by Genymobile. It drives the unmodified official `scrcpy` and `adb` binaries; the app adds the interface and the process management around them. Its structure and workflow are modelled on [gnirehtet-gui](https://github.com/sawongam/gnirehtet-gui) (Tauri + Svelte).

## Features

- Detects connected phones live and tells you what to do when one is unauthorized, offline or not accessible (Linux udev permissions).
- Mirror several devices at the same time, one scrcpy window per device.
- **Go wireless** in one click: reads the phone's Wi-Fi IP, switches adb to TCP/IP and connects, so you can unplug the cable. Also manual connect and Android 11+ pairing.
- Per-launch options: resolution, bit rate, FPS, codec (H.264 / H.265 / AV1), orientation, screen off, stay awake, always on top, fullscreen, borderless, no audio, view only, show touches.
- Record sessions to `.mkv` and take PNG screenshots into a folder you choose.
- Live scrcpy log panel for troubleshooting.
- Stays in the system tray while mirroring if you close the window; **Quit** from the tray stops every session cleanly (recordings are finalized).
- **scrcpy and adb are built in.** The installer ships a scrcpy release, and on every start the app checks the official releases and, when a newer one exists, downloads it, verifies its SHA-256 and switches to it automatically.
- Preferences: tool paths, default options, extra raw scrcpy arguments, light/dark theme, run at startup.

## Requirements

- Nothing to install for scrcpy or adb: both are bundled (the installer carries a scrcpy release, which includes adb) and updated automatically. An internet connection is only needed for the update check.
- An Android 5.0+ phone with **USB debugging** enabled.

If you prefer your own scrcpy/adb, set their paths in **Preferences**; a custom path is always used instead of the built-in copy and turns the automatic updates off. See [docs/INSTALL.md](docs/INSTALL.md).

## Usage

1. Enable *USB debugging* on the phone (Settings → About phone → tap *Build number* seven times, then Developer options → USB debugging).
2. Connect the cable and accept *Allow USB debugging?* on the phone.
3. Open Scrcpy GUI, click **Start mirroring** on your device.

To go wireless, click **Go wireless** on the device card while it is still plugged in.

## Building from source

```bash
git clone <this repo>
cd scrcpy-gui/apps/desktop
pnpm install
pnpm tauri dev      # run in development (downloads scrcpy on first run)
pnpm tauri build    # installers land in target/release/bundle/
```

Prerequisites (Rust 1.77+, Node.js 20+, pnpm, system libraries) are listed in [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

## Documentation

- [Installation](docs/INSTALL.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Development](docs/DEVELOPMENT.md)
- [Third-party software](THIRD_PARTY.md)
- [Contributing](CONTRIBUTING.md)
- [Changelog](CHANGELOG.md)

## License

Apache License 2.0, see [LICENSE](LICENSE). The bundled scrcpy is developed by Genymobile (Apache 2.0); see [THIRD_PARTY.md](THIRD_PARTY.md).
