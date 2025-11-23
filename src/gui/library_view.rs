use egui::{Color32, RichText, ScrollArea};
use std::sync::{Arc, Mutex};

use crate::api::Game;
use crate::games::InstalledGame;
use super::components::{GameCard, GameCardAction, SearchBar, GameFilter};

#[derive(Clone)]
pub struct LibraryView {
    filter: GameFilter,
    search_query: String,
    installing_games: Arc<Mutex<Vec<String>>>,
}

impl Default for LibraryView {
    fn default() -> Self {
        Self {
            filter: GameFilter::All,
            search_query: String::new(),
            installing_games: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl LibraryView {
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        library_games: &[Game],
        installed_games: &[InstalledGame],
    ) -> Option<LibraryAction> {
        let mut action = None;

        // Top bar with search and filters using the SearchBar component
        SearchBar::show(ui, &mut self.search_query, &mut self.filter);

        ui.separator();
        ui.add_space(15.0);

        // Game grid with enhanced layout
        ScrollArea::vertical().show(ui, |ui| {
            let available_width = ui.available_width();
            let card_width = 280.0; // Slightly larger cards
            let card_spacing = 15.0; // More spacing between cards
            let cards_per_row = (available_width / (card_width + card_spacing)).floor().max(1.0) as usize;

            let games_to_show: Vec<_> = match self.filter {
                GameFilter::All => library_games
                    .iter()
                    .filter(|g| {
                        self.search_query.is_empty()
                            || g.app_title
                                .to_lowercase()
                                .contains(&self.search_query.to_lowercase())
                    })
                    .collect(),
                GameFilter::Installed => library_games
                    .iter()
                    .filter(|g| {
                        installed_games.iter().any(|ig| ig.app_name == g.app_name)
                            && (self.search_query.is_empty()
                                || g.app_title
                                    .to_lowercase()
                                    .contains(&self.search_query.to_lowercase()))
                    })
                    .collect(),
            };

            if games_to_show.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.label(
                        RichText::new("No games found")
                            .size(18.0)
                            .color(Color32::GRAY),
                    );
                    if self.search_query.is_empty() {
                        ui.label(
                            RichText::new("Your library is empty or not yet loaded")
                                .color(Color32::GRAY),
                        );
                    }
                });
            } else {
                // Display games in a grid with enhanced spacing
                for row_games in games_to_show.chunks(cards_per_row) {
                    ui.horizontal(|ui| {
                        for game in row_games {
                            let is_installed = installed_games
                                .iter()
                                .any(|ig| ig.app_name == game.app_name);
                            let is_installing = self
                                .installing_games
                                .lock()
                                .unwrap()
                                .contains(&game.app_name);
                            
                            if let Some(game_action) = GameCard::show(ui, game, is_installed, is_installing) {
                                action = Some(match game_action {
                                    GameCardAction::Install(name) => LibraryAction::Install(name),
                                    GameCardAction::Launch(name) => LibraryAction::Launch(name),
                                    GameCardAction::Uninstall(name) => LibraryAction::Uninstall(name),
                                });
                            }
                            ui.add_space(card_spacing);
                        }
                    });
                    ui.add_space(15.0);
                }
            }
        });

        action
    }

    pub fn mark_installation_complete(&mut self, app_name: &str) {
        self.installing_games
            .lock()
            .unwrap()
            .retain(|name| name != app_name);
    }
}

pub enum LibraryAction {
    Install(String),
    Launch(String),
    Uninstall(String),
}
