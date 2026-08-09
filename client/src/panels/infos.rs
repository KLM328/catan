use catan::Game;
use eframe::egui;
use eframe::egui::Ui;
use crate::{theme, player_row};

pub(crate) fn show(ui: &mut Ui, game: &Game) {
    egui::Panel::right("info").exact_size(theme::SIDE_PANEL_W).show(ui, |ui| {
        ui.add_space(12.0);

        ui.heading("Joueurs");
        ui.add_space(8.0);

        let current = game.current_player();
        game.turn_order()
            .iter()
            .map(|&id| (id, game.get_player(id).unwrap()))
            .for_each(|(id, player)| {
                player_row(ui, player, id == current);
                ui.add_space(6.0);
            });
    });
}
