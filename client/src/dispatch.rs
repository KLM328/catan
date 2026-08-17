use crate::AppState;
use crate::app::UiState;
use crate::game_view::GameView;
use catan::{Building, BuildingKind};
use catan_protocol::{ClientMessage, ServerMessage};
use crate::panels::board::BuildMode;

pub(crate) fn apply(
    app_state: &mut AppState,
    ui_state: &mut UiState,
    incoming_message: ServerMessage,
    messages: &mut Vec<ClientMessage>,
    now : f64
) {
    match incoming_message {
        ServerMessage::BuildRoad(player, edge, state) => {
            if let AppState::Playing(view) = app_state {
                view.set_road(edge, player.id());
                view.update_player(player);
                view.set_status(state.status);
                view.set_current_turn(state.current_turn);
                ui_state.switch_buimd_mode(BuildMode::None)

            } else {
                messages.push(ClientMessage::Sync)
            }
        }
        ServerMessage::BuildSettlement(player, vertex, state) => {
            if let AppState::Playing(view) = app_state {
                view.set_building(vertex, Building::new(BuildingKind::Settlement, player.id()));
                view.update_player(player);
                view.set_status(state.status);
                view.set_current_turn(state.current_turn);
                ui_state.switch_buimd_mode(BuildMode::None)

            } else {
                messages.push(ClientMessage::Sync);
            }
        }
        ServerMessage::UpgradeCity(player, vertex) => {
            if let AppState::Playing(view) = app_state {
                view.set_building(vertex, Building::new(BuildingKind::City, player.id()));
                view.update_player(player);
                ui_state.switch_buimd_mode(BuildMode::None)

            } else {
                messages.push(ClientMessage::Sync)
            }
        }
        ServerMessage::Steal { robber, victim, state } => {
            if let AppState::Playing(view) = app_state {
                view.update_player(robber);
                view.set_status(state.status);
                view.set_current_turn(state.current_turn);
                if let Some(victim) = victim {
                    view.update_player(victim);
                }
            } else {
                messages.push(ClientMessage::Sync)
            }
        }

        ServerMessage::Discard(player, state) => {
            if let AppState::Playing(view) = app_state {
                view.update_player(player);
                view.set_status(state.status);
                view.set_current_turn(state.current_turn);
            } else {
                messages.push(ClientMessage::Sync)
            }
        }
        ServerMessage::NewRobberLocation(tile, state) => {
            if let AppState::Playing(view) = app_state {
                view.set_robber(tile);
                view.set_status(state.status);
                view.set_current_turn(state.current_turn)
            } else {
                messages.push(ClientMessage::Sync)
            }
        }
        ServerMessage::Roll(roll, state) => {
            if let AppState::Playing(view) = app_state {
                ui_state.set_last_roll(roll);
                view.set_status(state.status);
                view.set_current_turn(state.current_turn)
            } else {
                messages.push(ClientMessage::Sync)
            }
        }
        ServerMessage::NextPlayer(state) => {
            if let AppState::Playing(view) = app_state {
                view.set_status(state.status);
                view.set_current_turn(state.current_turn);
                ui_state.switch_buimd_mode(BuildMode::None)
            } else {
                messages.push(ClientMessage::Sync)
            }
        }
        ServerMessage::GameEnd(state) => {
            if let AppState::Playing(view) = app_state {
                view.set_status(state.status);
                view.set_current_turn(state.current_turn)
            }
        }
        ServerMessage::PlayerJoined(player) => {
            if let AppState::Lobby { players } = app_state {
                players.push(player);
            }
        }
        ServerMessage::Leave(player) => {
            ui_state.set_message(format!("Le joueur {} à quitté la partie", player.value()), now);
        }
        ServerMessage::Sync(game) => {
            *app_state = AppState::Playing(GameView::from(game))
        },
        ServerMessage::LobbyView {
            player_id : _,
            players,
            scenario : _,
        } => {
            *app_state = AppState::Lobby { players };
        }
        ServerMessage::StartGame(rolls) => {
            ui_state.set_rolls_display(rolls, now);
        }
        ServerMessage::Error(error) => {
            ui_state.set_message(error.to_string(), now);
        }
        ServerMessage::JoinGame(_) => {}
        ServerMessage::PauseGame => {
            if let AppState::Playing(view) = app_state {
                *app_state = AppState::Paused(view.clone())
            } else {
                messages.push(ClientMessage::Sync)
            }
        }
        ServerMessage::ResumeGame => {
            if let AppState::Paused(view) = app_state {
                *app_state = AppState::Playing(view.clone())
            } else {
                messages.push(ClientMessage::Sync)
            }
        }
        ServerMessage::HandUpdate(hand) => {
            if let AppState::Paused(view) = app_state {
                view.set_hand(hand)
            } else {
                messages.push(ClientMessage::Sync)
            }
        }
    }
}
