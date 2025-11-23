use egui::RichText;

pub struct StatusBar;

impl StatusBar {
    pub fn show(ui: &mut egui::Ui, message: &str, on_clear: &mut bool) {
        if !message.is_empty() {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(message)
                        .size(13.0)
                        .color(if message.contains("✓") || message.contains("success") {
                            egui::Color32::from_rgb(76, 175, 80)
                        } else if message.contains("Failed") || message.contains("Error") {
                            egui::Color32::from_rgb(244, 67, 54)
                        } else {
                            egui::Color32::from_rgb(200, 200, 200)
                        }),
                );
                if ui.button(RichText::new("Clear").size(12.0)).clicked() {
                    *on_clear = true;
                }
            });
        }
    }
}
