use clap::Parser;
use rauncher::{
    auth::AuthManager,
    cli::{Cli, Commands},
    config::Config,
    games::GameManager,
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    // Load configuration
    let config = Config::load()?;
    log::debug!("Configuration loaded");

    // Initialize auth manager
    let mut auth = AuthManager::new()?;

    match cli.command {
        None => {
            // Launch GUI when no command is provided
            use rauncher::gui::LauncherApp;

            let native_options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([1200.0, 800.0])
                    .with_min_inner_size([800.0, 600.0])
                    .with_title("R Games Launcher"),
                ..Default::default()
            };

            if let Err(e) = eframe::run_native(
                "Rauncher",
                native_options,
                Box::new(|cc| Ok(Box::new(LauncherApp::new(cc)))),
            ) {
                log::error!("Failed to run GUI: {}", e);
                std::process::exit(1);
            }
        }

        Some(command) => match command {
            Commands::Auth { logout } => {
                if logout {
                    auth.logout()?;
                    log::info!("Successfully logged out");
                } else {
                    use rauncher::api::EpicClient;

                    log::info!("Epic Games Store Authentication");

                    let client = EpicClient::new()?;

                    log::info!("Starting authentication process...");

                    match client.authenticate().await {
                        Ok((user_code, verification_url, token)) => {
                            log::info!("Please authenticate using your web browser:");
                            log::info!("Open this URL: {}", verification_url);
                            log::info!("Enter this code: {}", user_code);
                            log::info!("Waiting for authentication...");

                            // Save the token
                            auth.set_token(token)?;

                            log::info!("✓ Successfully authenticated with Epic Games Store!");
                            log::info!("You can now:");
                            log::info!("List your games: rauncher list");
                            log::info!("Install a game: rauncher install <app_name>");
                        }
                        Err(e) => {
                            log::error!("Authentication failed: {}", e);
                            log::error!("Please try again. If the problem persists, check:");
                            log::error!("Your internet connection");
                            log::error!("Epic Games services status");
                            std::process::exit(1);
                        }
                    }
                }
            }

            Commands::List { installed } => {
                if installed {
                    let manager = GameManager::new(config, auth)?;
                    let games = manager.list_installed()?;

                    if games.is_empty() {
                        log::info!("No games installed");
                    } else {
                        log::info!("Installed Games:");
                        log::info!("================");
                        for game in games {
                            log::info!(
                                "  {} - {} (v{})",
                                game.app_name, game.app_title, game.app_version
                            );
                            log::info!("    Path: {:?}", game.install_path);
                        }
                    }
                } else {
                    if !auth.is_authenticated() {
                        log::error!("Error: Not authenticated. Run 'rauncher auth' first.");
                        std::process::exit(1);
                    }

                    let mut manager = GameManager::new(config, auth)?;
                    let games = manager.list_library().await?;

                    if games.is_empty() {
                        log::info!("No games in library (or authentication required)");
                    } else {
                        log::info!("Library:");
                        log::info!("========");
                        for game in games {
                            log::info!(
                                "  {} - {} (v{})",
                                game.app_name, game.app_title, game.app_version
                            );
                        }
                    }
                }
            }

            Commands::Install { app_name } => {
                if !auth.is_authenticated() {
                    log::error!("Error: Not authenticated. Run 'rauncher auth' first.");
                    std::process::exit(1);
                }

                let mut manager = GameManager::new(config, auth)?;
                log::info!("Installing game: {}", app_name);

                match manager.install_game(&app_name).await {
                    Ok(()) => log::info!("Game installed successfully!"),
                    Err(e) => {
                        log::error!("Failed to install game: {}", e);
                        std::process::exit(1);
                    }
                }
            }

            Commands::Launch { app_name } => {
                let manager = GameManager::new(config, auth)?;

                match manager.launch_game(&app_name) {
                    Ok(()) => log::info!("Game launched successfully!"),
                    Err(e) => {
                        log::error!("Failed to launch game: {}", e);
                        std::process::exit(1);
                    }
                }
            }

            Commands::Uninstall { app_name } => {
                let manager = GameManager::new(config, auth)?;

                match manager.uninstall_game(&app_name) {
                    Ok(()) => log::info!("Game uninstalled successfully!"),
                    Err(e) => {
                        log::error!("Failed to uninstall game: {}", e);
                        std::process::exit(1);
                    }
                }
            }

            Commands::Info { app_name } => {
                let manager = GameManager::new(config, auth)?;

                match manager
                    .list_installed()?
                    .iter()
                    .find(|g| g.app_name == app_name)
                {
                    Some(game) => {
                        log::info!("Game Information:");
                        log::info!("================");
                        log::info!("Name: {}", game.app_name);
                        log::info!("Title: {}", game.app_title);
                        log::info!("Version: {}", game.app_version);
                        log::info!("Install Path: {:?}", game.install_path);
                        log::info!("Executable: {}", game.executable);
                    }
                    None => {
                        log::error!("Game not found: {}", app_name);
                        std::process::exit(1);
                    }
                }
            }

            Commands::Status => {
                log::info!("R Games Launcher Status");
                log::info!("=======================");
                log::info!("Version: {}", env!("CARGO_PKG_VERSION"));
                log::info!(
                    "Authenticated: {}",
                    if auth.is_authenticated() { "Yes" } else { "No" }
                );
                log::info!("Configuration:");
                log::info!("  Install Directory: {:?}", config.install_dir);
                log::info!("  Log Level: {}", config.log_level);

                if let Ok(config_path) = Config::config_path() {
                    log::info!("Config Path: {:?}", config_path);
                }

                if let Ok(data_dir) = Config::data_dir() {
                    log::info!("Data Directory: {:?}", data_dir);
                }
            }

            Commands::Update {
                app_name,
                check_only,
            } => {
                if !auth.is_authenticated() {
                    log::error!("Error: Not authenticated. Run 'rauncher auth' first.");
                    std::process::exit(1);
                }

                let manager = GameManager::new(config, auth)?;

                if check_only {
                    log::info!("Checking for updates for {}...", app_name);
                    match manager.check_for_updates(&app_name).await {
                        Ok(Some(version)) => {
                            log::info!("✓ Update available: version {}", version);
                        }
                        Ok(None) => {
                            log::info!("✓ Game is up to date");
                        }
                        Err(e) => {
                            log::error!("Failed to check for updates: {}", e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    match manager.update_game(&app_name).await {
                        Ok(()) => log::info!("✓ Update complete!"),
                        Err(e) => {
                            log::error!("Failed to update game: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }

            Commands::CloudSave {
                app_name,
                download,
                upload,
            } => {
                if !auth.is_authenticated() {
                    log::error!("Error: Not authenticated. Run 'rauncher auth' first.");
                    std::process::exit(1);
                }

                let manager = GameManager::new(config, auth)?;

                if !download && !upload {
                    log::error!("Error: Specify --download or --upload");
                    std::process::exit(1);
                }

                if download {
                    match manager.download_cloud_saves(&app_name).await {
                        Ok(()) => {}
                        Err(e) => {
                            log::error!("Failed to download cloud saves: {}", e);
                            std::process::exit(1);
                        }
                    }
                }

                if upload {
                    match manager.upload_cloud_saves(&app_name).await {
                        Ok(()) => {}
                        Err(e) => {
                            log::error!("Failed to upload cloud saves: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }

            Commands::Gui => {
                use rauncher::gui::LauncherApp;

                let native_options = eframe::NativeOptions {
                    viewport: egui::ViewportBuilder::default()
                        .with_inner_size([1200.0, 800.0])
                        .with_min_inner_size([800.0, 600.0])
                        .with_title("R Games Launcher"),
                    ..Default::default()
                };

                if let Err(e) = eframe::run_native(
                    "R Games Launcher",
                    native_options,
                    Box::new(|cc| Ok(Box::new(LauncherApp::new(cc)))),
                ) {
                    log::error!("Failed to run GUI: {}", e);
                    std::process::exit(1);
                }
            }
        },
    }

    Ok(())
}
