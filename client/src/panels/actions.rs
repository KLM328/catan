use eframe::egui;
use eframe::egui::{Align2, Ui};
use catan::{Cost, GameStatus};
use crate::{action_button, player_color, theme, BuildMode};
use crate::app::UiState;
use crate::game_view::GameView;

pub(crate) fn show(ui : &mut Ui, game : &GameView, ui_state: &mut UiState) {

    egui::Area::new(egui::Id::new("actions"))
        .anchor(Align2::RIGHT_BOTTOM, egui::vec2(-theme::SIDE_PANEL_W - theme::BUTTON_W_H * 2.0 - 42.0, -24.0))
        .show(ui.ctx(), |ui| if matches!(game.status(), GameStatus::PlayingActions) {
                let player = game.get_player(game.current_player()).unwrap();
                let color = player_color(player);

                ui.horizontal(|ui| {
                    // ui.spacing_mut().item_spacing.x = 10.0;
                    for (mode, cost) in [
                        (BuildMode::Road, &Cost::ROAD),
                        (BuildMode::Settlement, &Cost::SETTLEMENT),
                        (BuildMode::City, &Cost::CITY),
                    ] {
                        let ok = game.can_pay(cost).is_ok();
                        if action_button(ui, mode, ui_state.build_mode(), cost, ok, color).clicked()
                            && ok
                        {
                            ui_state.switch_buimd_mode(mode);
                        }
                    }
                });
        });
}