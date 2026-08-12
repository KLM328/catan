use crate::state::GameState;
use catan::{GameError, PlayerId, Roll, Steal};
use catan_protocol::{ClientMessage, PlayerInfo, ServerMessage};

pub(crate) fn apply(
    game_state: &mut GameState,
    player_id: PlayerId,
    msg: ClientMessage,
) -> Vec<(PlayerId, ServerMessage)> {
    let mut messages = Vec::new();

    let message: Result<ServerMessage, GameError> = build_message(game_state, player_id, msg);

    match message {
        Ok(msg) => match msg {
            ServerMessage::StealConfirmation {
                robber,
                victim,
                resource,
            } => {
                for &id in game_state.senders().keys() {
                    if id == robber.id || id == victim.id {
                        messages.push((
                            id,
                            ServerMessage::StealConfirmation {
                                robber: robber.clone(),
                                victim: victim.clone(),
                                resource: resource.clone(),
                            },
                        ));
                    } else {
                        messages.push((
                            id,
                            ServerMessage::StealNotification {
                                robber: robber.clone(),
                                victim: Some(victim.clone()),
                            },
                        ))
                    }
                }
            }
            msg => broadcast(&game_state, msg, &mut messages),
        },
        Err(e) => messages.push((player_id, ServerMessage::from(e))),
    }

    messages
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

fn build_message(
    game_state: &mut GameState,
    player_id: PlayerId,
    msg: ClientMessage,
) -> Result<ServerMessage, GameError> {
    match msg {
        ClientMessage::BuildRoad(edge) => {
            game_state.game_mut().build_road(player_id, edge)?;
            Ok(ServerMessage::BuildRoad(
                game_state.player_info(player_id)?,
                edge,
            ))
        }
        ClientMessage::BuildSettlement(vertex) => {
            game_state.game_mut().build_settlement(player_id, vertex)?;
            Ok(ServerMessage::BuildSettlement(
                game_state.player_info(player_id)?,
                vertex,
            ))
        }
        ClientMessage::UpgradeCity(vertex) => {
            game_state
                .game_mut()
                .upgrade_settlement_to_city(player_id, vertex)?;
            Ok(ServerMessage::UpgradeCity(
                game_state.player_info(player_id)?,
                vertex,
            ))
        }

        ClientMessage::Discard(resources) => {
            game_state.game_mut().discard(player_id, resources)?;
            Ok(ServerMessage::Discard(game_state.player_info(player_id)?))
        }

        ClientMessage::Steal(player_option) => match player_option {
            Some(victim_id) => {
                let resource = game_state.game().get_player(victim_id)?.hand().random_pick();
                game_state
                    .game_mut()
                    .steal(player_id, Some(Steal::new(victim_id, resource)))?;
                Ok(ServerMessage::StealConfirmation {
                    robber: game_state.player_info(player_id)?,
                    victim: PlayerInfo::from((game_state.game().get_player(victim_id)?, victim_id)),
                    resource,
                })
            }
            None => {
                game_state.game_mut().steal(player_id, None)?;
                Ok(ServerMessage::StealNotification {
                    robber: game_state.player_info(player_id)?,
                    victim: None,
                })
            }
        },

        ClientMessage::RobberLocation(tile) => {
            game_state.game_mut().move_robber(player_id, tile)?;
            Ok(ServerMessage::NewRobberLocation(tile))
        }
        ClientMessage::Roll => {
            let roll = Roll::random();
            let outcome = game_state.game_mut().apply_roll(player_id, roll)?;
            Ok(ServerMessage::Roll(roll, outcome))
        }
        ClientMessage::EndTurn => {
            game_state.game_mut().next_player(player_id)?;
            Ok(ServerMessage::NextPlayer(
                game_state.player_info(player_id)?,
            ))
        }

        ClientMessage::Join => Err(GameError::InvalidGameStatus),
    }
}
