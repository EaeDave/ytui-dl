//! Non-blocking update check against GitHub Releases.

use std::process::Stdio;
use std::time::Duration;

use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::time::timeout;

use crate::action::Action;

const REPO: &str = "EaeDave/ytui-dl";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const USER_AGENT: &str = "ytui-dl-update-check";

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
            // strip pre-release suffix: 1.0.0-beta
            p.split(|c: char| !c.is_ascii_digit())
                .next()
                .and_then(|n| n.parse().ok())
        })
        .unwrap_or(0);
    (major, minor, patch)
}

pub fn install_command() -> &'static str {
    "curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash"
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
