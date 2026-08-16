use crate::GameView;
use crate::{end_turn_button, theme};
use catan::GameStatus;
use catan_protocol::ClientMessage;
use eframe::egui;
use eframe::egui::{Align2, Ui};

pub(crate) fn show(ui: &mut Ui, game: &GameView) -> Vec<ClientMessage> {
    let mut actions = Vec::new();

    egui::Area::new(egui::Id::new("next_player"))
        .anchor(
            Align2::RIGHT_TOP,
            egui::vec2(-theme::SIDE_PANEL_W - 24.0, 24.0),
        )
        .show(ui.ctx(), |ui| {
            if let GameStatus::PlayingActions = game.status()
                && end_turn_button(ui).clicked()
            {
                actions.push(ClientMessage::EndTurn);
            }
        });

    actions
}
