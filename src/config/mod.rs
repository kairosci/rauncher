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
}

impl Default for Config {
    fn default() -> Self {
        let project_dirs = ProjectDirs::from("", "", "rauncher")
            .expect("Failed to determine project directories");

        Self {
            install_dir: project_dirs.data_dir().join("games"),
            log_level: "info".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        // TODO: Handle config migration for version changes
        // TODO: Merge user config with defaults for missing values
        // TODO: Add config file watching for hot-reload

        let config_path = Self::config_path()?;

        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&contents)?;
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
