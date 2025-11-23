use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::api::{EpicClient, Game};
use crate::auth::AuthManager;
use crate::config::Config;
use crate::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledGame {
    pub app_name: String,
    pub app_title: String,
    pub app_version: String,
    pub install_path: PathBuf,
    pub executable: String,
}

impl InstalledGame {
    pub fn save(&self, config: &Config) -> Result<()> {
        let games_dir = Self::installed_games_dir(config)?;
        fs::create_dir_all(&games_dir)?;

        let game_file = games_dir.join(format!("{}.json", self.app_name));
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&game_file, contents)?;

        Ok(())
    }

    pub fn load(config: &Config, app_name: &str) -> Result<Self> {
        let games_dir = Self::installed_games_dir(config)?;
        let game_file = games_dir.join(format!("{}.json", app_name));

        if !game_file.exists() {
            return Err(Error::GameNotFound(app_name.to_string()));
        }

        let contents = fs::read_to_string(&game_file)?;
        Ok(serde_json::from_str(&contents)?)
    }

    pub fn list_installed(config: &Config) -> Result<Vec<Self>> {
        let games_dir = Self::installed_games_dir(config)?;

        if !games_dir.exists() {
            return Ok(vec![]);
        }

        let mut games = Vec::new();

        for entry in fs::read_dir(&games_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Ok(game) = serde_json::from_str::<InstalledGame>(&contents) {
                        games.push(game);
                    }
                }
            }
        }

        Ok(games)
    }

    pub fn delete(&self, config: &Config) -> Result<()> {
        let games_dir = Self::installed_games_dir(config)?;
        let game_file = games_dir.join(format!("{}.json", self.app_name));

        if game_file.exists() {
            fs::remove_file(&game_file)?;
        }

        Ok(())
    }

    fn installed_games_dir(_config: &Config) -> Result<PathBuf> {
        let data_dir = Config::data_dir()?;
        Ok(data_dir.join("installed"))
    }
}

pub struct GameManager {
    config: Config,
    auth: AuthManager,
    client: EpicClient,
}

impl GameManager {
    pub fn new(config: Config, auth: AuthManager) -> Result<Self> {
        let client = EpicClient::new()?;
        Ok(Self {
            config,
            auth,
            client,
        })
    }

    pub async fn list_library(&self) -> Result<Vec<Game>> {
        let token = self.auth.get_token()?;
        self.client.get_games(token).await
    }

    pub fn list_installed(&self) -> Result<Vec<InstalledGame>> {
        InstalledGame::list_installed(&self.config)
    }

    pub async fn install_game(&self, app_name: &str) -> Result<()> {
        // TODO: Check available disk space before installation
        // TODO: Implement resume capability for interrupted installations
        // TODO: Add progress tracking with download speed and ETA
        // TODO: Verify file integrity after reconstruction
        // TODO: Handle installation cancellation gracefully
        // TODO: Support selective installation (choose components/languages)

        let token = self.auth.get_token()?;

        log::info!("Starting installation for game: {}", app_name);

        // Download and parse game manifest
        println!("Downloading game manifest...");
        let manifest = self.client.download_manifest(token, app_name).await?;

        log::info!("Manifest downloaded: version {}", manifest.app_version);
        println!("Manifest version: {}", manifest.app_version);
        println!("Build size: {} bytes", manifest.build_size);
        println!("Files to download: {}", manifest.file_list.len());

        // Create install directory
        let install_path = self.config.install_dir.join(app_name);
        fs::create_dir_all(&install_path)?;

        log::info!("Created install directory: {:?}", install_path);

        // Download game files
        if !manifest.file_list.is_empty() {
            // TODO: Implement parallel file downloads with thread pool
            // TODO: Reconstruct files from downloaded chunks
            // TODO: Verify file checksums against manifest
            // TODO: Set proper file permissions (executable, read-only, etc.)
            // TODO: Handle sparse files correctly
            // TODO: Track and save download progress for resume capability

            println!("\nDownloading game files...");

            for (idx, file) in manifest.file_list.iter().enumerate() {
                println!(
                    "  [{}/{}] {}",
                    idx + 1,
                    manifest.file_list.len(),
                    file.filename
                );

                // Download chunks for this file
                for chunk in &file.file_chunk_parts {
                    let _chunk_data = self.client.download_chunk(&chunk.guid, token).await?;
                    // TODO: Reconstruct file from chunks
                    // TODO: Write chunks to file at correct offsets
                    // TODO: Verify chunk integrity before writing
                }
            }

            println!("✓ Game files downloaded");
        } else {
            println!("\nNote: Manifest parsing complete, but CDN download not fully implemented.");
            println!("Creating installation record with manifest data...");
        }

        // Create installed game entry with manifest data
        let installed_game = InstalledGame {
            app_name: app_name.to_string(),
            app_title: app_name.to_string(),
            app_version: manifest.app_version.clone(),
            install_path: install_path.clone(),
            executable: manifest.launch_exe.clone(),
        };

        installed_game.save(&self.config)?;

        log::info!("Game installation completed for: {}", app_name);
        println!("\n✓ Installation complete!");

        Ok(())
    }

    pub fn launch_game(&self, app_name: &str) -> Result<()> {
        let game = InstalledGame::load(&self.config, app_name)?;

        let executable_path = game.install_path.join(&game.executable);

        if !executable_path.exists() {
            return Err(Error::Other(format!(
                "Executable not found: {:?}",
                executable_path
            )));
        }

        log::info!("Launching game: {} ({})", game.app_title, game.app_name);

        Command::new(&executable_path)
            .current_dir(&game.install_path)
            .spawn()
            .map_err(|e| Error::Other(format!("Failed to launch game: {}", e)))?;

        Ok(())
    }

    pub fn uninstall_game(&self, app_name: &str) -> Result<()> {
        let game = InstalledGame::load(&self.config, app_name)?;

        // Remove game files
        if game.install_path.exists() {
            fs::remove_dir_all(&game.install_path)?;
        }

        // Remove metadata
        game.delete(&self.config)?;

        log::info!("Uninstalled game: {} ({})", game.app_title, game.app_name);

        Ok(())
    }

    /// Check for game updates
    pub async fn check_for_updates(&self, app_name: &str) -> Result<Option<String>> {
        let token = self.auth.get_token()?;
        let game = InstalledGame::load(&self.config, app_name)?;

        log::info!(
            "Checking for updates for {} (current: {})",
            app_name,
            game.app_version
        );

        self.client
            .check_for_updates(token, app_name, &game.app_version)
            .await
    }

    /// Update a game to the latest version
    pub async fn update_game(&self, app_name: &str) -> Result<()> {
        // TODO: Implement differential updates (download only changed files)
        // TODO: Compare old and new manifests to identify changes
        // TODO: Support update rollback in case of failure
        // TODO: Preserve user settings and save files during update
        // TODO: Show update changelog to user

        let token = self.auth.get_token()?;

        log::info!("Updating game: {}", app_name);

        // Check if update is available
        match self.check_for_updates(app_name).await? {
            Some(new_version) => {
                println!("Update available: {}", new_version);
                println!("Downloading update...");

                // Download new manifest
                let manifest = self.client.download_manifest(token, app_name).await?;

                // Update game files (differential update would be more efficient)
                println!("Updating game files...");

                // Update installation record
                let mut game = InstalledGame::load(&self.config, app_name)?;
                game.app_version = manifest.app_version.clone();
                game.executable = manifest.launch_exe.clone();
                game.save(&self.config)?;

                println!("✓ Game updated to version {}", manifest.app_version);
                Ok(())
            }
            None => {
                println!("Game is already up to date");
                Ok(())
            }
        }
    }

    /// Download cloud saves for a game
    pub async fn download_cloud_saves(&self, app_name: &str) -> Result<()> {
        // TODO: Implement conflict resolution for cloud vs local saves
        // TODO: Compare timestamps to detect newer save
        // TODO: Allow user to choose which save to keep
        // TODO: Create backup of local saves before overwriting
        // TODO: Support automatic sync on game launch/exit

        let token = self.auth.get_token()?;
        let game = InstalledGame::load(&self.config, app_name)?;

        log::info!("Downloading cloud saves for {}", app_name);
        println!("Fetching cloud saves...");

        let saves = self.client.get_cloud_saves(token, app_name).await?;

        if saves.is_empty() {
            println!("No cloud saves found");
            return Ok(());
        }

        println!("Found {} cloud save(s)", saves.len());

        // Create saves directory
        let saves_dir = game.install_path.join("saves");
        fs::create_dir_all(&saves_dir)?;

        for save in saves {
            println!("  Downloading: {}", save.filename);
            let save_data = self.client.download_cloud_save(token, &save.id).await?;

            let save_path = saves_dir.join(&save.filename);
            fs::write(&save_path, &save_data)?;

            log::info!("Downloaded save: {:?}", save_path);
        }

        println!("✓ Cloud saves downloaded");
        Ok(())
    }

    /// Upload cloud saves for a game
    pub async fn upload_cloud_saves(&self, app_name: &str) -> Result<()> {
        let token = self.auth.get_token()?;
        let game = InstalledGame::load(&self.config, app_name)?;

        log::info!("Uploading cloud saves for {}", app_name);
        println!("Uploading cloud saves...");

        let saves_dir = game.install_path.join("saves");

        if !saves_dir.exists() {
            println!("No local saves found");
            return Ok(());
        }

        let mut uploaded = 0;

        for entry in fs::read_dir(&saves_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let save_data = fs::read(&path)?;
                println!(
                    "  Uploading: {}",
                    path.file_name().unwrap().to_string_lossy()
                );

                self.client
                    .upload_cloud_save(token, app_name, &save_data)
                    .await?;
                uploaded += 1;
            }
        }

        println!("✓ Uploaded {} save file(s)", uploaded);
        Ok(())
    }
}
