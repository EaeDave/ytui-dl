use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};
use crate::i18n::Language;
use crate::models::{AudioFormat, MediaMode, OutputProfile, QualityPreset};

const APP_DIR: &str = "ytd";
/// Previous app id — read once for migration, never written again.
const LEGACY_APP_DIR: &str = "ytui-dl";
const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub output_dir: PathBuf,
    pub output_template: String,
    pub default_mode: MediaMode,
    #[serde(default)]
    pub default_profile: OutputProfile,
    pub default_quality: QualityPreset,
    pub default_audio_format: AudioFormat,
    #[serde(default)]
    pub language: Language,
    /// Open the file with the system default app when a download finishes.
    #[serde(default = "default_true")]
    pub auto_open: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_dir: default_output_dir(),
            output_template: "%(title)s [%(id)s].%(ext)s".into(),
            default_mode: MediaMode::Video,
            default_profile: OutputProfile::WhatsApp,
            default_quality: QualityPreset::Best,
            default_audio_format: AudioFormat::M4a,
            language: Language::En,
            auto_open: true,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        match load_from_disk() {
            Ok(cfg) => {
                let cfg = migrate_legacy_paths(cfg);
                // Always rewrite to the current config location so renames stick.
                let _ = cfg.save();
                cfg
            }
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
        .join(APP_DIR)
}

/// If the last path component is the old app name, rename it to `ytd`.
/// e.g. `~/Downloads/ytui-dl` → `~/Downloads/ytd`
fn migrate_legacy_output_dir(path: PathBuf) -> PathBuf {
    match path.file_name().and_then(|n| n.to_str()) {
        Some(name) if name == LEGACY_APP_DIR => path.with_file_name(APP_DIR),
        _ => path,
    }
}

fn migrate_legacy_paths(mut cfg: Config) -> Config {
    cfg.output_dir = migrate_legacy_output_dir(cfg.output_dir);
    cfg
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

fn legacy_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|b| b.join(LEGACY_APP_DIR).join(CONFIG_FILE))
}

fn load_from_disk() -> Result<Config> {
    let path = config_path()?;
    if path.is_file() {
        let content = fs::read_to_string(&path)?;
        return toml::from_str(&content).map_err(|e| AppError::Config(e.to_string()));
    }

    // One-shot migrate preferences from the old ytui-dl config location.
    if let Some(legacy) = legacy_config_path() {
        if legacy.is_file() {
            let content = fs::read_to_string(&legacy)?;
            let cfg: Config =
                toml::from_str(&content).map_err(|e| AppError::Config(e.to_string()))?;
            return Ok(cfg);
        }
    }

    Err(AppError::Config("config not found".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renames_legacy_download_folder() {
        let p = PathBuf::from("/home/u/Downloads/ytui-dl");
        assert_eq!(
            migrate_legacy_output_dir(p),
            PathBuf::from("/home/u/Downloads/ytd")
        );
    }

    #[test]
    fn leaves_custom_download_folder() {
        let p = PathBuf::from("/home/u/Videos/clips");
        assert_eq!(migrate_legacy_output_dir(p.clone()), p);
    }
}
