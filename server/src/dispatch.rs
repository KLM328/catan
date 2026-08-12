use crate::state::GameState;
use catan::{PlayerId, Roll, Steal};
use catan_protocol::{ClientMessage, PlayerInfo, ServerMessage};
use catan_protocol::ServerError;

pub(crate) fn apply(
    game_state: &mut GameState,
    player_id: PlayerId,
    msg: ClientMessage,
) -> Vec<(PlayerId, ServerMessage)> {

    let messages: Result<Vec<(PlayerId, ServerMessage)>, ServerError> = build_messages(game_state, player_id, msg);

    match messages {
        Ok(messages) => messages,
        Err(error) => {
            let mut messages = Vec::new();
            messages.push((player_id, ServerMessage::from(error)));
            messages
        }
    }
}

fn broadcast(
    game_state: &GameState,
    message: ServerMessage,
    messages: &mut Vec<(PlayerId, ServerMessage)>,
) {
    for &id in game_state.senders().keys() {
        messages.push((id, message.clone()))
    }
}

fn build_messages(
    game_state: &mut GameState,
    player_id: PlayerId,
    msg: ClientMessage,
) -> Result<Vec<(PlayerId, ServerMessage)>, ServerError> {
    let mut messages = Vec::new();
    if game_state.senders().len() < game_state.game().players().len() {
        return Err(ServerError::GamePaused)
    }
    match msg {
        ClientMessage::BuildRoad(edge) => {
            game_state.game_mut().build_road(player_id, edge)?;
            broadcast(game_state, ServerMessage::BuildRoad(game_state.player_info(player_id)?, edge), &mut messages);
        }
        ClientMessage::BuildSettlement(vertex) => {
            game_state.game_mut().build_settlement(player_id, vertex)?;
            broadcast(game_state, ServerMessage::BuildSettlement(game_state.player_info(player_id)?, vertex), &mut messages);
        }
        ClientMessage::UpgradeCity(vertex) => {
            game_state.game_mut().upgrade_settlement_to_city(player_id, vertex)?;
            broadcast(game_state, ServerMessage::UpgradeCity(game_state.player_info(player_id)?, vertex), &mut messages)
        }

        ClientMessage::Discard(resources) => {
            game_state.game_mut().discard(player_id, resources)?;
            broadcast(game_state, ServerMessage::Discard(game_state.player_info(player_id)?), &mut messages)
        }

        ClientMessage::Steal(player_option) => match player_option {
            Some(victim_id) => {
                let resource = game_state.game().get_player(victim_id)?.hand().random_pick();
                game_state.game_mut().steal(player_id, Some(Steal::new(victim_id, resource)))?;
                for &id in game_state.senders().keys() {
                    if id == victim_id || id == player_id {
                        messages.push((id, ServerMessage::StealConfirmation {
                            robber: game_state.player_info(player_id)?,
                            victim: PlayerInfo::from((game_state.game().get_player(victim_id)?, victim_id)),
                            resource,
                        }))
                    }
                    else {
                        messages.push((id, ServerMessage::StealNotification {
                            robber: game_state.player_info(player_id)?,
                            victim: Some(PlayerInfo::from((game_state.game().get_player(victim_id)?, victim_id))),
                        }))
                    }
                }
            }
            None => {
                game_state.game_mut().steal(player_id, None)?;
                broadcast(game_state, ServerMessage::StealNotification {robber: game_state.player_info(player_id)?, victim: None}, &mut messages)
            }
        },

        ClientMessage::RobberLocation(tile) => {
            game_state.game_mut().move_robber(player_id, tile)?;
            broadcast(game_state, ServerMessage::NewRobberLocation(tile), &mut messages)
        }
        ClientMessage::Roll => {
            let roll = Roll::random();
            let outcome = game_state.game_mut().apply_roll(player_id, roll)?;
            broadcast(game_state, ServerMessage::Roll(roll, outcome), &mut messages)
        }
        ClientMessage::EndTurn => {
            game_state.game_mut().next_player(player_id)?;
            broadcast(game_state, ServerMessage::NextPlayer(game_state.player_info(player_id)?), &mut messages)
        }
        ClientMessage::StartGame => {
            if player_id == PlayerId::new(0) {
                let rolls = game_state.game().players().iter().map(|_| Roll::random()).collect();
                game_state.game_mut().set_players_order(&rolls)?;
                let tiles = if game_state.random_board() {game_state.game().scenario().shuffled_terrains()} else {game_state.game().scenario().terrains().to_vec()};
                game_state.game_mut().start(&tiles)?;
                broadcast(game_state, ServerMessage::StartGame(rolls), &mut messages);
                for &player_id in game_state.senders().keys() {
                    messages.push((player_id, ServerMessage::from((game_state.game(), player_id)) ))
                }
            }
            else {
                return Err(ServerError::NotTheHost)
            }
        }

        ClientMessage::Join {..}  => return Err(ServerError::InvalidMessageType)
    };
    Ok(messages)
}
