use crate::{theme, UiAction, hand_over_button, player_color};
use catan::{GameStatus};
use eframe::egui;
use eframe::egui::{Align2, Ui};
use crate::game_view::GameView;

pub(crate) fn show(ui: &mut Ui, game: &GameView) -> Vec<UiAction> {
    let mut actions = Vec::new();

    egui::Area::new(egui::Id::new("next_player"))
        .anchor(Align2::RIGHT_TOP, egui::vec2(- theme::SIDE_PANEL_W -24.0, 24.0))
        .show(ui.ctx(), |ui| {
            if let GameStatus::PlayingActions = game.status() {
                let next_player = game.get_player(game.get_next_player()).unwrap();
                if hand_over_button(
                    ui,
                    player_color(next_player),
                    next_player.color().color_name(),
                )
                .clicked()
                {
                    actions.push(UiAction::NextPlayer);
                }
            }
        });

    actions
}
