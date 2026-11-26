# Installation

## 1. Install Scrcpy GUI

Run the installer for your system (from the releases page, or build it yourself: [DEVELOPMENT.md](DEVELOPMENT.md)).
scrcpy and adb are included, so there is nothing else to install.

On Linux the tray icon needs `libayatana-appindicator3` and a panel that shows tray icons (KDE, Xfce, Cinnamon, or GNOME with the AppIndicator extension). Without a tray, closing the window simply quits and stops your sessions.

## 2. Prepare the phone

1. Settings → About phone → tap *Build number* seven times.
2. Settings → Developer options → enable **USB debugging**.
3. Connect the cable and accept *Allow USB debugging?* (tick *Always allow*).

For Wi-Fi without a cable on Android 11+: Developer options → **Wireless debugging** → *Pair device with pairing code*, then use **Wireless → Pair** in the app.

## How scrcpy updates work

- On every start the app asks GitHub for the latest scrcpy release. If it is newer than the copy in use, it downloads the archive for your platform, checks its SHA-256 against the `SHA256SUMS.txt` published in that release, unpacks it into your user data folder and starts using it. The previous copy stays until it is no longer needed, so a failed update never breaks a working setup.
- If you are mirroring when an update finishes, it is activated when the last session ends (switching adb in the middle of a session would drop it).
- No internet, or GitHub unreachable: the app silently keeps using the copy it has.
- Turn it off or check manually in **Preferences → Tools**.
- Downloaded copies live in `%LOCALAPPDATA%\dev.scrcpygui.app\scrcpy` (Windows), `~/.local/share/dev.scrcpygui.app/scrcpy` (Linux) or `~/Library/Application Support/dev.scrcpygui.app/scrcpy` (macOS).

## Using your own scrcpy / adb

Set the path in **Preferences → Tools** (file or folder). A custom scrcpy path replaces the built-in copy and disables automatic updates. If only adb is set, scrcpy keeps updating by itself but uses your adb.

> The app always hands scrcpy the same adb it uses itself (`ADB` environment variable) and prefers the adb shipped next to the scrcpy in use. Two different adb versions fighting over one server is the classic cause of devices randomly disappearing: if another tool (Android Studio, a terminal) uses an older adb, the first one to start may restart the server of the other. Closing and reopening that tool fixes it.

Supported platforms for the built-in copy and auto-update: Windows x64, Linux x86_64, macOS (Intel and Apple Silicon), which are the platforms scrcpy publishes official builds for. On anything else, install scrcpy yourself and set the path.
