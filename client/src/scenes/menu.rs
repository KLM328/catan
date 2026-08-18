use catan_protocol::ClientMessage;
use crate::app::AppState;

pub(crate) fn show(ui : &mut egui::Ui, app_state : &mut AppState, messages : &mut [ClientMessage]) {
    ui.label("Menu");
    if ui.button("Join").clicked() {
    }
}