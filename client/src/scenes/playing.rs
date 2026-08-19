use egui::Ui;
use catan_protocol::ClientMessage;
use crate::UiState;
use crate::game_view::GameView;
use crate::panels::{actions, board, dice, end, hand, infos, message, next_player, pause, rolls};

pub(crate) fn show(ui : &mut Ui, game : &GameView, ui_state : &mut UiState, actions : &mut Vec<ClientMessage>) {
    infos::show(ui, game);

    actions.extend(board::show(
        ui,
        game,
        ui_state
    ));

    actions.extend(dice::show(ui, game, ui_state));
    actions.extend(next_player::show(ui, game));
    actions.extend(hand::show(ui, game, ui_state));

    actions::show(ui, game, ui_state);

    end::show(ui, game);

    message::show(ui, ui_state);

    rolls::show(ui, game.players(), ui_state);

    pause::show(ui, game);

}
