mod app;
mod panels;
mod theme;
mod widgets;
mod game_view;

pub use app::UiAction;
pub(crate) use theme::{player_color, resource_color, terrain_color};
pub(crate) use widgets::{action_button, disc_button, draw_die, hand_over_button, player_row, player_disc, card, badge};
pub(crate) use panels::board::BuildMode;
pub(crate) use game_view::GameView;

use eframe::egui::{self};
use crate::app::CatanApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_fullscreen(true),
        ..Default::default()
    };

    eframe::run_native(
        "Catan",
        options,
        Box::new(|_cc| Ok(Box::new(CatanApp::new()))),
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
