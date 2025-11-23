use egui::{Align, Layout, RichText};
use poll_promise::Promise;
use std::time::{Duration, Instant};

use crate::api::{DeviceAuthResponse, EpicClient};
use crate::auth::{AuthManager, AuthToken};
use crate::Result;

enum AuthState {
    Idle,
    RequestingDeviceAuth,
    Polling {
        device_code: String,
        last_poll: Instant,
        attempts: u32,
    },
}

pub struct AuthView {
    auth_status: String,
    state: AuthState,
    verification_url: Option<String>,
    user_code: Option<String>,
    device_auth_promise: Option<Promise<Result<DeviceAuthResponse>>>,
    poll_promise: Option<Promise<Result<Option<AuthToken>>>>,
}

impl Default for AuthView {
    fn default() -> Self {
        Self {
            auth_status: String::new(),
            state: AuthState::Idle,
            verification_url: None,
            user_code: None,
            device_auth_promise: None,
            poll_promise: None,
        }
    }
}

impl AuthView {
    pub fn ui(&mut self, ui: &mut egui::Ui, auth: &mut AuthManager) -> bool {
        // Handle device auth promise
        if let Some(promise) = &self.device_auth_promise {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(device_auth) => {
                        self.verification_url = Some(device_auth.verification_uri_complete.clone());
                        self.user_code = Some(device_auth.user_code.clone());
                        self.state = AuthState::Polling {
                            device_code: device_auth.device_code.clone(),
                            last_poll: Instant::now() - Duration::from_secs(10), // Poll immediately
                            attempts: 0,
                        };
                    }
                    Err(e) => {
                        self.auth_status = format!("Failed to start authentication: {}", e);
                        self.state = AuthState::Idle;
                        self.verification_url = None;
                        self.user_code = None;
                    }
                }
                self.device_auth_promise = None;
            }
        }

        // Handle polling state - extract values first to avoid borrow checker issues
        let polling_info = if let AuthState::Polling {
            device_code,
            last_poll,
            attempts,
        } = &self.state
        {
            Some((device_code.clone(), *last_poll, *attempts))
        } else {
            None
        };

        if let Some((device_code, last_poll, attempts)) = polling_info {
            // Check if poll promise is ready
            if let Some(promise) = &self.poll_promise {
                if let Some(result) = promise.ready() {
                    match result {
                        Ok(Some(token)) => {
                            // Successfully got token, save it
                            if let Err(e) = auth.set_token(token.clone()) {
                                self.auth_status = format!("Error saving token: {}", e);
                                self.state = AuthState::Idle;
                            } else {
                                self.auth_status =
                                    "✓ Successfully authenticated with Epic Games!".to_string();
                                self.state = AuthState::Idle;
                                self.poll_promise = None;
                                return true; // Signal successful login
                            }
                        }
                        Ok(None) => {
                            // Still waiting, continue polling
                            let new_attempts = attempts + 1;
                            if new_attempts >= 120 {
                                // Timeout after 10 minutes (120 * 5 seconds)
                                self.auth_status =
                                    "Authentication timed out. Please try again.".to_string();
                                self.state = AuthState::Idle;
                                self.verification_url = None;
                                self.user_code = None;
                            } else {
                                self.state = AuthState::Polling {
                                    device_code: device_code.clone(),
                                    last_poll: Instant::now(),
                                    attempts: new_attempts,
                                };
                            }
                        }
                        Err(e) => {
                            self.auth_status = format!("Authentication failed: {}", e);
                            self.state = AuthState::Idle;
                            self.verification_url = None;
                            self.user_code = None;
                        }
                    }
                    self.poll_promise = None;
                }
            }

            // Start new poll if needed
            if self.poll_promise.is_none() && last_poll.elapsed() >= Duration::from_secs(5) {
                let device_code_clone = device_code.clone();
                let promise = Promise::spawn_thread("poll_auth", move || {
                    tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(async move {
                            let client = EpicClient::new()?;
                            client.poll_for_token(&device_code_clone).await
                        })
                });
                self.poll_promise = Some(promise);
            }
        }

        ui.vertical_centered(|ui| {
            ui.add_space(80.0);

            // Enhanced Epic Games branding
            ui.heading(
                RichText::new("EPIC GAMES STORE")
                    .size(36.0)
                    .strong()
                    .color(egui::Color32::WHITE),
            );
            ui.add_space(15.0);
            ui.label(
                RichText::new("Sign in to your account")
                    .size(18.0)
                    .color(egui::Color32::from_rgb(180, 180, 190)),
            );

            ui.add_space(50.0);

            // Center the content
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.set_max_width(500.0);

                match &self.state {
                    AuthState::Idle => {
                        // Show enhanced login button with Epic blue
                        let button = egui::Button::new(
                            RichText::new("Sign In with Epic Games")
                                .size(18.0)
                                .strong()
                                .color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(0, 121, 214))
                        .min_size(egui::Vec2::new(280.0, 50.0));
                        
                        if ui.add(button).clicked() {
                            self.start_authentication();
                        }

                        ui.add_space(30.0);

                        // Enhanced instructions
                        ui.label(
                            RichText::new("Click the button above to authenticate with Epic Games")
                                .size(15.0)
                                .color(egui::Color32::from_rgb(160, 160, 170)),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("You'll receive a code to enter in your browser")
                                .size(15.0)
                                .color(egui::Color32::from_rgb(160, 160, 170)),
                        );
                    }
                    AuthState::RequestingDeviceAuth => {
                        ui.spinner();
                        ui.add_space(10.0);
                        ui.label("Initializing authentication...");

                        ui.add_space(20.0);

                        if ui.button("Cancel").clicked() {
                            self.cancel_authentication();
                        }
                    }
                    AuthState::Polling { attempts, .. } => {
                        // Show authentication in progress with enhanced styling
                        ui.heading(
                            RichText::new("⏳ Authentication in Progress")
                                .size(24.0)
                                .strong()
                                .color(egui::Color32::WHITE),
                        );
                        ui.add_space(25.0);

                        if let (Some(url), Some(code)) = (&self.verification_url, &self.user_code) {
                            ui.label(
                                RichText::new("Please complete authentication in your browser:")
                                    .size(17.0)
                                    .color(egui::Color32::from_rgb(220, 220, 230)),
                            );
                            ui.add_space(20.0);

                            // Display verification URL in an enhanced frame
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(32, 34, 40))
                                .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 121, 214)))
                                .rounding(egui::Rounding::same(6.0))
                                .inner_margin(20.0)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new("URL:")
                                                .strong()
                                                .size(15.0)
                                                .color(egui::Color32::from_rgb(180, 180, 190)),
                                        );
                                        ui.add_space(8.0);
                                        let _ = ui.selectable_label(
                                            false,
                                            RichText::new(url)
                                                .monospace()
                                                .size(14.0)
                                                .color(egui::Color32::from_rgb(100, 170, 230)),
                                        );
                                    });

                                    ui.add_space(12.0);

                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new("Code:")
                                                .strong()
                                                .size(15.0)
                                                .color(egui::Color32::from_rgb(180, 180, 190)),
                                        );
                                        ui.add_space(8.0);
                                        let _ = ui.selectable_label(
                                            false,
                                            RichText::new(code)
                                                .monospace()
                                                .size(22.0)
                                                .strong()
                                                .color(egui::Color32::WHITE),
                                        );
                                    });
                                });

                            ui.add_space(20.0);

                            // Enhanced "Open in Browser" button
                            let browser_button = egui::Button::new(
                                RichText::new("🌐 Open in Browser")
                                    .size(16.0)
                                    .strong()
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(egui::Color32::from_rgb(0, 121, 214))
                            .min_size(egui::Vec2::new(200.0, 42.0));
                            
                            if ui.add(browser_button).clicked() {
                                let _ = webbrowser::open(url);
                            }

                            ui.add_space(15.0);
                            ui.label(
                                RichText::new(format!(
                                    "Waiting for you to complete authentication... (attempt {}/120)",
                                    attempts + 1
                                ))
                                .size(15.0)
                                .color(egui::Color32::from_rgb(100, 170, 230)),
                            );
                        }

                        ui.add_space(25.0);

                        if ui.button(RichText::new("Cancel").size(14.0)).clicked() {
                            self.cancel_authentication();
                        }
                    }
                }

                ui.add_space(20.0);

                // Status message
                if !self.auth_status.is_empty() {
                    ui.colored_label(
                        if self.auth_status.starts_with("✓") {
                            egui::Color32::GREEN
                        } else {
                            egui::Color32::RED
                        },
                        &self.auth_status,
                    );
                }
            });
        });

        false
    }

    fn start_authentication(&mut self) {
        self.state = AuthState::RequestingDeviceAuth;
        self.auth_status = String::new();
        self.verification_url = None;
        self.user_code = None;

        // Spawn thread to run async device auth request
        let promise = Promise::spawn_thread("device_auth", || {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async move {
                    let client = EpicClient::new()?;
                    client.request_device_auth().await
                })
        });

        self.device_auth_promise = Some(promise);
    }

    fn cancel_authentication(&mut self) {
        self.state = AuthState::Idle;
        self.device_auth_promise = None;
        self.poll_promise = None;
        self.verification_url = None;
        self.user_code = None;
        self.auth_status = "Authentication cancelled".to_string();
    }
}
