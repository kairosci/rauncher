use eframe::egui;
use poll_promise::Promise;
use std::sync::{Arc, Mutex};

use crate::api::Game;
use crate::auth::AuthManager;
use crate::config::Config;
use crate::games::{GameManager, InstalledGame};
use crate::Result;

use super::auth_view::AuthView;
use super::library_view::{LibraryAction, LibraryView};
use super::styles;
use super::components::{Header, StatusBar};

enum AppState {
    Login,
    Library,
}

pub struct LauncherApp {
    state: AppState,
    auth: Arc<Mutex<AuthManager>>,
    config: Arc<Config>,
    auth_view: AuthView,
    library_view: LibraryView,
    library_games: Vec<Game>,
    installed_games: Vec<InstalledGame>,
    status_message: String,
    loading_library: bool,
    library_promise: Option<Promise<Result<Vec<Game>>>>,
}

impl LauncherApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        styles::setup_custom_style(&cc.egui_ctx);

        let config = Config::load().unwrap_or_default();
        let auth = AuthManager::new().unwrap_or_default();

        // Check if already authenticated
        let is_authenticated = auth.is_authenticated();

        Self {
            state: if is_authenticated {
                AppState::Library
            } else {
                AppState::Login
            },
            auth: Arc::new(Mutex::new(auth)),
            config: Arc::new(config),
            auth_view: AuthView::default(),
            library_view: LibraryView::default(),
            library_games: Vec::new(),
            installed_games: Vec::new(),
            status_message: String::new(),
            loading_library: false,
            library_promise: None,
        }
    }

    fn handle_login(&mut self) {
        self.state = AppState::Library;
        self.load_library();
        self.load_installed_games();
    }

    fn load_library(&mut self) {
        if self.loading_library {
            return;
        }

        self.loading_library = true;
        let _auth = Arc::clone(&self.auth);
        let _config = Arc::clone(&self.config);

        // Create a demo library for now since Epic API integration is not complete
        self.library_games = vec![
            Game {
                app_name: "demo_game_1".to_string(),
                app_title: "Demo Game 1".to_string(),
                app_version: "1.0.0".to_string(),
                install_path: None,
            },
            Game {
                app_name: "demo_game_2".to_string(),
                app_title: "Epic Adventure".to_string(),
                app_version: "2.1.0".to_string(),
                install_path: None,
            },
            Game {
                app_name: "demo_game_3".to_string(),
                app_title: "Racing Challenge".to_string(),
                app_version: "1.5.2".to_string(),
                install_path: None,
            },
            Game {
                app_name: "demo_game_4".to_string(),
                app_title: "Strategy Master".to_string(),
                app_version: "3.0.1".to_string(),
                install_path: None,
            },
            Game {
                app_name: "demo_game_5".to_string(),
                app_title: "Space Shooter".to_string(),
                app_version: "1.2.0".to_string(),
                install_path: None,
            },
        ];
        self.loading_library = false;
        
    }

    fn load_installed_games(&mut self) {
        if let Ok(manager) =
            GameManager::new((*self.config).clone(), (*self.auth.lock().unwrap()).clone())
        {
            if let Ok(games) = manager.list_installed() {
                self.installed_games = games;
            }
        }
    }

    fn handle_install(&mut self, app_name: String) {
        self.status_message = format!("Installing {}...", app_name);

        // Find the game in our library to get proper title
        let game_title = self
            .library_games
            .iter()
            .find(|g| g.app_name == app_name)
            .map(|g| g.app_title.clone())
            .unwrap_or_else(|| format!("Game: {}", app_name));

        let game_version = self
            .library_games
            .iter()
            .find(|g| g.app_name == app_name)
            .map(|g| g.app_version.clone())
            .unwrap_or_else(|| "1.0.0".to_string());

        // For demo purposes, we'll create a mock installation
        let config = Arc::clone(&self.config);
        let app_name_clone = app_name.clone();

        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(2));

            // Create the installation directory
            let install_path = config.install_dir.join(&app_name_clone);
            if let Err(e) = std::fs::create_dir_all(&install_path) {
                eprintln!("Failed to create install directory: {}", e);
                return;
            }

            // Create a demo installed game entry
            let game = InstalledGame {
                app_name: app_name_clone.clone(),
                app_title: game_title,
                app_version: game_version,
                install_path: install_path.clone(),
                executable: "game.sh".to_string(),
            };

            // Create a simple demo executable script
            let executable_path = install_path.join("game.sh");
            let script_content = format!(
                "#!/bin/bash\necho 'Launching {}'\necho 'This is a demo game executable'\n",
                game.app_title
            );
            if let Err(e) = std::fs::write(&executable_path, script_content) {
                eprintln!("Failed to create demo executable: {}", e);
            }

            // Make it executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = std::fs::metadata(&executable_path) {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o755);
                    let _ = std::fs::set_permissions(&executable_path, perms);
                }
            }

            // Save the installation record
            if let Err(e) = game.save(&config) {
                eprintln!("Failed to save game installation: {}", e);
            }
        });
    }

    fn handle_launch(&mut self, app_name: String) {
        let config = (*self.config).clone();
        let auth = (*self.auth.lock().unwrap()).clone();

        match GameManager::new(config, auth) {
            Ok(manager) => match manager.launch_game(&app_name) {
                Ok(()) => {
                    self.status_message = format!("Launched {}", app_name);
                }
                Err(e) => {
                    self.status_message = format!("Failed to launch {}: {}", app_name, e);
                }
            },
            Err(e) => {
                self.status_message = format!("Error: {}", e);
            }
        }
    }

    fn handle_uninstall(&mut self, app_name: String) {
        let config = (*self.config).clone();
        let auth = (*self.auth.lock().unwrap()).clone();

        match GameManager::new(config, auth) {
            Ok(manager) => match manager.uninstall_game(&app_name) {
                Ok(()) => {
                    self.status_message = format!("Uninstalled {}", app_name);
                    self.load_installed_games();
                }
                Err(e) => {
                    self.status_message = format!("Failed to uninstall {}: {}", app_name, e);
                }
            },
            Err(e) => {
                self.status_message = format!("Error: {}", e);
            }
        }
    }
}

impl eframe::App for LauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for library loading completion
        if let Some(promise) = &self.library_promise {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(games) => {
                        self.library_games = games.clone();
                        self.status_message = "Library loaded successfully".to_string();
                    }
                    Err(e) => {
                        self.status_message = format!("Failed to load library: {}", e);
                    }
                }
                self.loading_library = false;
                self.library_promise = None;
            }
        }

        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::none()
                .fill(egui::Color32::from_rgb(22, 24, 28))
                .inner_margin(egui::Margin::symmetric(20.0, 15.0)))
            .show(ctx, |ui| {
                let mut logout_requested = false;
                let is_authenticated = matches!(self.state, AppState::Library);
                Header::show(ui, is_authenticated, &mut logout_requested);

                if logout_requested {
                    if let Ok(mut auth) = self.auth.lock() {
                        let _ = auth.logout();
                    }
                    self.state = AppState::Login;
                    self.library_games.clear();
                    self.installed_games.clear();
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.state {
                AppState::Login => {
                    if self.auth_view.ui(ui, &mut self.auth.lock().unwrap()) {
                        self.handle_login();
                    }
                }
                AppState::Library => {
                    if let Some(action) =
                        self.library_view
                            .ui(ui, &self.library_games, &self.installed_games)
                    {
                        match action {
                            LibraryAction::Install(app_name) => {
                                self.handle_install(app_name.clone());
                                // Mark installation complete after delay
                                let mut view = self.library_view.clone();
                                let app_name_clone = app_name.clone();
                                std::thread::spawn(move || {
                                    std::thread::sleep(std::time::Duration::from_secs(3));
                                    view.mark_installation_complete(&app_name_clone);
                                });
                            }
                            LibraryAction::Launch(app_name) => {
                                self.handle_launch(app_name);
                            }
                            LibraryAction::Uninstall(app_name) => {
                                self.handle_uninstall(app_name);
                            }
                        }
                    }
                }
            }

            // Status bar at bottom using StatusBar component
            let mut clear_status = false;
            StatusBar::show(ui, &self.status_message, &mut clear_status);
            if clear_status {
                self.status_message.clear();
            }
        });

        // Request repaint for animations/updates
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
