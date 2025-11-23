use egui::RichText;

#[derive(Clone, PartialEq)]
pub enum GameFilter {
    All,
    Installed,
}

pub struct SearchBar;

impl SearchBar {
    pub fn show(
        ui: &mut egui::Ui,
        search_query: &mut String,
        filter: &mut GameFilter,
    ) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Library").size(20.0).strong());
            ui.add_space(20.0);

            // Search box with enhanced styling
            ui.label(RichText::new("🔍").size(16.0));
            ui.add_space(5.0);
            let search_edit = egui::TextEdit::singleline(search_query)
                .hint_text("Search games...")
                .desired_width(250.0);
            ui.add(search_edit);

            ui.add_space(20.0);

            // Filters with Epic-style buttons
            let all_selected = *filter == GameFilter::All;
            if ui
                .selectable_label(all_selected, RichText::new("All Games").size(14.0))
                .clicked()
            {
                *filter = GameFilter::All;
            }
            
            ui.add_space(5.0);
            
            let installed_selected = *filter == GameFilter::Installed;
            if ui
                .selectable_label(installed_selected, RichText::new("Installed").size(14.0))
                .clicked()
            {
                *filter = GameFilter::Installed;
            }
        });
    }
}
