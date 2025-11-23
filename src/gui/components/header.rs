use egui::RichText;

pub struct Header;

impl Header {
    pub fn show(ui: &mut egui::Ui, is_authenticated: bool, on_logout: &mut bool) {
        ui.horizontal(|ui| {
            // Logo/Title with Epic Games-inspired styling
            ui.heading(
                RichText::new("R Games Launcher")
                    .size(22.0)
                    .strong()
                    .color(egui::Color32::WHITE),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if is_authenticated {
                    if ui
                        .button(RichText::new("Logout").size(14.0))
                        .clicked()
                    {
                        *on_logout = true;
                    }
                }
            });
        });
    }
}
