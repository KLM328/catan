use eframe::egui;
use eframe::egui::{Align2, Ui};
use catan::{Cost, Game, GameStatus};
use crate::{action_button, player_color, theme, BuildMode};
use crate::game_view::GameView;

pub(crate) fn show(ui : &mut Ui, game : &GameView, build_mode: &mut BuildMode) {

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
                        if action_button(ui, mode, *build_mode, cost, ok, color).clicked()
                            && ok
                        {
                            *build_mode = if *build_mode == mode {
                                BuildMode::None
                            } else {
                                mode
                            };
                        }
                    }
                });
        });
}