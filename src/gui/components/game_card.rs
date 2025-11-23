use egui::{Color32, RichText, Vec2};

use crate::api::Game;

pub struct GameCard;

impl GameCard {
    pub fn show(
        ui: &mut egui::Ui,
        game: &Game,
        is_installed: bool,
        is_installing: bool,
    ) -> Option<GameCardAction> {
        let mut action = None;

        egui::Frame::none()
            .fill(Color32::from_rgb(28, 28, 32))
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(45, 45, 50)))
            .rounding(egui::Rounding::same(6.0))
            .inner_margin(egui::Margin::same(0.0))
            .show(ui, |ui| {
                ui.set_min_size(Vec2::new(280.0, 340.0));
                ui.set_max_size(Vec2::new(280.0, 340.0));

                ui.vertical(|ui| {
                    // Game image placeholder with gradient effect
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::new(280.0, 200.0), egui::Sense::hover());
                    
                    // Create a gradient background for the image placeholder
                    let painter = ui.painter();
                    let image_rounding = egui::Rounding {
                        nw: 6.0,
                        ne: 6.0,
                        sw: 0.0,
                        se: 0.0,
                    };
                    
                    painter.rect_filled(
                        rect,
                        image_rounding,
                        Color32::from_rgb(45, 50, 65),
                    );
                    
                    // Add a subtle overlay gradient
                    if response.hovered() {
                        painter.rect_filled(
                            rect,
                            image_rounding,
                            Color32::from_rgba_premultiplied(0, 121, 214, 20),
                        );
                    }

                    // Game title on image
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        &game.app_title,
                        egui::FontId::proportional(16.0),
                        Color32::WHITE,
                    );

                    ui.add_space(15.0);

                    // Content area with padding
                    ui.add_space(0.0);
                    ui.horizontal(|ui| {
                        ui.add_space(15.0);
                        ui.vertical(|ui| {
                            // Game title
                            ui.label(
                                RichText::new(&game.app_title)
                                    .size(16.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            );

                            ui.add_space(5.0);

                            // Version info
                            ui.label(
                                RichText::new(format!("v{}", &game.app_version))
                                    .size(12.0)
                                    .color(Color32::from_rgb(160, 160, 160)),
                            );

                            ui.add_space(15.0);

                            // Action buttons
                            ui.horizontal(|ui| {
                                if is_installed {
                                    // Play button - Epic blue
                                    let play_button = egui::Button::new(
                                        RichText::new("▶ Play")
                                            .size(15.0)
                                            .strong()
                                            .color(Color32::WHITE),
                                    )
                                    .fill(Color32::from_rgb(0, 121, 214))
                                    .min_size(Vec2::new(120.0, 36.0));
                                    
                                    if ui.add(play_button).clicked() {
                                        action = Some(GameCardAction::Launch(game.app_name.clone()));
                                    }
                                    
                                    ui.add_space(5.0);
                                    
                                    // Uninstall button
                                    let uninstall_button = egui::Button::new(
                                        RichText::new("Uninstall").size(13.0),
                                    )
                                    .fill(Color32::from_rgb(60, 60, 65))
                                    .min_size(Vec2::new(100.0, 36.0));
                                    
                                    if ui.add(uninstall_button).clicked() {
                                        action = Some(GameCardAction::Uninstall(game.app_name.clone()));
                                    }
                                } else if is_installing {
                                    ui.add_enabled_ui(false, |ui| {
                                        let installing_button = egui::Button::new(
                                            RichText::new("⏳ Installing...")
                                                .size(15.0)
                                                .color(Color32::from_rgb(180, 180, 180)),
                                        )
                                        .fill(Color32::from_rgb(50, 50, 55))
                                        .min_size(Vec2::new(200.0, 36.0));
                                        
                                        let _ = ui.add(installing_button);
                                    });
                                } else {
                                    // Install button - Epic blue
                                    let install_button = egui::Button::new(
                                        RichText::new("Get")
                                            .size(15.0)
                                            .strong()
                                            .color(Color32::WHITE),
                                    )
                                    .fill(Color32::from_rgb(0, 121, 214))
                                    .min_size(Vec2::new(200.0, 36.0));
                                    
                                    if ui.add(install_button).clicked() {
                                        action = Some(GameCardAction::Install(game.app_name.clone()));
                                    }
                                }
                            });
                        });
                    });
                });
            });

        action
    }
}

pub enum GameCardAction {
    Install(String),
    Launch(String),
    Uninstall(String),
}
