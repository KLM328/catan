use catan::{Board, GameError, GameStatus};
use catan_protocol::{ClientMessage, ServerMessage};
use crate::app::UiState;
use crate::AppState;
use crate::game_view::GameView;

pub(crate) fn apply(app_state : &mut AppState, ui_state: &mut UiState, incoming_message: ServerMessage, messages : &mut Vec<ClientMessage>){

    match incoming_message {
        ServerMessage::BuildRoad(player, edge) => {
            if let AppState::Playing(game) = app_state {
                if matches!(game.status(), GameStatus::FirstPlacementRoad | GameStatus::SecondPlacementRoad) && let Err(e) =game.board_mut().place_road(Board::can_place_road_during_placement, edge, player.id()){
                    ui_state.set_message(e.to_string())
                } else if matches!(game.status(), GameStatus::PlayingActions) && let Err(e) =game.board_mut().place_road(Board::can_place_road_during_placement, edge, player.id()){
                    ui_state.set_message(e.to_string())
                } else {
                    ui_state.set_message(GameError::InvalidGameStatus.to_string())
                }
            }
            else {
                messages.push(ClientMessage::Sync)
            }
        }
        ServerMessage::BuildSettlement(player, vertex) => {
            if let AppState::Playing(game) = app_state {
                if matches!(game.status(), GameStatus::FirstPlacementSettlement | GameStatus::SecondPlacementSettlement) && let Err(e) =game.board_mut().place_settlement(Board::can_place_settlement_during_placement, vertex, player.id()){
                    ui_state.set_message(e.to_string())
                } else if matches!(game.status(), GameStatus::PlayingActions) && let Err(e) =game.board_mut().place_settlement(Board::can_place_settlement_during_playing, vertex, player.id()){
                    ui_state.set_message(e.to_string())
                } else {
                    ui_state.set_message(GameError::InvalidGameStatus.to_string())
                }
            }
            else {
                messages.push(ClientMessage::Sync)
            }
        }
        ServerMessage::UpgradeCity(player, vertex) => {
            if let AppState::Playing(game) = app_state {
                if let Err(e) =game.board_mut().upgrade_settlement_to_city(vertex, player.id()){
                    ui_state.set_message(e.to_string())
                }
            }
            else {
                messages.push(ClientMessage::Sync)
            }
        }
        ServerMessage::StealNotification { .. } => {
            todo!()
        }
        ServerMessage::StealConfirmation { .. } => {
            todo!()
        }
        ServerMessage::Discard(_) => {
            todo!()
        }
        ServerMessage::NewRobberLocation(tile) => {
            if let AppState::Playing(game) = app_state {
                if let Err(e) =game.board_mut().move_robber(tile){
                    ui_state.set_message(e.to_string())
                }
            }
            else {
                messages.push(ClientMessage::Sync)
            }
        }
        ServerMessage::Roll(roll, _) => {
            if let AppState::Playing(game) = app_state {
                ui_state.set_last_roll(roll);
            }
            else {
                messages.push(ClientMessage::Sync)
            }
        }
        ServerMessage::NextPlayer(player) => {
            if let AppState::Playing(game) = app_state {
                if player.id() != game.next_player() {
                    messages.push(ClientMessage::Sync)
                }
            }
        }
        ServerMessage::GameEnd { winner } => {
            if let AppState::Playing(game) = app_state {
                game.win(winner);
            }
        }
        ServerMessage::PlayerJoined(player) => {
            if let AppState::Lobby { players } = app_state {
                players.push(player);
            }
        }
        ServerMessage::Leave(player) => {
            ui_state.set_message(format!("Le joueur {} à quitté la partie", player.value()));

        }
        ServerMessage::Sync(game ) => {
            *app_state = AppState::Playing(GameView::from(game))
        }
        ServerMessage::LobbyView { player_id, players, scenario } => {
            *app_state = AppState::Lobby {players};

        }
        ServerMessage::StartGame(game) => {
            todo!("afficher les lancés de dés")
        }
        ServerMessage::Error(error) => {
            ui_state.set_message(error.to_string());
        }
        ServerMessage::JoinGame(token) => {
        }
        ServerMessage::PauseGame => {
            if let AppState::Playing(game) = app_state {
                *app_state = AppState::Paused(game.clone())
            } else {
                messages.push(ClientMessage::Sync)
            }
        }
        ServerMessage::ResumeGame => {
            if let AppState::Paused(game) = app_state {
                *app_state = AppState::Playing(game.clone())
            } else {
                messages.push(ClientMessage::Sync)
            }
        }
    }
}