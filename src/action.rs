use uuid::Uuid;

use crate::models::{ProgressUpdate, VideoInfo};

/// Messages that drive the application state machine.
#[derive(Debug)]
pub enum Action {
    Tick,
    Render,
    Key(crossterm::event::KeyEvent),
    Paste(String),
    #[allow(dead_code)]
    Resize(u16, u16),

    MetadataReady(VideoInfo),
    MetadataFailed(String),

    /// Worker progress / lifecycle events.
    DownloadProgress {
        job_id: Uuid,
        update: ProgressUpdate,
    },
    DownloadStarted {
        job_id: Uuid,
    },
    DownloadFinished {
        job_id: Uuid,
        output_path: Option<std::path::PathBuf>,
    },
    DownloadFailed {
        job_id: Uuid,
        error: String,
    },
    DownloadCancelled {
        job_id: Uuid,
    },

    /// Newer GitHub release detected (version without leading `v`).
    UpdateAvailable {
        version: String,
    },
    UpdateProgress {
        message: String,
    },
    UpdateSucceeded {
        version: String,
    },
    UpdateFailed {
        error: String,
    },

    Status(String),
}
