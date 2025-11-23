use eframe::egui;
use poll_promise::Promise;
use std::sync::{Arc, Mutex};

use crate::api::{EpicClient, Game};
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
    epic_client: Arc<EpicClient>,
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
        let epic_client = EpicClient::new().unwrap();

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
            epic_client: Arc::new(epic_client),
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
        self.status_message = "Loading library...".to_string();

        let auth_manager = self.auth.lock().unwrap();
        if let Ok(token) = auth_manager.get_token() {
            let token = token.clone();
            let epic_client = Arc::clone(&self.epic_client);

            self.library_promise = Some(Promise::spawn_thread("load_library", move || {
                let rt = tokio::runtime::Runtime::new()
                    .expect("Failed to create Tokio runtime for library load");
                rt.block_on(async move { epic_client.get_games(&token).await })
            }));
        } else {
            self.status_message = "Authentication token not found.".to_string();
            self.loading_library = false;
        }
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
        // TODO: Implement real game installation
        self.status_message = format!("Installation for {} not implemented yet.", app_name);
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
                                self.handle_install(app_name);
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
