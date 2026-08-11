use catan::{GameError, PlayerId, Roll, Steal};
use catan_protocol::{ClientMessage, ServerMessage};
use crate::state::GameState;

pub(crate) fn apply(game_state: &mut GameState, player_id: PlayerId, msg: ClientMessage) -> Vec<(PlayerId, ServerMessage)> {
    let mut messages = Vec::new();


    let _action = {
        match msg {
            ClientMessage::BuildRoad(edge) => game_state.game_mut().build_road(player_id, edge),
            ClientMessage::BuildSettlement(vertex) => {
                game_state.game_mut().build_settlement(player_id, vertex)
            }
            ClientMessage::UpgradeCity(vertex) => {
                game_state.game_mut().upgrade_settlement_to_city(player_id, vertex)
            }
            ClientMessage::Discard(resources) => game_state.game_mut().discard(player_id, resources),
            ClientMessage::Steal(player_option) => {
                let steal_opt = match player_option {
                    Some(victim) => Some(Steal::new(
                        victim,
                        game_state.game_mut().get_player(victim).unwrap().hand().random_pick(),
                    )),
                    None => None,
                };
                game_state.game_mut().steal(player_id, steal_opt)
            }
            ClientMessage::RobberLocation(tile) => {game_state.game_mut().move_robber(player_id, tile)}
            ClientMessage::Roll => {game_state.game_mut().apply_roll(player_id, Roll::random()).map(|_| ())}
            ClientMessage::EndTurn => {game_state.game_mut().next_player(player_id)}
            ClientMessage::Join => {Err(GameError::InvalidGameStatus)}
        }
    };

    messages
}
