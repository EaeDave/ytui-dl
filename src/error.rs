use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("yt-dlp não encontrado no PATH. Instale com: pacman -S yt-dlp | apt install yt-dlp | brew install yt-dlp")]
    YtDlpNotFound,

    #[error("falha ao executar yt-dlp: {0}")]
    YtDlpExec(String),

    #[error("yt-dlp retornou erro: {0}")]
    YtDlpFailed(String),

    #[error("não foi possível parsear metadata JSON: {0}")]
    MetadataParse(String),

    #[error("URL inválida ou vazia")]
    InvalidUrl,

    #[error("IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("config: {0}")]
    Config(String),

    #[error("caminho inválido: {0}")]
    InvalidPath(PathBuf),
}
