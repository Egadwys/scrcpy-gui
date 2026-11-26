# Changelog

## Unreleased

- scrcpy and adb are bundled into the installer.
- Automatic scrcpy updates: checked at every start, downloaded from the official GitHub release, verified with SHA-256 and used automatically (activated after running sessions end).
- Single-device screen modelled on gnirehtet-gui; Log, Options and Wireless as overlays.
- scrcpy is always started with `--pause-on-exit=false`.

## 0.1.0

- First version: device detection, mirroring with per-launch options, multi-device sessions.
- Wi-Fi: Go wireless, manual connect/disconnect, Android 11+ pairing.
- Recording to `.mkv`, PNG screenshots, log panel.
- System tray, single instance, run at startup, light/dark theme.
