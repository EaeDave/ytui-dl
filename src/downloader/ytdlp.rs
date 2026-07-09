use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use serde::Deserialize;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::action::Action;
use crate::error::{AppError, Result};
use crate::models::{AudioFormat, MediaMode, QualityPreset, VideoInfo};

use super::progress::{parse_progress_line, PROGRESS_TEMPLATE};

#[derive(Debug, Clone)]
pub struct Tools {
    pub ytdlp: PathBuf,
    pub ffmpeg: Option<PathBuf>,
}

impl Tools {
    pub fn detect() -> Result<Self> {
        let ytdlp = which::which("yt-dlp").map_err(|_| AppError::YtDlpNotFound)?;
        let ffmpeg = which::which("ffmpeg").ok();
        Ok(Self { ytdlp, ffmpeg })
    }

    pub fn has_ffmpeg(&self) -> bool {
        self.ffmpeg.is_some()
    }
}

#[derive(Debug, Deserialize)]
struct YtDlpJson {
    id: Option<String>,
    title: Option<String>,
    uploader: Option<String>,
    channel: Option<String>,
    duration: Option<f64>,
    webpage_url: Option<String>,
    original_url: Option<String>,
    thumbnail: Option<String>,
}

/// Fetch video metadata via `yt-dlp -J --no-playlist`.
pub async fn fetch_video_info(tools: &Tools, url: &str) -> Result<VideoInfo> {
    let url = url.trim();
    if url.is_empty() {
        return Err(AppError::InvalidUrl);
    }

    let output = Command::new(&tools.ytdlp)
        .args([
            "-J",
            "--no-playlist",
            "--no-warnings",
            "--",
            url,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| AppError::YtDlpExec(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let msg = first_nonempty_line(&stderr)
            .or_else(|| first_nonempty_line(&stdout))
            .unwrap_or("yt-dlp failed to fetch metadata")
            .to_string();
        return Err(AppError::YtDlpFailed(msg));
    }

    let json: YtDlpJson = serde_json::from_slice(&output.stdout)
        .map_err(|e| AppError::MetadataParse(e.to_string()))?;

    Ok(VideoInfo {
        id: json.id.unwrap_or_else(|| "unknown".into()),
        title: json.title.unwrap_or_else(|| "Untitled".into()),
        uploader: json
            .uploader
            .or(json.channel)
            .unwrap_or_else(|| "Unknown".into()),
        duration_secs: json.duration.map(|d| d.max(0.0) as u64),
        webpage_url: json
            .webpage_url
            .or(json.original_url)
            .unwrap_or_else(|| url.to_string()),
        thumbnail: json.thumbnail,
    })
}

#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub job_id: Uuid,
    pub url: String,
    pub mode: MediaMode,
    pub quality: QualityPreset,
    pub audio_format: AudioFormat,
    pub output_template: PathBuf,
}

/// Tracks the best-known output file path from yt-dlp stdout/stderr.
pub type PathTracker = Arc<Mutex<Option<PathBuf>>>;

pub fn new_path_tracker() -> PathTracker {
    Arc::new(Mutex::new(None))
}

/// Spawn yt-dlp and stream progress events into `tx`.
/// Returns the child process and a path tracker filled as yt-dlp reports file paths.
pub async fn start_download(
    tools: &Tools,
    req: DownloadRequest,
    tx: mpsc::UnboundedSender<Action>,
    last_path: PathTracker,
) -> Result<Child> {
    // Ensure parent directory exists
    if let Some(parent) = req.output_template.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let out_tmpl = req
        .output_template
        .to_str()
        .ok_or_else(|| AppError::InvalidPath(req.output_template.clone()))?;

    let mut args: Vec<String> = vec![
        "--no-playlist".into(),
        "--newline".into(),
        "--progress".into(),
        "--progress-template".into(),
        PROGRESS_TEMPLATE.into(),
        // Emit final file path after download/move for auto-open.
        "--print".into(),
        "after_move:filepath".into(),
        "--print".into(),
        "after_video:filepath".into(),
        "-o".into(),
        out_tmpl.into(),
        "--no-mtime".into(),
    ];

    match req.mode {
        MediaMode::Video => {
            args.push("-f".into());
            args.push(req.quality.format_selector().into());
            // Prefer merge to mp4 when possible
            args.push("--merge-output-format".into());
            args.push("mp4".into());
        }
        MediaMode::Audio => {
            args.push("-x".into());
            if let Some(fmt) = req.audio_format.yt_dlp_arg() {
                args.push("--audio-format".into());
                args.push(fmt.into());
            }
            args.push("-f".into());
            args.push("ba/b".into());
        }
    }

    if let Some(ffmpeg) = &tools.ffmpeg {
        if let Some(dir) = ffmpeg.parent() {
            args.push("--ffmpeg-location".into());
            args.push(dir.display().to_string());
        }
    }

    args.push("--".into());
    args.push(req.url.clone());

    let mut child = Command::new(&tools.ytdlp)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| AppError::YtDlpExec(e.to_string()))?;

    let job_id = req.job_id;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let _ = tx.send(Action::DownloadStarted { job_id });

    if let Some(out) = stdout {
        let tx_out = tx.clone();
        let path_out = last_path.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(out).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Some(update) = parse_progress_line(&line) {
                    let _ = tx_out.send(Action::DownloadProgress { job_id, update });
                } else if let Some(path) = extract_filepath(&line) {
                    if let Ok(mut guard) = path_out.lock() {
                        *guard = Some(path);
                    }
                }
            }
        });
    }

    if let Some(err) = stderr {
        let tx_err = tx.clone();
        let path_err = last_path.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(err).lines();
            let mut last_error: Option<String> = None;
            while let Ok(Some(line)) = lines.next_line().await {
                if let Some(update) = parse_progress_line(&line) {
                    let _ = tx_err.send(Action::DownloadProgress { job_id, update });
                } else if let Some(path) = extract_filepath(&line) {
                    if let Ok(mut guard) = path_err.lock() {
                        *guard = Some(path);
                    }
                } else if looks_like_error(&line) {
                    last_error = Some(line);
                }
            }
            let _ = last_error;
        });
    }

    Ok(child)
}

/// Wait for download completion or cancellation.
///
/// When `cancel` receives a unit value, the child process is killed and
/// [`Action::DownloadCancelled`] is emitted.
pub async fn watch_download(
    mut child: Child,
    job_id: Uuid,
    output_dir: PathBuf,
    last_path: PathTracker,
    tx: mpsc::UnboundedSender<Action>,
    mut cancel: mpsc::UnboundedReceiver<()>,
) {
    tokio::select! {
        _ = cancel.recv() => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            let _ = tx.send(Action::DownloadCancelled { job_id });
        }
        result = child.wait() => {
            match result {
                Ok(status) if status.success() => {
                    // Brief pause so stdout flush from --print can land in the tracker.
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                    let path = last_path
                        .lock()
                        .ok()
                        .and_then(|g| g.clone())
                        .filter(|p| p.is_file())
                        .or_else(|| newest_media_file(&output_dir))
                        .unwrap_or(output_dir);
                    let _ = tx.send(Action::DownloadFinished {
                        job_id,
                        output_path: Some(path),
                    });
                }
                Ok(status) => {
                    let code = status
                        .code()
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "signal".into());
                    let _ = tx.send(Action::DownloadFailed {
                        job_id,
                        error: format!("yt-dlp exited with code {code}"),
                    });
                }
                Err(e) => {
                    let _ = tx.send(Action::DownloadFailed {
                        job_id,
                        error: e.to_string(),
                    });
                }
            }
        }
    }
}

fn extract_filepath(line: &str) -> Option<PathBuf> {
    let line = line.trim();
    if line.is_empty() || parse_progress_line(line).is_some() {
        return None;
    }

    if let Some(rest) = line.strip_prefix("[download] Destination: ") {
        let p = PathBuf::from(rest.trim());
        return Some(p);
    }
    if let Some(rest) = line.strip_prefix("[Merger] Merging formats into \"") {
        let rest = rest.trim_end_matches('"');
        return Some(PathBuf::from(rest));
    }
    if let Some(rest) = line.strip_prefix("[ExtractAudio] Destination: ") {
        return Some(PathBuf::from(rest.trim()));
    }

    // Bare path from --print after_*:filepath
    let p = PathBuf::from(line);
    if line.starts_with('[') {
        return None;
    }
    let looks_media = p
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "mp4" | "mkv" | "webm" | "mp3" | "m4a" | "opus" | "ogg" | "wav" | "flac" | "aac"
            )
        });
    if looks_media && (p.is_absolute() || line.contains('/')) {
        return Some(p);
    }
    None
}

/// Fallback: most recently modified media file in the output directory.
pub fn newest_media_file(dir: &Path) -> Option<PathBuf> {
    let mut best: Option<(SystemTime, PathBuf)> = None;
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())?;
        if !matches!(
            ext.as_str(),
            "mp4" | "mkv" | "webm" | "mp3" | "m4a" | "opus" | "ogg" | "wav" | "flac" | "aac"
        ) {
            continue;
        }
        let modified = entry.metadata().ok()?.modified().ok()?;
        match &best {
            Some((t, _)) if modified <= *t => {}
            _ => best = Some((modified, path)),
        }
    }
    best.map(|(_, p)| p)
}

fn first_nonempty_line(s: &str) -> Option<&str> {
    s.lines().map(str::trim).find(|l| !l.is_empty())
}

fn looks_like_error(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("error") || lower.contains("erro") || lower.starts_with("error:")
}

/// Build absolute output template path from config pieces.
pub fn build_output_template(output_dir: &Path, template: &str) -> PathBuf {
    output_dir.join(template)
}
