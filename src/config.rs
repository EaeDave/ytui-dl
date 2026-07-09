use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};
use crate::i18n::Language;
use crate::models::{AudioFormat, MediaMode, QualityPreset};

const APP_DIR: &str = "ytui-dl";
const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub output_dir: PathBuf,
    pub output_template: String,
    pub default_mode: MediaMode,
    pub default_quality: QualityPreset,
    pub default_audio_format: AudioFormat,
    #[serde(default)]
    pub language: Language,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_dir: default_output_dir(),
            output_template: "%(title)s [%(id)s].%(ext)s".into(),
            default_mode: MediaMode::Video,
            default_quality: QualityPreset::Best,
            default_audio_format: AudioFormat::M4a,
            language: Language::En,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        match load_from_disk() {
            Ok(cfg) => cfg,
            Err(_) => {
                let cfg = Self::default();
                let _ = cfg.save();
                cfg
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content =
            toml::to_string_pretty(self).map_err(|e| AppError::Config(e.to_string()))?;
        fs::write(&path, content)?;
        Ok(())
    }
}

fn default_output_dir() -> PathBuf {
    dirs::download_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ytui-dl")
}

fn config_dir() -> Result<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| {
        AppError::Config("could not determine config directory".into())
    })?;
    Ok(base.join(APP_DIR))
}

fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join(CONFIG_FILE))
}

fn load_from_disk() -> Result<Config> {
    let path = config_path()?;
    let content = fs::read_to_string(&path)?;
    toml::from_str(&content).map_err(|e| AppError::Config(e.to_string()))
}
