use regex::Regex;
use std::sync::OnceLock;

use crate::models::ProgressUpdate;

/// Custom yt-dlp progress template (one JSON-ish line per update via --newline).
///
/// Fields are delimited so the parser stays stable across yt-dlp versions.
pub const PROGRESS_TEMPLATE: &str =
    "progress|%(progress.downloaded_bytes)s|%(progress.total_bytes)s|%(progress.total_bytes_estimate)s|%(progress.speed)s|%(progress.eta)s|%(progress._percent_str)s";

fn progress_line_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^progress\|([^|]*)\|([^|]*)\|([^|]*)\|([^|]*)\|([^|]*)\|([^|]*)$")
            .expect("valid progress regex")
    })
}

/// Fallback for default yt-dlp progress lines, e.g.
/// `[download]  45.2% of  10.00MiB at  1.23MiB/s ETA 00:04`
fn default_progress_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\[download\]\s+(\d+(?:\.\d+)?)%.*?at\s+(\S+)\s+ETA\s+(\S+)",
        )
        .expect("valid default progress regex")
    })
}

/// Parse a single stdout/stderr line from yt-dlp into a progress update, if any.
pub fn parse_progress_line(line: &str) -> Option<ProgressUpdate> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    if let Some(caps) = progress_line_re().captures(line) {
        let downloaded = parse_u64(caps.get(1).map(|m| m.as_str()).unwrap_or(""));
        let total = parse_u64(caps.get(2).map(|m| m.as_str()).unwrap_or(""))
            .or_else(|| parse_u64(caps.get(3).map(|m| m.as_str()).unwrap_or("")));
        let speed_raw = caps.get(4).map(|m| m.as_str()).unwrap_or("").trim();
        let eta_raw = caps.get(5).map(|m| m.as_str()).unwrap_or("").trim();
        let percent_raw = caps.get(6).map(|m| m.as_str()).unwrap_or("").trim();

        let percent = parse_percent(percent_raw).or_else(|| {
            match (downloaded, total) {
                (Some(d), Some(t)) if t > 0 => Some((d as f64 / t as f64) * 100.0),
                _ => None,
            }
        });

        let speed = if is_na(speed_raw) {
            None
        } else {
            Some(format_speed(speed_raw))
        };

        let eta = if is_na(eta_raw) {
            None
        } else {
            Some(format_eta(eta_raw))
        };

        return Some(ProgressUpdate {
            percent,
            speed,
            eta,
            total_bytes: total,
            downloaded_bytes: downloaded,
        });
    }

    if let Some(caps) = default_progress_re().captures(line) {
        let percent = caps
            .get(1)
            .and_then(|m| m.as_str().parse::<f64>().ok());
        let speed = caps.get(2).map(|m| m.as_str().to_string());
        let eta = caps.get(3).map(|m| m.as_str().to_string());
        return Some(ProgressUpdate {
            percent,
            speed,
            eta,
            total_bytes: None,
            downloaded_bytes: None,
        });
    }

    None
}

fn parse_u64(s: &str) -> Option<u64> {
    let s = s.trim();
    if is_na(s) {
        return None;
    }
    // yt-dlp may emit floats for bytes in templates
    if let Ok(v) = s.parse::<u64>() {
        return Some(v);
    }
    s.parse::<f64>().ok().map(|f| f as u64)
}

fn parse_percent(s: &str) -> Option<f64> {
    let s = s.trim().trim_end_matches('%').trim();
    if is_na(s) {
        return None;
    }
    s.parse::<f64>().ok().map(|p| p.clamp(0.0, 100.0))
}

fn is_na(s: &str) -> bool {
    s.is_empty() || s.eq_ignore_ascii_case("NA") || s.eq_ignore_ascii_case("N/A")
}

fn format_speed(raw: &str) -> String {
    // If numeric bytes/s, humanize; otherwise pass through (already human).
    if let Ok(bps) = raw.parse::<f64>() {
        return human_bytes(bps) + "/s";
    }
    raw.to_string()
}

fn format_eta(raw: &str) -> String {
    if let Ok(secs) = raw.parse::<f64>() {
        let secs = secs.max(0.0) as u64;
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        if h > 0 {
            return format!("{h:02}:{m:02}:{s:02}");
        }
        return format!("{m:02}:{s:02}");
    }
    raw.to_string()
}

fn human_bytes(bytes: f64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut v = bytes.max(0.0);
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{v:.0} {}", UNITS[i])
    } else {
        format!("{v:.2} {}", UNITS[i])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_custom_template_line() {
        let line = "progress|5242880|10485760|NA|1048576|5|50.0%";
        let p = parse_progress_line(line).expect("parsed");
        assert!((p.percent.unwrap() - 50.0).abs() < f64::EPSILON);
        assert_eq!(p.downloaded_bytes, Some(5_242_880));
        assert_eq!(p.total_bytes, Some(10_485_760));
        assert!(p.speed.as_ref().unwrap().contains("MiB"));
        assert_eq!(p.eta.as_deref(), Some("00:05"));
    }

    #[test]
    fn parses_with_estimate_total() {
        let line = "progress|1000|NA|2000|500|2|NA";
        let p = parse_progress_line(line).expect("parsed");
        assert!((p.percent.unwrap() - 50.0).abs() < f64::EPSILON);
        assert_eq!(p.total_bytes, Some(2000));
    }

    #[test]
    fn parses_default_ytdlp_line() {
        let line = "[download]  45.2% of  10.00MiB at  1.23MiB/s ETA 00:04";
        let p = parse_progress_line(line).expect("parsed");
        assert!((p.percent.unwrap() - 45.2).abs() < 0.01);
        assert_eq!(p.speed.as_deref(), Some("1.23MiB/s"));
        assert_eq!(p.eta.as_deref(), Some("00:04"));
    }

    #[test]
    fn ignores_non_progress() {
        assert!(parse_progress_line("[info] Downloading").is_none());
        assert!(parse_progress_line("").is_none());
    }
}
