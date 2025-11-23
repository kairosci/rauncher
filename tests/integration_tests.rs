use rauncher::{auth::AuthManager, config::Config, games::GameManager};
use std::fs;
use tempfile::TempDir;

/// Integration test for configuration management
#[test]
fn test_config_creation_and_loading() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // Create a config
    let config = Config {
        install_dir: temp_dir.path().join("games"),
        log_level: "info".to_string(),
    };

    // Save it
    let config_str = toml::to_string(&config).unwrap();
    fs::write(&config_path, config_str).unwrap();

    // Load it back
    let loaded_config: Config = toml::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();

    assert_eq!(config.log_level, loaded_config.log_level);
}

/// Test authentication token persistence
#[test]
fn test_auth_manager_initialization() {
    let auth = AuthManager::new().unwrap();
    // Should not be authenticated initially in a fresh environment
    // Note: This assumes no existing auth file
    assert!(!auth.is_authenticated());
}

/// Test game manager can be created
#[test]
fn test_game_manager_creation() {
    let temp_dir = TempDir::new().unwrap();
    let config = Config {
        install_dir: temp_dir.path().join("games"),
        log_level: "info".to_string(),
    };

    let auth = AuthManager::new().unwrap();
    let manager = GameManager::new(config, auth);

    assert!(manager.is_ok());
}

/// Test listing installed games when none exist
#[test]
fn test_list_installed_games_empty() {
    let temp_dir = TempDir::new().unwrap();
    let config = Config {
        install_dir: temp_dir.path().join("games"),
        log_level: "info".to_string(),
    };

    let auth = AuthManager::new().unwrap();
    let manager = GameManager::new(config, auth).unwrap();

    let games = manager.list_installed().unwrap();
    assert_eq!(games.len(), 0);
}

/// Test directory creation for installation
#[test]
fn test_install_directory_setup() {
    let temp_dir = TempDir::new().unwrap();
    let install_dir = temp_dir.path().join("games");

    // Directory should not exist initially
    assert!(!install_dir.exists());

    // Create it
    fs::create_dir_all(&install_dir).unwrap();

    // Now it should exist
    assert!(install_dir.exists());
    assert!(install_dir.is_dir());
}

/// Test error handling for invalid paths
#[test]
fn test_invalid_install_path_handling() {
    let temp_dir = TempDir::new().unwrap();
    let config = Config {
        install_dir: temp_dir.path().join("games"),
        log_level: "info".to_string(),
    };

    let auth = AuthManager::new().unwrap();
    let manager = GameManager::new(config, auth).unwrap();

    // Try to uninstall a non-existent game
    let result = manager.uninstall_game("nonexistent_game");
    assert!(result.is_err());
}
