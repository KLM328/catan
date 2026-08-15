mod app;
mod panels;
mod theme;
mod widgets;
mod game_view;
mod scenes;
mod dispatch;
mod network;

pub(crate) use theme::{player_color, resource_color, terrain_color};
pub(crate) use widgets::{action_button, disc_button, draw_die, hand_over_button, player_row, player_disc, card, badge};
pub(crate) use panels::board::BuildMode;
pub(crate) use game_view::GameView;


use eframe::egui::{self};
use tokio::sync::mpsc;
use catan_protocol::{ClientMessage, ServerMessage};
use crate::app::{CatanApp, UiState, AppState};



fn main() -> eframe::Result {
    let (to_server_tx, to_server_rx) = mpsc::channel::<ClientMessage>(32);
    let (to_ui_tx, to_ui_rx) = mpsc::channel::<ServerMessage>(64);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_fullscreen(true),
        ..Default::default()
    };
    eframe::run_native(
        "Catan",
        options,
        Box::new(move |cc| {
            let ctx = cc.egui_ctx.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(network::run("127.0.0.1:8080", to_server_rx, to_ui_tx, ctx));
            });
            Ok(Box::new(CatanApp::new(to_server_tx, to_ui_rx)))
        }),
    )
}



fn cell(ui: &mut egui::Ui, text: egui::RichText) {
    ui.with_layout(
        egui::Layout::centered_and_justified(egui::Direction::TopDown),
        |ui| {
            ui.label(text);
        },
    );
}
