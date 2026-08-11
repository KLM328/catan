use std::sync::Mutex;
use catan::{GameError, PlayerId, Roll, Steal};
use catan_protocol::{ClientMessage, ServerMessage};
use crate::state::GameState;

pub(crate) async fn dispatch(line : &String, player_id : PlayerId, game_state : &Mutex<GameState>) -> Vec<(PlayerId, ServerMessage)>{
    let mut messages = Vec::new();
    let msg: ClientMessage = match serde_json::from_str(line) {
        Ok(msg) => msg,
        Err(_) => {
            let msg = ServerMessage::Error("Erreur de format".to_string());
            messages.push((player_id, msg));
            return messages;
        }
    };

    let _action = {
        let mut g = game_state.lock().unwrap();
        match msg {
            ClientMessage::BuildRoad(edge) => g.game_mut().build_road(player_id, edge),
            ClientMessage::BuildSettlement(vertex) => {
                g.game_mut().build_settlement(player_id, vertex)
            }
            ClientMessage::UpgradeCity(vertex) => {
                g.game_mut().upgrade_settlement_to_city(player_id, vertex)
            }
            ClientMessage::Discard(resources) => g.game_mut().discard(player_id, resources),
            ClientMessage::Steal(player_option) => {
                let steal_opt = match player_option {
                    Some(victim) => Some(Steal::new(
                        victim,
                        g.game_mut().get_player(victim).unwrap().hand().random_pick(),
                    )),
                    None => None,
                };
                g.game_mut().steal(player_id, steal_opt)
            }
            ClientMessage::RobberLocation(tile) => {g.game_mut().move_robber(player_id, tile)}
            ClientMessage::Roll => {g.game_mut().apply_roll(player_id, Roll::random()).map(|_| ())}
            ClientMessage::EndTurn => {g.game_mut().next_player(player_id)}
            ClientMessage::Join => {Err(GameError::InvalidGameStatus)}
        }
    };

    messages
}
