use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("yt-dlp not found in PATH. Install with: pacman -S yt-dlp | apt install yt-dlp | brew install yt-dlp")]
    YtDlpNotFound,

    #[error("failed to run yt-dlp: {0}")]
    YtDlpExec(String),

    #[error("yt-dlp error: {0}")]
    YtDlpFailed(String),

    #[error("could not parse metadata JSON: {0}")]
    MetadataParse(String),

    #[error("invalid or empty URL")]
    InvalidUrl,

    #[error("IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("config: {0}")]
    Config(String),

    #[error("invalid path: {0}")]
    InvalidPath(PathBuf),
}
