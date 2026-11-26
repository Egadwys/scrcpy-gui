//! The scrcpy that ships with the app, plus automatic updates from the official GitHub releases.
//!
//! Layout on disk:
//!   <resource dir>/resources/scrcpy/            bundled copy (filled by scripts/fetch-scrcpy.mjs)
//!   <local data dir>/scrcpy/<tag>/              downloaded copies, one folder per release
//!
//! A new release is always unpacked into its own folder and only then selected, so a copy that is
//! in use is never modified. The newest complete copy wins.

use crate::{
    error::{AppError, Result},
    state::AppState,
    tools,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    cmp::Ordering,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering as AtomicOrdering},
        Mutex, RwLock,
    },
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager};

const LATEST_URL: &str = "https://api.github.com/repos/Genymobile/scrcpy/releases/latest";
/// Assets are only ever downloaded from the official repository.
const DOWNLOAD_PREFIX: &str = "https://github.com/Genymobile/scrcpy/releases/download/";
/// Downloaded versions kept on disk (the newest ones).
const KEEP_VERSIONS: usize = 2;

/// Name used by scrcpy releases for this platform: scrcpy-<id>-<tag>.<zip|tar.gz>
const ASSET_ID: Option<&str> = if cfg!(windows) {
    Some("win64")
} else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
    Some("linux-x86_64")
} else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
    Some("macos-aarch64")
} else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
    Some("macos-x86_64")
} else {
    None
};

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Bundled,
    Downloaded,
}

/// The scrcpy folder currently in use.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScrcpyHome {
    pub dir: PathBuf,
    pub tag: String,
    pub source: Source,
}

/// Progress of an update check, sent to the UI as `scrcpy-update`.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    /// checking | downloading | installing | updated | uptodate | error | unsupported
    pub phase: &'static str,
    pub version: Option<String>,
    /// 0..100 while downloading
    pub progress: Option<f32>,
    pub message: Option<String>,
    /// A new version is installed but waits for running sessions to end.
    pub pending: bool,
}

static CURRENT: RwLock<Option<ScrcpyHome>> = RwLock::new(None);
static STATUS: Mutex<Option<UpdateStatus>> = Mutex::new(None);
static BUSY: AtomicBool = AtomicBool::new(false);
static PENDING: AtomicBool = AtomicBool::new(false);

pub fn current() -> Option<ScrcpyHome> {
    CURRENT.read().unwrap().clone()
}

pub fn status_now() -> Option<UpdateStatus> {
    STATUS.lock().unwrap().clone()
}

fn publish(
    app: &AppHandle,
    phase: &'static str,
    version: Option<String>,
    progress: Option<f32>,
    message: Option<String>,
) {
    let status = UpdateStatus {
        phase,
        version,
        progress,
        message,
        pending: PENDING.load(AtomicOrdering::SeqCst),
    };
    *STATUS.lock().unwrap() = Some(status.clone());
    let _ = app.emit("scrcpy-update", status);
}

// ---------------------------------------------------------------- versions

/// "v4.1" / "3.3.4" -> [4, 1] / [3, 3, 4]. Anything else is rejected, which also keeps
/// strange tag names out of file paths.
pub fn parse_version(s: &str) -> Option<Vec<u32>> {
    let t = s.trim().trim_start_matches('v');
    if t.is_empty() {
        return None;
    }
    t.split('.').map(|p| p.parse::<u32>().ok()).collect()
}

pub fn cmp_versions(a: &[u32], b: &[u32]) -> Ordering {
    for i in 0..a.len().max(b.len()) {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        match x.cmp(&y) {
            Ordering::Equal => continue,
            other => return other,
        }
    }
    Ordering::Equal
}

// ------------------------------------------------------------- local copies

fn managed_root(app: &AppHandle) -> Result<PathBuf> {
    let base = app
        .path()
        .app_local_data_dir()
        .map_err(|e| AppError::msg(e.to_string()))?;
    Ok(base.join("scrcpy"))
}

fn exe_in(dir: &Path) -> PathBuf {
    dir.join(tools::exe("scrcpy"))
}

fn bundled_dir(app: &AppHandle) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(res) = app.path().resource_dir() {
        candidates.push(res.join("resources").join("scrcpy"));
    }
    // `tauri dev` runs straight from the source tree.
    #[cfg(debug_assertions)]
    candidates.push(Path::new(env!("CARGO_MANIFEST_DIR")).join("resources").join("scrcpy"));
    candidates.into_iter().find(|d| exe_in(d).is_file())
}

fn read_tag(dir: &Path) -> Option<String> {
    fs::read_to_string(dir.join("VERSION"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Complete downloaded copies, newest first.
fn managed_versions(root: &Path) -> Vec<(Vec<u32>, String, PathBuf)> {
    let Ok(rd) = fs::read_dir(root) else {
        return Vec::new();
    };
    let mut found: Vec<(Vec<u32>, String, PathBuf)> = rd
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let path = e.path();
            let name = e.file_name().to_string_lossy().into_owned();
            let ver = parse_version(&name)?;
            (path.join(".complete").is_file() && exe_in(&path).is_file()).then_some((ver, name, path))
        })
        .collect();
    found.sort_by(|a, b| cmp_versions(&b.0, &a.0));
    found
}

fn ensure_executable(dir: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for name in ["scrcpy", "adb"] {
            let p = dir.join(name);
            if let Ok(meta) = fs::metadata(&p) {
                let mut perm = meta.permissions();
                if perm.mode() & 0o111 == 0 {
                    perm.set_mode(perm.mode() | 0o755);
                    let _ = fs::set_permissions(&p, perm);
                }
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = dir;
    }
}

/// Re-scans the bundled and downloaded copies and makes the newest one current.
pub fn refresh_current(app: &AppHandle) {
    let mut best: Option<(Vec<u32>, ScrcpyHome)> = None;

    if let Some(dir) = bundled_dir(app) {
        let tag = read_tag(&dir).unwrap_or_else(|| "bundled".to_string());
        let ver = parse_version(&tag).unwrap_or_default();
        best = Some((
            ver,
            ScrcpyHome {
                dir,
                tag,
                source: Source::Bundled,
            },
        ));
    }

    if let Ok(root) = managed_root(app) {
        if let Some((ver, tag, dir)) = managed_versions(&root).into_iter().next() {
            let newer = match &best {
                Some((bv, _)) => cmp_versions(&ver, bv) != Ordering::Less,
                None => true,
            };
            if newer {
                best = Some((
                    ver,
                    ScrcpyHome {
                        dir,
                        tag,
                        source: Source::Downloaded,
                    },
                ));
            }
        }
    }

    if let Some((_, home)) = &best {
        ensure_executable(&home.dir);
    }
    *CURRENT.write().unwrap() = best.map(|(_, h)| h);
}

/// Activates an update that was installed while sessions were running.
pub fn apply_pending(app: &AppHandle) {
    if PENDING.swap(false, AtomicOrdering::SeqCst) {
        refresh_current(app);
        publish(app, "updated", current().map(|h| h.tag), None, None);
    }
}

fn cleanup(root: &Path) {
    if let Ok(rd) = fs::read_dir(root) {
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with(".tmp-") || name.starts_with(".dl-") {
                let p = entry.path();
                let _ = if p.is_dir() {
                    fs::remove_dir_all(&p)
                } else {
                    fs::remove_file(&p)
                };
            }
        }
    }
    for (_, _, dir) in managed_versions(root).into_iter().skip(KEEP_VERSIONS) {
        let _ = fs::remove_dir_all(dir);
    }
}

// ----------------------------------------------------------------- network

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

fn net(e: reqwest::Error) -> AppError {
    AppError::msg(format!("Network error: {e}"))
}

fn timed_out() -> AppError {
    AppError::msg("The request timed out.")
}

fn client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(concat!("scrcpy-gui/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(15))
        .build()
        .map_err(net)
}

async fn get_text(c: &reqwest::Client, url: &str, accept: Option<&str>) -> Result<String> {
    let mut req = c.get(url);
    if let Some(a) = accept {
        req = req.header("Accept", a);
    }
    let fut = async { req.send().await?.error_for_status()?.text().await };
    tokio::time::timeout(Duration::from_secs(30), fut)
        .await
        .map_err(|_| timed_out())?
        .map_err(net)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Finds `<sha256>  <file>` in a sha256sum style listing.
fn expected_hash(sums: &str, file: &str) -> Option<String> {
    sums.lines().find_map(|line| {
        let (hash, rest) = line.trim().split_once(char::is_whitespace)?;
        let name = rest.trim().trim_start_matches('*');
        (name == file && hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()))
            .then(|| hash.to_lowercase())
    })
}

/// Streams a file to disk, reporting progress, and returns its SHA-256.
async fn download(
    app: &AppHandle,
    c: &reqwest::Client,
    url: &str,
    dest: &Path,
    size_hint: u64,
    version: &str,
) -> Result<String> {
    let mut resp = tokio::time::timeout(Duration::from_secs(30), c.get(url).send())
        .await
        .map_err(|_| timed_out())?
        .map_err(net)?
        .error_for_status()
        .map_err(net)?;
    let total = resp.content_length().filter(|n| *n > 0).unwrap_or(size_hint).max(1);

    let mut file = fs::File::create(dest)?;
    let mut hasher = Sha256::new();
    let mut done: u64 = 0;
    let mut last_pct: i32 = -1;

    loop {
        let next = tokio::time::timeout(Duration::from_secs(60), resp.chunk())
            .await
            .map_err(|_| AppError::msg("The download stalled."))?
            .map_err(net)?;
        let Some(chunk) = next else { break };
        hasher.update(&chunk);
        file.write_all(&chunk)?;
        done += chunk.len() as u64;

        let pct = (((done as f64 / total as f64) * 100.0).min(100.0)) as i32;
        if pct != last_pct {
            last_pct = pct;
            publish(app, "downloading", Some(version.to_string()), Some(pct as f32), None);
        }
    }
    file.flush()?;
    Ok(hex(&hasher.finalize()))
}

// ---------------------------------------------------------------- install

fn extract(archive: &Path, dest: &Path, is_zip: bool) -> Result<()> {
    fs::create_dir_all(dest)?;
    let file = fs::File::open(archive)?;
    if is_zip {
        let mut z = zip::ZipArchive::new(file)
            .map_err(|e| AppError::msg(format!("The downloaded archive is not valid: {e}")))?;
        z.extract(dest)
            .map_err(|e| AppError::msg(format!("Could not extract the archive: {e}")))?;
    } else {
        tar::Archive::new(flate2::read::GzDecoder::new(file)).unpack(dest)?;
    }
    Ok(())
}

/// Unpacks `archive`, validates it and moves it to `final_dir`. Blocking.
fn install_archive(archive: &Path, tmp: &Path, final_dir: &Path, is_zip: bool) -> Result<()> {
    extract(archive, tmp, is_zip)?;

    // Release archives wrap everything in one top-level folder.
    let mut src = tmp.to_path_buf();
    let entries: Vec<_> = fs::read_dir(tmp)?.filter_map(|e| e.ok()).collect();
    if entries.len() == 1 && entries[0].path().is_dir() {
        src = entries[0].path();
    }
    if !exe_in(&src).is_file() {
        return Err(AppError::msg("The downloaded archive does not contain scrcpy."));
    }
    ensure_executable(&src);
    fs::write(src.join(".complete"), "ok")?;

    if final_dir.exists() {
        fs::remove_dir_all(final_dir)?;
    }
    fs::rename(&src, final_dir)?;
    let _ = fs::remove_dir_all(tmp);
    Ok(())
}

async fn run(app: &AppHandle) -> Result<()> {
    let Some(asset_id) = ASSET_ID else {
        publish(
            app,
            "unsupported",
            None,
            None,
            Some("Automatic scrcpy updates are not available for this platform.".into()),
        );
        return Ok(());
    };
    let is_zip = cfg!(windows);

    publish(app, "checking", None, None, None);
    let c = client()?;
    let body = get_text(&c, LATEST_URL, Some("application/vnd.github+json")).await?;
    let release: Release = serde_json::from_str(&body)?;
    let latest = parse_version(&release.tag_name)
        .ok_or_else(|| AppError::msg(format!("Unexpected release tag: {}", release.tag_name)))?;
    let tag = release.tag_name.clone();

    if let Some(cur) = current() {
        if let Some(cv) = parse_version(&cur.tag) {
            if cmp_versions(&cv, &latest) != Ordering::Less {
                publish(app, "uptodate", Some(cur.tag), None, None);
                return Ok(());
            }
        }
    }

    let wanted = format!("scrcpy-{asset_id}-{tag}.{}", if is_zip { "zip" } else { "tar.gz" });
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == wanted)
        .ok_or_else(|| AppError::msg(format!("Release {tag} has no {wanted}.")))?;
    let sums = release
        .assets
        .iter()
        .find(|a| a.name == "SHA256SUMS.txt")
        .ok_or_else(|| AppError::msg(format!("Release {tag} has no checksum file.")))?;
    for a in [asset, sums] {
        if !a.browser_download_url.starts_with(DOWNLOAD_PREFIX) {
            return Err(AppError::msg("Unexpected download location, update aborted."));
        }
    }

    let sums_text = get_text(&c, &sums.browser_download_url, None).await?;
    let expected = expected_hash(&sums_text, &wanted)
        .ok_or_else(|| AppError::msg(format!("No checksum is published for {wanted}.")))?;

    let root = managed_root(app)?;
    fs::create_dir_all(&root)?;
    cleanup(&root);
    let part = root.join(format!(".dl-{tag}.part"));
    let tmp = root.join(format!(".tmp-{tag}"));
    let final_dir = root.join(&tag);

    publish(app, "downloading", Some(tag.clone()), Some(0.0), None);
    let actual = download(app, &c, &asset.browser_download_url, &part, asset.size, &tag).await?;
    if actual != expected {
        let _ = fs::remove_file(&part);
        return Err(AppError::msg("Checksum mismatch: the download was discarded."));
    }

    publish(app, "installing", Some(tag.clone()), None, None);
    let (p, t, f) = (part.clone(), tmp, final_dir);
    tauri::async_runtime::spawn_blocking(move || install_archive(&p, &t, &f, is_zip))
        .await
        .map_err(|e| AppError::msg(e.to_string()))??;
    let _ = fs::remove_file(&part);
    cleanup(&root);

    // Switching adb under a live session would restart the adb server and drop it,
    // so with sessions running the switch waits until the last one ends.
    if app.state::<AppState>().has_sessions() {
        PENDING.store(true, AtomicOrdering::SeqCst);
        publish(app, "updated", Some(tag), None, None);
    } else {
        refresh_current(app);
        publish(app, "updated", Some(tag), None, None);
    }
    Ok(())
}

/// Checks GitHub for a newer scrcpy and installs it. Safe to call repeatedly: only one runs at a time.
pub async fn check_and_install(app: AppHandle) {
    if BUSY.swap(true, AtomicOrdering::SeqCst) {
        return;
    }
    if let Err(e) = run(&app).await {
        publish(&app, "error", None, None, Some(e.to_string()));
    }
    BUSY.store(false, AtomicOrdering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_compares_versions() {
        assert_eq!(parse_version("v4.1"), Some(vec![4, 1]));
        assert_eq!(parse_version("3.3.4"), Some(vec![3, 3, 4]));
        assert_eq!(parse_version(".tmp-v4.1"), None);
        assert_eq!(parse_version("../evil"), None);
        assert_eq!(cmp_versions(&[4, 1], &[3, 3, 4]), Ordering::Greater);
        assert_eq!(cmp_versions(&[4, 0], &[4]), Ordering::Equal);
        assert_eq!(cmp_versions(&[3, 3, 4], &[3, 10]), Ordering::Less);
    }

    #[test]
    fn finds_checksum_for_file() {
        let h = "a".repeat(64);
        let g = "b".repeat(64);
        let sums = format!("{h}  scrcpy-server-v4.1\n{g} *scrcpy-win64-v4.1.zip\n");
        assert_eq!(expected_hash(&sums, "scrcpy-win64-v4.1.zip"), Some(g));
        assert_eq!(expected_hash(&sums, "scrcpy-server-v4.1"), Some(h));
        assert_eq!(expected_hash(&sums, "missing"), None);
    }

    #[test]
    fn hex_encodes() {
        assert_eq!(hex(&[0x00, 0xab, 0xff]), "00abff");
    }
}
