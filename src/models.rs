use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::i18n::Strings;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Home,
    Preview,
    Queue,
    Settings,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaMode {
    #[default]
    Video,
    Audio,
}

impl MediaMode {
    pub fn label(self, t: &Strings) -> &'static str {
        match self {
            Self::Video => t.mode_video,
            Self::Audio => t.mode_audio,
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            Self::Video => Self::Audio,
            Self::Audio => Self::Video,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityPreset {
    #[default]
    Best,
    P1080,
    P720,
    P480,
    Worst,
}

impl QualityPreset {
    pub const ALL: [Self; 5] = [
        Self::Best,
        Self::P1080,
        Self::P720,
        Self::P480,
        Self::Worst,
    ];

    pub fn label(self, t: &Strings) -> &'static str {
        match self {
            Self::Best => t.quality_best,
            Self::P1080 => "1080p",
            Self::P720 => "720p",
            Self::P480 => "480p",
            Self::Worst => t.quality_worst,
        }
    }

    /// Format selector string for yt-dlp `-f`.
    pub fn format_selector(self) -> &'static str {
        match self {
            Self::Best => "bv*+ba/b",
            Self::P1080 => "bv*[height<=1080]+ba/b[height<=1080]/b",
            Self::P720 => "bv*[height<=720]+ba/b[height<=720]/b",
            Self::P480 => "bv*[height<=480]+ba/b[height<=480]/b",
            Self::Worst => "wv*+wa/w",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Best => Self::P1080,
            Self::P1080 => Self::P720,
            Self::P720 => Self::P480,
            Self::P480 => Self::Worst,
            Self::Worst => Self::Best,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Best => Self::Worst,
            Self::P1080 => Self::Best,
            Self::P720 => Self::P1080,
            Self::P480 => Self::P720,
            Self::Worst => Self::P480,
        }
    }

    pub fn from_digit(d: char) -> Option<Self> {
        match d {
            '1' => Some(Self::Best),
            '2' => Some(Self::P1080),
            '3' => Some(Self::P720),
            '4' => Some(Self::P480),
            '5' => Some(Self::Worst),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioFormat {
    #[default]
    M4a,
    Mp3,
    Opus,
    Best,
}

impl AudioFormat {
    pub const ALL: [Self; 4] = [Self::M4a, Self::Mp3, Self::Opus, Self::Best];

    pub fn label(self, t: &Strings) -> &'static str {
        match self {
            Self::M4a => "M4A",
            Self::Mp3 => "MP3",
            Self::Opus => "Opus",
            Self::Best => t.audio_best,
        }
    }

    /// Argument for `--audio-format` when extracting audio.
    pub fn yt_dlp_arg(self) -> Option<&'static str> {
        match self {
            Self::M4a => Some("m4a"),
            Self::Mp3 => Some("mp3"),
            Self::Opus => Some("opus"),
            Self::Best => None,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::M4a => Self::Mp3,
            Self::Mp3 => Self::Opus,
            Self::Opus => Self::Best,
            Self::Best => Self::M4a,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::M4a => Self::Best,
            Self::Mp3 => Self::M4a,
            Self::Opus => Self::Mp3,
            Self::Best => Self::Opus,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JobStatus {
    #[default]
    Queued,
    Downloading,
    Done,
    Failed,
    Cancelled,
}

impl JobStatus {
    pub fn label(self, t: &Strings) -> &'static str {
        match self {
            Self::Queued => t.status_queued,
            Self::Downloading => t.status_downloading,
            Self::Done => t.status_done,
            Self::Failed => t.status_failed,
            Self::Cancelled => t.status_cancelled,
        }
    }

    pub fn is_active(self) -> bool {
        matches!(self, Self::Queued | Self::Downloading)
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Done | Self::Failed | Self::Cancelled)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    UrlInput,
    Mode,
    Quality,
    AudioFormat,
    Confirm,
    QueueList,
    SettingsOutput,
    SettingsTemplate,
    SettingsLanguage,
}

#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub id: String,
    pub title: String,
    pub uploader: String,
    pub duration_secs: Option<u64>,
    pub webpage_url: String,
    #[allow(dead_code)]
    pub thumbnail: Option<String>,
}

impl VideoInfo {
    pub fn duration_label(&self) -> String {
        match self.duration_secs {
            Some(secs) => {
                let h = secs / 3600;
                let m = (secs % 3600) / 60;
                let s = secs % 60;
                if h > 0 {
                    format!("{h:02}:{m:02}:{s:02}")
                } else {
                    format!("{m:02}:{s:02}")
                }
            }
            None => "—".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DownloadJob {
    pub id: Uuid,
    pub url: String,
    pub mode: MediaMode,
    pub quality: QualityPreset,
    pub audio_format: AudioFormat,
    pub status: JobStatus,
    /// Progress 0.0..=100.0
    pub progress: f64,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub error: Option<String>,
    pub output_path: Option<PathBuf>,
    pub title: Option<String>,
}

impl DownloadJob {
    pub fn new(
        url: String,
        mode: MediaMode,
        quality: QualityPreset,
        audio_format: AudioFormat,
        title: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            url,
            mode,
            quality,
            audio_format,
            status: JobStatus::Queued,
            progress: 0.0,
            speed: None,
            eta: None,
            error: None,
            output_path: None,
            title,
        }
    }

    pub fn display_title(&self) -> &str {
        self.title
            .as_deref()
            .filter(|t| !t.is_empty())
            .unwrap_or(&self.url)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProgressUpdate {
    pub percent: Option<f64>,
    pub speed: Option<String>,
    pub eta: Option<String>,
    #[allow(dead_code)]
    pub total_bytes: Option<u64>,
    #[allow(dead_code)]
    pub downloaded_bytes: Option<u64>,
}
