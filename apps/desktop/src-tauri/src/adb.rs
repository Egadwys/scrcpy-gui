//! Thin async wrapper around the `adb` command line.

use crate::{
    error::{AppError, Result},
    settings::Settings,
    tools,
};
use chrono::Local;
use serde::Serialize;
use std::{
    path::{Path, PathBuf},
    process::{Output, Stdio},
    time::Duration,
};
use tokio::time::{sleep, timeout};

#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub serial: String,
    /// device | unauthorized | offline | no-permissions | ...
    pub state: String,
    pub model: Option<String>,
    pub product: Option<String>,
    pub is_wifi: bool,
}

#[derive(Serialize, Clone, PartialEq, Debug, Default)]
pub struct DeviceSnapshot {
    pub devices: Vec<Device>,
    pub error: Option<String>,
}

async fn run_raw(adb: &Path, args: &[&str]) -> Result<Output> {
    let mut cmd = tools::command(adb);
    cmd.args(args).stdin(Stdio::null()).kill_on_drop(true);
    match timeout(Duration::from_secs(20), cmd.output()).await {
        Ok(out) => Ok(out?),
        Err(_) => Err(AppError::msg(format!("adb {} timed out", args.join(" ")))),
    }
}

/// Runs adb and returns stdout, or an error carrying adb's message.
async fn run(adb: &Path, args: &[&str]) -> Result<String> {
    let out = run_raw(adb, args).await?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    if out.status.success() {
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let msg = if stderr.trim().is_empty() { stdout } else { stderr.into_owned() };
        Err(AppError::msg(msg.trim().to_string()))
    }
}

pub fn parse_devices(text: &str) -> Vec<Device> {
    text.lines()
        .skip_while(|l| !l.starts_with("List of devices"))
        .skip(1)
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('*') {
                return None;
            }
            let (serial, rest) = line.split_once(char::is_whitespace)?;
            let rest = rest.trim_start();
            let state = if rest.starts_with("no permissions") {
                "no-permissions".to_string()
            } else {
                rest.split_whitespace().next()?.to_string()
            };
            let mut model = None;
            let mut product = None;
            for token in rest.split_whitespace() {
                if let Some(v) = token.strip_prefix("model:") {
                    model = Some(v.replace('_', " "));
                } else if let Some(v) = token.strip_prefix("product:") {
                    product = Some(v.to_string());
                }
            }
            let is_wifi = serial.contains(':') || serial.contains("_adb-tls");
            Some(Device {
                serial: serial.to_string(),
                state,
                model,
                product,
                is_wifi,
            })
        })
        .collect()
}

pub async fn snapshot(settings: &Settings) -> DeviceSnapshot {
    let adb = match tools::resolve_adb(settings) {
        Ok(p) => p,
        Err(e) => {
            return DeviceSnapshot {
                devices: vec![],
                error: Some(e.to_string()),
            }
        }
    };
    match run(&adb, &["devices", "-l"]).await {
        Ok(text) => DeviceSnapshot {
            devices: parse_devices(&text),
            error: None,
        },
        Err(e) => DeviceSnapshot {
            devices: vec![],
            error: Some(e.to_string()),
        },
    }
}

/// Extracts the first IPv4 address from `ip -f inet addr show wlan0` output.
fn parse_inet(text: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let mut it = line.split_whitespace();
        while let Some(tok) = it.next() {
            if tok == "inet" {
                return it.next().and_then(|a| a.split('/').next()).map(str::to_string);
            }
        }
        None
    })
}

/// Fallback: reads `src <ip>` from `ip route`, preferring wlan interfaces.
fn parse_route_src(text: &str) -> Option<String> {
    let pick = |only_wlan: bool| {
        text.lines()
            .filter(|l| !only_wlan || l.contains("wlan"))
            .find_map(|line| {
                let mut it = line.split_whitespace();
                while let Some(tok) = it.next() {
                    if tok == "src" {
                        return it.next().map(str::to_string);
                    }
                }
                None
            })
    };
    pick(true).or_else(|| pick(false))
}

pub async fn device_ip(adb: &Path, serial: &str) -> Result<String> {
    if let Ok(out) = run(adb, &["-s", serial, "shell", "ip", "-f", "inet", "addr", "show", "wlan0"]).await {
        if let Some(ip) = parse_inet(&out) {
            return Ok(ip);
        }
    }
    if let Ok(out) = run(adb, &["-s", serial, "shell", "ip", "route"]).await {
        if let Some(ip) = parse_route_src(&out) {
            return Ok(ip);
        }
    }
    Err(AppError::msg(
        "Could not read the device's Wi-Fi IP address. Make sure it is connected to the same Wi-Fi network as this computer.",
    ))
}

/// Switches a USB device to TCP/IP mode and connects to it. Returns `ip:port`.
pub async fn enable_wifi(adb: &Path, serial: &str, port: u16) -> Result<String> {
    let ip = device_ip(adb, serial).await?;
    run(adb, &["-s", serial, "tcpip", &port.to_string()]).await?;
    sleep(Duration::from_millis(1500)).await;
    let addr = format!("{ip}:{port}");
    connect(adb, &addr).await?;
    Ok(addr)
}

pub async fn connect(adb: &Path, addr: &str) -> Result<()> {
    // `adb connect` exits 0 even when it fails, so the output has to be inspected.
    let out = run_raw(adb, &["connect", addr]).await?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let lower = text.to_lowercase();
    if lower.contains("connected to") {
        Ok(())
    } else {
        Err(AppError::msg(text.trim().to_string()))
    }
}

pub async fn disconnect(adb: &Path, addr: &str) -> Result<()> {
    run(adb, &["disconnect", addr]).await.map(|_| ())
}

/// Wireless debugging pairing (Android 11+).
pub async fn pair(adb: &Path, addr: &str, code: &str) -> Result<()> {
    let out = run_raw(adb, &["pair", addr, code]).await?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if text.to_lowercase().contains("successfully paired") {
        Ok(())
    } else {
        Err(AppError::msg(text.trim().to_string()))
    }
}

/// Saves a PNG screenshot into `dir` and returns its path.
pub async fn screenshot(adb: &Path, serial: &str, dir: &Path) -> Result<PathBuf> {
    let out = run_raw(adb, &["-s", serial, "exec-out", "screencap", "-p"]).await?;
    if !out.status.success() || out.stdout.is_empty() {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(AppError::msg(if err.is_empty() {
            "Screenshot failed.".to_string()
        } else {
            err
        }));
    }
    std::fs::create_dir_all(dir)?;
    let file = dir.join(format!(
        "screenshot-{}-{}.png",
        tools::sanitize(serial),
        Local::now().format("%Y%m%d-%H%M%S")
    ));
    std::fs::write(&file, &out.stdout)?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_device_list() {
        let text = "* daemon not running; starting now at tcp:5037\n* daemon started successfully\nList of devices attached\nR58M12345 device usb:1-1 product:beyond1 model:SM_G973F device:beyond1 transport_id:1\n192.168.1.5:5555 device product:x model:Pixel_7 device:y transport_id:2\nABC unauthorized usb:1-2 transport_id:3\nXYZ no permissions (user in plugdev group; are you root?) usb:1-3\n\n";
        let d = parse_devices(text);
        assert_eq!(d.len(), 4);
        assert_eq!(d[0].serial, "R58M12345");
        assert_eq!(d[0].state, "device");
        assert_eq!(d[0].model.as_deref(), Some("SM G973F"));
        assert!(!d[0].is_wifi);
        assert!(d[1].is_wifi);
        assert_eq!(d[2].state, "unauthorized");
        assert_eq!(d[3].state, "no-permissions");
    }

    #[test]
    fn parses_ip() {
        let out = "5: wlan0: <BROADCAST> mtu 1500\n    inet 192.168.1.23/24 brd 192.168.1.255 scope global wlan0\n";
        assert_eq!(parse_inet(out).as_deref(), Some("192.168.1.23"));
        let route = "10.0.0.0/8 dev rmnet0 src 10.1.1.1\n192.168.1.0/24 dev wlan0 proto kernel scope link src 192.168.1.23\n";
        assert_eq!(parse_route_src(route).as_deref(), Some("192.168.1.23"));
    }
}
