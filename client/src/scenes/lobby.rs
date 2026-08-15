use catan_protocol::{ClientMessage, PlayerInfo};
use crate::app::UiState;
use crate::panels::message;

pub(crate) fn show(ui: &mut egui::Ui, ui_state: &UiState,players: &Vec<PlayerInfo>, messages: &mut Vec<ClientMessage>) {
    let mut grid = egui::Grid::new("lobby");
    grid.show(ui, |ui| {
        ui.heading("Joueurs en ligne");
        for player in players {
            ui.label(player.id.to_string());
        }
    });
    if ui.button("Start").clicked() {
        messages.push(ClientMessage::StartGame);
    };
    
    message::show(ui, ui_state);
}