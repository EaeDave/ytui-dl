//! Update check and self-update against GitHub Releases.

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
            _ => {
                // Network / timeout / curl missing — silent; never block the UI.
            }
        }
    });
}

/// CLI entry: `ytui-dl --update`
///
/// Downloads the latest release binary and replaces the install target
/// (`~/.local/bin/ytui-dl` when possible, else the path of this executable).
pub async fn run_self_update(force: bool) -> Result<()> {
    if which::which("curl").is_err() {
        bail!("curl is required for --update (install curl and retry)");
    }

    println!("==> current version: {CURRENT_VERSION}");
    println!("==> checking GitHub releases…");

    let tag = fetch_latest_tag()
        .await
        .map_err(|_| eyre!("could not resolve latest release (network / GitHub?)"))?
        .ok_or_else(|| eyre!("no release tag found"))?;
    let remote = tag.trim_start_matches('v').to_string();
    println!("==> latest release:  {remote}");

    if !force && !version_gt(&remote, CURRENT_VERSION) {
        if remote == CURRENT_VERSION || !version_gt(CURRENT_VERSION, &remote) {
            println!("==> already up to date");
            return Ok(());
        }
        // Installed build is newer than published release (dev build)
        println!("==> local version is newer than the latest release; use --force to overwrite");
        return Ok(());
    }

    if !force && remote == CURRENT_VERSION {
        println!("==> already up to date");
        return Ok(());
    }

    let target = detect_target()?;
    let asset = format!("{BIN_NAME}-{target}");
    let url = format!("https://github.com/{REPO}/releases/download/{tag}/{asset}");
    let dest = install_destination()?;

    println!("==> downloading {asset}");
    let tmp_dir = env::temp_dir().join(format!("ytui-dl-update-{}", std::process::id()));
    tokio::fs::create_dir_all(&tmp_dir)
        .await
        .wrap_err("create temp dir")?;
    let tmp_bin = tmp_dir.join(BIN_NAME);

    download_file(&url, &tmp_bin).await?;

    // Optional checksum
    let sum_url = format!("{url}.sha256");
    let sum_path = tmp_dir.join(format!("{asset}.sha256"));
    if download_file(&sum_url, &sum_path).await.is_ok() {
        println!("==> verifying SHA256");
        verify_sha256(&tmp_bin, &sum_path).await?;
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

    println!("==> installing to {}", dest.display());
    // Replace atomically when possible: write sibling temp then rename.
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
            // Fallback if rename across devices fails
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

    println!("==> updated to v{remote}");
    println!("==> run: ytui-dl --version");
    Ok(())
}

fn install_destination() -> Result<PathBuf> {
    // Prefer the path of this executable when it looks like a real install.
    if let Ok(exe) = env::current_exe() {
        let exe = exe
            .canonicalize()
            .unwrap_or(exe);
        let name = exe
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        // Avoid writing over cargo/debug builds in the project tree when a user install exists.
        let in_target = exe.components().any(|c| c.as_os_str() == "target");
        if name.starts_with(BIN_NAME) && !in_target {
            return Ok(exe);
        }
        let local = default_user_bin().join(BIN_NAME);
        if local.exists() || !in_target {
            if in_target {
                return Ok(local);
            }
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
    if os != "linux" {
        bail!("--update currently supports Linux only (detected {os}); use the install script or cargo");
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

/// True if `a` > `b` for simple dotted versions (1.2.3).
pub fn version_gt(a: &str, b: &str) -> bool {
    let a = parse_version(a);
    let b = parse_version(b);
    a > b
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

/// Prefer telling users about --update when an update exists.
pub fn update_hint_command() -> &'static str {
    "ytui-dl --update"
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
