//! Update check and self-update against GitHub Releases (Linux first).
//!
//! Windows support can reuse the same flow later with platform-specific
//! install paths and restart (`CreateProcess` + exit).

use std::env;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use color_eyre::eyre::{bail, eyre, Result, WrapErr};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::time::timeout;

use crate::action::Action;

const REPO: &str = "EaeDave/ytui-dl";
const BIN_NAME: &str = "ytui-dl";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const USER_AGENT: &str = "ytui-dl-update";

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
            Ok(outcome) if outcome.installed => {
                let _ = tx.send(Action::UpdateSucceeded {
                    version: outcome.version,
                });
            }
            Ok(outcome) => {
                let _ = tx.send(Action::UpdateSucceeded {
                    version: outcome.version,
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
}

/// CLI entry: `ytui-dl --update`
pub async fn run_self_update(force: bool) -> Result<()> {
    let outcome = run_self_update_inner(force, |msg| println!("==> {msg}")).await?;
    if outcome.installed {
        println!("==> updated to v{}", outcome.version);
        println!("==> run: ytui-dl --version");
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
            });
        }
    }

    let target = detect_target()?;
    let asset = format!("{BIN_NAME}-{target}");
    let url = format!("https://github.com/{REPO}/releases/download/{tag}/{asset}");
    let dest = install_destination()?;

    progress(format!("downloading {asset}"));
    let tmp_dir = env::temp_dir().join(format!("ytui-dl-update-{}", std::process::id()));
    tokio::fs::create_dir_all(&tmp_dir)
        .await
        .wrap_err("create temp dir")?;
    let tmp_bin = tmp_dir.join(BIN_NAME);

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
    // Atomic-ish replace: write sibling then rename (Linux can replace a running binary).
    let dest_tmp = dest.with_extension("new");
    tokio::fs::copy(&tmp_bin, &dest_tmp)
        .await
        .wrap_err("copy binary into place")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&dest_tmp).await?.permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&dest_tmp, perms).await?;
    }
    tokio::fs::rename(&dest_tmp, &dest)
        .await
        .or_else(|_| {
            std::fs::copy(&tmp_bin, &dest).map(|_| ()).and_then(|_| {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = std::fs::metadata(&dest)?.permissions();
                    perms.set_mode(0o755);
                    std::fs::set_permissions(&dest, perms)?;
                }
                let _ = std::fs::remove_file(&dest_tmp);
                Ok(())
            })
        })
        .wrap_err_with(|| format!("install to {}", dest.display()))?;

    let _ = tokio::fs::remove_dir_all(&tmp_dir).await;
    Ok(UpdateOutcome {
        version: remote,
        installed: true,
    })
}

/// CLI entry: `ytui-dl --uninstall`
///
/// Removes the installed binary (not config or downloads).
pub fn run_uninstall() -> Result<()> {
    let mut removed = Vec::new();
    let mut missing = Vec::new();
    let mut errors = Vec::new();

    for path in uninstall_candidates() {
        if !path.exists() {
            missing.push(path);
            continue;
        }
        match std::fs::remove_file(&path) {
            Ok(()) => {
                println!("==> removed {}", path.display());
                removed.push(path);
            }
            Err(e) => {
                // On Linux, deleting the running binary usually works (unlink).
                // Permission errors need sudo for system installs.
                eprintln!("!!  could not remove {}: {e}", path.display());
                errors.push((path, e));
            }
        }
    }

    if removed.is_empty() && errors.is_empty() {
        println!("==> ytui-dl binary not found in common install locations");
        println!("    checked: ~/.local/bin/ytui-dl, current executable, PATH");
        return Ok(());
    }

    if !removed.is_empty() {
        println!("==> uninstalled binary ({} file(s))", removed.len());
        println!("==> config (~/.config/ytui-dl) and downloads were not removed");
    }

    if !errors.is_empty() {
        bail!(
            "failed to remove {} path(s); try with sudo if installed system-wide",
            errors.len()
        );
    }

    let _ = missing;
    Ok(())
}

fn uninstall_candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(exe) = env::current_exe() {
        let exe = exe.canonicalize().unwrap_or(exe);
        let in_target = exe.components().any(|c| c.as_os_str() == "target");
        if !in_target {
            paths.push(exe);
        }
    }

    let local = default_user_bin().join(BIN_NAME);
    if !paths.iter().any(|p| p == &local) {
        paths.push(local);
    }

    if let Ok(from_path) = which::which(BIN_NAME) {
        let from_path = from_path.canonicalize().unwrap_or(from_path);
        if !paths.iter().any(|p| p == &from_path) {
            let in_target = from_path.components().any(|c| c.as_os_str() == "target");
            if !in_target {
                paths.push(from_path);
            }
        }
    }

    paths
}

/// Re-exec this process with the (possibly updated) binary. Linux/unix.
pub fn reexec_self() -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let exe = env::current_exe().wrap_err("resolve current executable")?;
        let err = std::process::Command::new(&exe).args(std::env::args().skip(1)).exec();
        bail!("failed to restart {}: {err}", exe.display());
    }
    #[cfg(not(unix))]
    {
        bail!("automatic restart is not supported on this platform yet; relaunch ytui-dl manually");
    }
}

fn install_destination() -> Result<PathBuf> {
    if let Ok(exe) = env::current_exe() {
        let exe = exe.canonicalize().unwrap_or(exe);
        let name = exe
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let in_target = exe.components().any(|c| c.as_os_str() == "target");
        if name.starts_with(BIN_NAME) && !in_target {
            return Ok(exe);
        }
        let local = default_user_bin().join(BIN_NAME);
        if in_target {
            return Ok(local);
        }
        if name.starts_with(BIN_NAME) {
            return Ok(exe);
        }
        return Ok(local);
    }
    Ok(default_user_bin().join(BIN_NAME))
}

fn default_user_bin() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local")
        .join("bin")
}

fn detect_target() -> Result<String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    // Focus: Linux now. Windows later (different asset triple + install dir).
    if os != "linux" {
        bail!("in-app update currently supports Linux only (detected {os})");
    }
    let arch = match arch {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        other => bail!("unsupported architecture for prebuilt releases: {other}"),
    };
    Ok(format!("{arch}-unknown-linux-gnu"))
}

async fn fetch_latest_tag() -> Result<Option<String>, ()> {
    let output = Command::new("curl")
        .args([
            "-fsSLI",
            "-o",
            "/dev/null",
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

async fn download_file(url: &str, dest: &Path) -> Result<()> {
    let status = Command::new("curl")
        .args(["-fsSL", "-A", USER_AGENT, "-o"])
        .arg(dest)
        .arg(url)
        .status()
        .await
        .wrap_err("run curl")?;
    if !status.success() {
        bail!("download failed: {url}");
    }
    Ok(())
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

    let output = Command::new("sha256sum")
        .arg(bin)
        .output()
        .await
        .wrap_err("sha256sum")?;
    if !output.status.success() {
        bail!("sha256sum failed");
    }
    let actual = String::from_utf8_lossy(&output.stdout);
    let actual = actual
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_lowercase();
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
}
