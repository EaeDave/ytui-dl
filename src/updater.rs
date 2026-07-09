//! Update check and self-update against GitHub Releases (Linux + Windows).

use std::env;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use color_eyre::eyre::{bail, eyre, Result, WrapErr};
use sha2::{Digest, Sha256};
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::time::timeout;

use crate::action::Action;

const REPO: &str = "EaeDave/ytui-dl";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const USER_AGENT: &str = "ytui-dl-update";

/// Binary name on the current OS (`ytui-dl` / `ytui-dl.exe`).
pub fn binary_name() -> &'static str {
    if cfg!(windows) {
        "ytui-dl.exe"
    } else {
        "ytui-dl"
    }
}

/// Spawn a background task that reports a newer release tag, if any.
pub fn spawn_check(tx: mpsc::UnboundedSender<Action>) {
    tokio::spawn(async move {
        match timeout(Duration::from_secs(8), fetch_latest_tag()).await {
            Ok(Ok(Some(tag))) => {
                let remote = tag.trim_start_matches('v');
                if version_gt(remote, CURRENT_VERSION) {
                    let _ = tx.send(Action::UpdateAvailable {
                        version: remote.to_string(),
                    });
                }
            }
            _ => {}
        }
    });
}

/// Background install for the TUI (progress + result via actions).
pub fn spawn_tui_update(tx: mpsc::UnboundedSender<Action>) {
    tokio::spawn(async move {
        let report = {
            let tx = tx.clone();
            move |msg: String| {
                let _ = tx.send(Action::UpdateProgress { message: msg });
            }
        };

        match run_self_update_inner(false, report).await {
            Ok(outcome) => {
                let _ = tx.send(Action::UpdateSucceeded {
                    version: outcome.version,
                    install_path: outcome.install_path,
                });
            }
            Err(e) => {
                let _ = tx.send(Action::UpdateFailed {
                    error: format!("{e:#}"),
                });
            }
        }
    });
}

struct UpdateOutcome {
    version: String,
    installed: bool,
    /// Absolute path where the binary was installed (for restart after self-replace).
    install_path: Option<PathBuf>,
}

/// CLI entry: `ytui-dl --update`
pub async fn run_self_update(force: bool) -> Result<()> {
    let outcome = run_self_update_inner(force, |msg| println!("==> {msg}")).await?;
    if outcome.installed {
        println!("==> updated to v{}", outcome.version);
        println!("==> run: {} --version", binary_name());
    }
    Ok(())
}

/// Installs the latest release when needed. Returns version + whether files changed.
async fn run_self_update_inner(
    force: bool,
    mut progress: impl FnMut(String),
) -> Result<UpdateOutcome> {
    if which::which("curl").is_err() {
        bail!("curl is required for updates (install curl and retry)");
    }

    progress(format!("current version: {CURRENT_VERSION}"));
    progress("checking GitHub releases…".into());

    let tag = fetch_latest_tag()
        .await
        .map_err(|_| eyre!("could not resolve latest release (network / GitHub?)"))?
        .ok_or_else(|| eyre!("no release tag found"))?;
    let remote = tag.trim_start_matches('v').to_string();
    progress(format!("latest release: {remote}"));

    if !force {
        if version_gt(CURRENT_VERSION, &remote) {
            bail!("local version is newer than latest release; use --force to overwrite");
        }
        if !version_gt(&remote, CURRENT_VERSION) {
            progress(format!("already up to date (v{CURRENT_VERSION})"));
            return Ok(UpdateOutcome {
                version: CURRENT_VERSION.to_string(),
                installed: false,
                install_path: resolve_restart_path().ok(),
            });
        }
    }

    let target = detect_target()?;
    let asset = release_asset_name(&target);
    let url = format!("https://github.com/{REPO}/releases/download/{tag}/{asset}");
    let dest = install_destination()?;

    progress(format!("downloading {asset}"));
    let tmp_dir = env::temp_dir().join(format!("ytui-dl-update-{}", std::process::id()));
    tokio::fs::create_dir_all(&tmp_dir)
        .await
        .wrap_err("create temp dir")?;
    let tmp_bin = tmp_dir.join(binary_name());

    download_file(&url, &tmp_bin).await?;

    let sum_url = format!("{url}.sha256");
    let sum_path = tmp_dir.join(format!("{asset}.sha256"));
    if download_file(&sum_url, &sum_path).await.is_ok() {
        progress("verifying SHA256…".into());
        verify_sha256(&tmp_bin, &sum_path).await?;
    } else {
        progress("no checksum asset; skipping SHA256".into());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&tmp_bin).await?.permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&tmp_bin, perms).await?;
    }

    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .wrap_err_with(|| format!("create {}", parent.display()))?;
    }

    progress(format!("installing to {}", dest.display()));
    install_binary(&tmp_bin, &dest).await?;

    let _ = tokio::fs::remove_dir_all(&tmp_dir).await;
    Ok(UpdateOutcome {
        version: remote,
        installed: true,
        install_path: Some(dest),
    })
}

async fn install_binary(src: &Path, dest: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        // Windows: rename running .exe away, then put the new one in place.
        let dest_old = dest.with_extension("exe.old");
        let dest_tmp = dest.with_extension("exe.new");
        let _ = tokio::fs::remove_file(&dest_tmp).await;
        let _ = tokio::fs::remove_file(&dest_old).await;

        tokio::fs::copy(src, &dest_tmp)
            .await
            .wrap_err("copy binary into place")?;

        if dest.exists() {
            // Renaming a running executable is usually allowed on Windows.
            tokio::fs::rename(dest, &dest_old)
                .await
                .wrap_err_with(|| {
                    format!(
                        "could not replace running binary at {} (close ytui-dl and retry, or run: ytui-dl --update from a new shell)",
                        dest.display()
                    )
                })?;
        }

        tokio::fs::rename(&dest_tmp, dest)
            .await
            .or_else(|_| {
                std::fs::copy(src, dest).map(|_| ()).and_then(|_| {
                    let _ = std::fs::remove_file(&dest_tmp);
                    Ok(())
                })
            })
            .wrap_err_with(|| format!("install to {}", dest.display()))?;

        // Best-effort cleanup of previous binary
        let _ = tokio::fs::remove_file(&dest_old).await;
        return Ok(());
    }

    #[cfg(not(windows))]
    {
        // Unix: write sibling then rename (Linux can replace a running binary).
        let dest_tmp = dest.with_extension("new");
        tokio::fs::copy(src, &dest_tmp)
            .await
            .wrap_err("copy binary into place")?;
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = tokio::fs::metadata(&dest_tmp).await?.permissions();
            perms.set_mode(0o755);
            tokio::fs::set_permissions(&dest_tmp, perms).await?;
        }
        tokio::fs::rename(&dest_tmp, dest)
            .await
            .or_else(|_| {
                std::fs::copy(src, dest).map(|_| ()).and_then(|_| {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = std::fs::metadata(dest)?.permissions();
                    perms.set_mode(0o755);
                    std::fs::set_permissions(dest, perms)?;
                    let _ = std::fs::remove_file(&dest_tmp);
                    Ok(())
                })
            })
            .wrap_err_with(|| format!("install to {}", dest.display()))?;
        Ok(())
    }
}

/// CLI entry: `ytui-dl --uninstall`
///
/// Removes the installed binary (not config or downloads).
pub fn run_uninstall() -> Result<()> {
    let mut removed = Vec::new();
    let mut errors = Vec::new();

    for path in uninstall_candidates() {
        if !path.exists() {
            continue;
        }
        match std::fs::remove_file(&path) {
            Ok(()) => {
                println!("==> removed {}", path.display());
                removed.push(path);
            }
            Err(e) => {
                eprintln!("!!  could not remove {}: {e}", path.display());
                errors.push((path, e));
            }
        }
    }

    // Windows leftover from update
    #[cfg(windows)]
    {
        let old = default_user_bin().join("ytui-dl.exe.old");
        let _ = std::fs::remove_file(old);
    }

    if removed.is_empty() && errors.is_empty() {
        println!("==> ytui-dl binary not found in common install locations");
        println!(
            "    checked: {}, current executable, PATH",
            default_user_bin().join(binary_name()).display()
        );
        return Ok(());
    }

    if !removed.is_empty() {
        println!("==> uninstalled binary ({} file(s))", removed.len());
        println!("==> config and downloads were not removed");
        #[cfg(windows)]
        println!("    config: %LOCALAPPDATA%\\ytui-dl  (or %APPDATA%\\ytui-dl)");
        #[cfg(not(windows))]
        println!("    config: ~/.config/ytui-dl");
    }

    if !errors.is_empty() {
        bail!(
            "failed to remove {} path(s); close ytui-dl and retry{}",
            errors.len(),
            if cfg!(windows) {
                ""
            } else {
                ", or use sudo for system-wide installs"
            }
        );
    }

    Ok(())
}

fn uninstall_candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(exe) = env::current_exe() {
        let exe = strip_deleted_marker(exe);
        let exe = if exe.is_file() {
            exe.canonicalize().unwrap_or(exe)
        } else {
            exe
        };
        let in_target = path_in_target(&exe);
        if !in_target {
            paths.push(if cfg!(windows) {
                ensure_exe_name(exe)
            } else {
                exe
            });
        }
    }

    let local = default_user_bin().join(binary_name());
    if !paths.iter().any(|p| p == &local) {
        paths.push(local);
    }

    for name in [binary_name(), "ytui-dl"] {
        if let Ok(from_path) = which::which(name) {
            let from_path = from_path.canonicalize().unwrap_or(from_path);
            if !path_in_target(&from_path) && !paths.iter().any(|p| p == &from_path) {
                paths.push(from_path);
            }
        }
    }

    paths
}

/// Re-exec / relaunch the updated binary.
///
/// Prefer an explicit install path (from a successful update). Never trust a
/// `current_exe()` that points at a deleted inode (`… (deleted)` on Linux).
pub fn reexec_self(preferred: Option<PathBuf>) -> Result<()> {
    let exe = preferred
        .filter(|p| p.is_file())
        .or_else(|| resolve_restart_path().ok())
        .ok_or_else(|| eyre!("could not find ytui-dl binary to restart; run: {}", binary_name()))?;

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = std::process::Command::new(&exe).exec();
        bail!("failed to restart {}: {err}", exe.display());
    }

    #[cfg(windows)]
    {
        // Cannot exec() on Windows — spawn a new process and exit cleanly.
        std::process::Command::new(&exe)
            .spawn()
            .wrap_err_with(|| format!("failed to launch {}", exe.display()))?;
        std::process::exit(0);
    }

    #[cfg(not(any(unix, windows)))]
    {
        bail!(
            "automatic restart is not supported on this platform; relaunch {}",
            binary_name()
        );
    }
}

/// Path that exists on disk and should be used after self-update.
pub fn resolve_restart_path() -> Result<PathBuf> {
    let local = default_user_bin().join(binary_name());
    if local.is_file() {
        return Ok(local);
    }

    for name in [binary_name(), "ytui-dl"] {
        if let Ok(from_path) = which::which(name) {
            if from_path.is_file() {
                return Ok(from_path);
            }
        }
    }

    if let Ok(exe) = env::current_exe() {
        let cleaned = strip_deleted_marker(exe);
        if cleaned.is_file() {
            return Ok(cleaned);
        }
        if let Some(name) = cleaned.file_name().and_then(|n| n.to_str()) {
            if name.starts_with("ytui-dl") {
                if let Some(parent) = cleaned.parent() {
                    let candidate = parent.join(binary_name());
                    if candidate.is_file() {
                        return Ok(candidate);
                    }
                }
            }
        }
    }

    bail!("ytui-dl binary not found for restart")
}

/// Where to write the binary during update.
fn install_destination() -> Result<PathBuf> {
    let local = default_user_bin().join(binary_name());

    if let Ok(exe) = env::current_exe() {
        let cleaned = strip_deleted_marker(exe);
        let exe = if cleaned.is_file() {
            cleaned.canonicalize().unwrap_or(cleaned)
        } else {
            cleaned
        };
        let in_target = path_in_target(&exe);
        if in_target {
            return Ok(local);
        }

        let name = exe
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .replace(" (deleted)", "");

        if name.starts_with("ytui-dl") {
            if let Some(parent) = exe.parent() {
                return Ok(parent.join(binary_name()));
            }
            return Ok(ensure_exe_name(exe));
        }
    }
    Ok(local)
}

fn path_in_target(path: &Path) -> bool {
    path.components().any(|c| c.as_os_str() == "target")
}

fn ensure_exe_name(path: PathBuf) -> PathBuf {
    if cfg!(windows) {
        if path.extension().and_then(|e| e.to_str()) == Some("exe") {
            path
        } else if let Some(parent) = path.parent() {
            parent.join(binary_name())
        } else {
            PathBuf::from(binary_name())
        }
    } else {
        path
    }
}

/// Linux marks replaced-in-place executables as `path (deleted)` via /proc/self/exe.
fn strip_deleted_marker(path: PathBuf) -> PathBuf {
    let s = path.to_string_lossy();
    if let Some(clean) = s.strip_suffix(" (deleted)") {
        PathBuf::from(clean)
    } else {
        path
    }
}

/// User-writable install directory for the binary.
///
/// - Linux/macOS: `~/.local/bin`
/// - Windows: `%LOCALAPPDATA%\ytui-dl\bin`
pub fn default_user_bin() -> PathBuf {
    if cfg!(windows) {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ytui-dl")
            .join("bin")
    } else {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".local")
            .join("bin")
    }
}

fn detect_target() -> Result<String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let arch = match arch {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        other => bail!("unsupported architecture for prebuilt releases: {other}"),
    };
    match os {
        "linux" => Ok(format!("{arch}-unknown-linux-gnu")),
        "windows" => {
            if arch != "x86_64" {
                bail!("Windows prebuilt releases are currently x86_64 only");
            }
            Ok("x86_64-pc-windows-msvc".into())
        }
        "macos" => bail!(
            "macOS prebuilt releases are not published yet; use: cargo install --git https://github.com/{REPO}"
        ),
        other => bail!("unsupported OS for prebuilt releases: {other}"),
    }
}

fn release_asset_name(target: &str) -> String {
    if target.contains("windows") {
        format!("ytui-dl-{target}.exe")
    } else {
        format!("ytui-dl-{target}")
    }
}

async fn fetch_latest_tag() -> Result<Option<String>, ()> {
    let output = Command::new("curl")
        .args([
            "-fsSLI",
            "-o",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
            "-w",
            "%{url_effective}",
            "-A",
            USER_AGENT,
            &format!("https://github.com/{REPO}/releases/latest"),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
        .map_err(|_| ())?;

    if !output.status.success() {
        return Err(());
    }

    let url = String::from_utf8_lossy(&output.stdout);
    let tag = url.trim().rsplit('/').next().unwrap_or("").trim();
    if tag.is_empty() || tag == "latest" {
        return Ok(None);
    }
    Ok(Some(tag.to_string()))
}

/// Download with retries — GitHub release CDN occasionally returns 5xx/timeouts.
async fn download_file(url: &str, dest: &Path) -> Result<()> {
    const ATTEMPTS: u32 = 4;
    let mut last_err = String::new();

    for attempt in 1..=ATTEMPTS {
        let status = Command::new("curl")
            .args([
                "-fsSL",
                "--connect-timeout",
                "15",
                "--max-time",
                "300",
                "--retry",
                "3",
                "--retry-delay",
                "1",
                "--retry-all-errors",
                "-A",
                USER_AGENT,
                "-o",
            ])
            .arg(dest)
            .arg(url)
            .status()
            .await
            .wrap_err("run curl")?;

        if status.success() {
            let meta = tokio::fs::metadata(dest).await.wrap_err("stat download")?;
            if meta.len() > 0 {
                return Ok(());
            }
            last_err = "downloaded empty file".into();
        } else {
            last_err = status
                .code()
                .map(|c| format!("curl exit code {c}"))
                .unwrap_or_else(|| "curl failed".into());
        }

        if attempt < ATTEMPTS {
            let wait = attempt * 2;
            tokio::time::sleep(Duration::from_secs(wait.into())).await;
        }
    }

    bail!("download failed after {ATTEMPTS} attempts ({last_err}): {url}");
}

async fn verify_sha256(bin: &Path, sum_file: &Path) -> Result<()> {
    let expected = tokio::fs::read_to_string(sum_file)
        .await
        .wrap_err("read checksum file")?;
    let expected = expected
        .split_whitespace()
        .next()
        .ok_or_else(|| eyre!("empty checksum file"))?
        .to_lowercase();

    let mut file = tokio::fs::File::open(bin).await.wrap_err("open binary")?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf).await.wrap_err("read binary")?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let actual = format!("{:x}", hasher.finalize());
    if actual != expected {
        bail!("SHA256 mismatch (expected {expected}, got {actual})");
    }
    Ok(())
}

pub fn version_gt(a: &str, b: &str) -> bool {
    parse_version(a) > parse_version(b)
}

fn parse_version(s: &str) -> (u64, u64, u64) {
    let s = s.trim().trim_start_matches('v');
    let mut parts = s.split('.');
    let major = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let patch = parts
        .next()
        .and_then(|p| {
            p.split(|c: char| !c.is_ascii_digit())
                .next()
                .and_then(|n| n.parse().ok())
        })
        .unwrap_or(0);
    (major, minor, patch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions() {
        assert!(version_gt("0.2.0", "0.1.0"));
        assert!(version_gt("1.0.0", "0.9.9"));
        assert!(!version_gt("0.1.0", "0.1.0"));
        assert!(!version_gt("0.1.0", "0.2.0"));
        assert!(version_gt("0.1.1", "0.1.0"));
    }

    #[test]
    fn asset_names() {
        assert_eq!(
            release_asset_name("x86_64-unknown-linux-gnu"),
            "ytui-dl-x86_64-unknown-linux-gnu"
        );
        assert_eq!(
            release_asset_name("x86_64-pc-windows-msvc"),
            "ytui-dl-x86_64-pc-windows-msvc.exe"
        );
    }
}
