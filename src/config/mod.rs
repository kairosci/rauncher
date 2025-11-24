use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::{Error, Result};

// TODO: Add more configuration options:
// - download_threads: Number of concurrent downloads
// - bandwidth_limit: Optional download speed limit
// - cdn_region: Preferred CDN region
// - auto_update: Auto-update games in background
// - proxy_settings: HTTP/SOCKS proxy configuration
// - cache_size: Maximum cache size for manifests/metadata

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub install_dir: PathBuf,
    pub log_level: String,
    // Opzioni avanzate (alcune opzionali per retro‑compatibilità)
    #[serde(default = "default_download_threads")]
    pub download_threads: usize,
    #[serde(default)]
    pub bandwidth_limit_kbps: Option<u64>,
    #[serde(default)]
    pub cdn_region: Option<String>,
    #[serde(default)]
    pub auto_update: bool,
    #[serde(default)]
    pub proxy: Option<String>,
    #[serde(default = "default_cache_size_mb")]
    pub cache_size_mb: u64,
}

impl Default for Config {
    fn default() -> Self {
        let project_dirs = ProjectDirs::from("", "", "rauncher")
            .expect("Failed to determine project directories");

        Self {
            install_dir: project_dirs.data_dir().join("games"),
            log_level: "info".to_string(),
            download_threads: default_download_threads(),
            bandwidth_limit_kbps: None,
            cdn_region: None,
            auto_update: false,
            proxy: None,
            cache_size_mb: default_cache_size_mb(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        // Nota: i nuovi campi hanno default, quindi il caricamento resta retro‑compatibile
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)?;
            // Applica default ai nuovi campi eventualmente assenti
            let mut config: Config = toml::from_str(&contents)?;
            let defaults = Config::default();
            // Merge semplice: se alcuni valori sono "vuoti", sostituisci con default sensati
            if config.download_threads == 0 {
                config.download_threads = defaults.download_threads;
            }
            if config.cache_size_mb == 0 {
                config.cache_size_mb = defaults.cache_size_mb;
            }
            config.validate()?;
            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Validate configuration values
    fn validate(&self) -> Result<()> {
        // Validate log level
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(Error::Config(format!(
                "Invalid log level: '{}'. Must be one of: {}",
                self.log_level,
                valid_log_levels.join(", ")
            )));
        }

        // Validate install directory - ensure parent exists or can be created
        if let Some(parent) = self.install_dir.parent() {
            if !parent.exists() {
                return Err(Error::Config(format!(
                    "Install directory parent does not exist: {}",
                    parent.display()
                )));
            }
        }

        if self.download_threads == 0 {
            return Err(Error::Config("download_threads must be >= 1".to_string()));
        }
        if let Some(kbps) = self.bandwidth_limit_kbps {
            if kbps == 0 {
                return Err(Error::Config("bandwidth_limit_kbps must be > 0 if set".to_string()));
            }
        }
        if self.cache_size_mb == 0 {
            return Err(Error::Config("cache_size_mb must be >= 1".to_string()));
        }

        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self).map_err(|e| Error::Config(e.to_string()))?;
        fs::write(&config_path, contents)?;

        Ok(())
    }

    pub fn config_path() -> Result<PathBuf> {
        let project_dirs = ProjectDirs::from("", "", "rauncher")
            .ok_or_else(|| Error::Config("Failed to determine project directories".to_string()))?;

        Ok(project_dirs.config_dir().join("config.toml"))
    }

    pub fn data_dir() -> Result<PathBuf> {
        let project_dirs = ProjectDirs::from("", "", "rauncher")
            .ok_or_else(|| Error::Config("Failed to determine project directories".to_string()))?;

        Ok(project_dirs.data_dir().to_path_buf())
    }
}

fn default_download_threads() -> usize { 4 }
fn default_cache_size_mb() -> u64 { 512 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.log_level, "info");
        assert!(config.install_dir.to_string_lossy().contains("games"));
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(config.log_level, deserialized.log_level);
    }
}
