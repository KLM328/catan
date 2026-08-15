use catan_protocol::ServerMessage;
use crate::app::UiState;
use crate::AppState;
use crate::game_view::GameView;

pub(crate) fn apply(app_state : &mut AppState, ui_state: &mut UiState, msg : ServerMessage){

    match msg {
        ServerMessage::BuildRoad(_, _) => {}
        ServerMessage::BuildSettlement(_, _) => {}
        ServerMessage::UpgradeCity(_, _) => {}
        ServerMessage::StealNotification { .. } => {}
        ServerMessage::StealConfirmation { .. } => {}
        ServerMessage::Discard(_) => {}
        ServerMessage::NewRobberLocation(_) => {}
        ServerMessage::Roll(_, _) => {}
        ServerMessage::NextPlayer(_) => {}
        ServerMessage::GameEnd { .. } => {}
        ServerMessage::PlayerJoined(player) => {
            if let AppState::Lobby { players } = app_state {
                players.push(player);
            }
        }
        ServerMessage::Leave(_) => {}
        ServerMessage::Sync(game ) => {
            *app_state = AppState::Playing(GameView::from(game))
        }
        ServerMessage::LobbyView { player_id, players, scenario } => {
            *app_state = AppState::Lobby {players};

        }
        ServerMessage::StartGame(_) => {}
        ServerMessage::Error(error) => {
            ui_state.set_message(error.to_string());
        }
        ServerMessage::JoinGame(token) => {
        }
        ServerMessage::PauseGame => {}
        ServerMessage::ResumeGame => {}
    }
}