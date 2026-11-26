# Troubleshooting

**"scrcpy was not found" / "adb was not found"**
A normal installer includes both. If you run from source without `pnpm fetch-scrcpy` (or offline), the app falls back to downloading scrcpy at startup, or to `PATH`. Install scrcpy yourself ([INSTALL.md](INSTALL.md)) or set the paths in Preferences → Tools.

**"Last check failed" in Preferences → Tools**
The update check needs https://api.github.com and github.com. Behind a proxy or firewall it fails and the app keeps the current copy; this is harmless. GitHub also rate-limits anonymous API calls (60 per hour per IP), so many restarts in a row on a shared network can hit it. Try again later or press *Check now*.

**"Checksum mismatch: the download was discarded"**
The file was corrupted or altered on the way (captive portal, proxy). Nothing was installed. Retry on another network.

**The update installed but I still see the old version**
If a session was running, the new version is activated when the last session ends. Stop mirroring and check Preferences → Tools.

**Devices disappear after an update**
A new scrcpy brings a new adb. If another program keeps an adb server of a different version running, the two restart each other's server. Close the other tool, or run `adb kill-server` once.

**Device shows `unauthorized`**
Unlock the phone and accept *Allow USB debugging?*. If the prompt never appears: unplug, Developer options → *Revoke USB debugging authorizations*, plug in again.

**Device shows `offline`**
Replug the cable. If it persists run `adb kill-server` in a terminal; the app restarts the server automatically on the next poll.

**Device shows `no-permissions` (Linux)**
Your user cannot access the USB device. Install `android-udev-rules` (or the udev rules from your distribution), add yourself to the `plugdev` group, then replug.

**scrcpy window opens and closes immediately**
Open the Log panel at the bottom; scrcpy prints the reason. Common causes: scrcpy older than 3.0 with an option it does not know (see *Extra arguments* in Preferences), a device that refuses the requested codec (switch back to H.264), or an unsupported max size.

**Mouse and keyboard do not control the phone**
Some manufacturers (Xiaomi/HyperOS and others) need **USB debugging (Security settings)** enabled in Developer options, and a SIM inserted on some models.

**"Go wireless" says it cannot read the IP**
The phone must be on the same Wi-Fi network as the computer. Make sure Wi-Fi is on and not just mobile data.

**Wi-Fi connection drops after a while**
Some routers isolate clients (AP / client isolation); disable it or use a different network. After a reboot the phone leaves TCP/IP mode, so use **Go wireless** again while it is plugged in.

**Recording file is empty or cannot be opened**
Stop sessions from the app (Stop button, or Quit from the tray) so scrcpy can finalize the file. Recordings are `.mkv` because that format survives abrupt termination better than `.mp4`.
