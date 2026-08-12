use crate::state::GameState;
use catan::{GameError, PlayerId, Roll, Steal};
use catan_protocol::{ClientMessage, PlayerInfo, ServerMessage};

pub(crate) fn apply(
    game_state: &mut GameState,
    player_id: PlayerId,
    msg: ClientMessage,
) -> Vec<(PlayerId, ServerMessage)> {
    let mut messages = Vec::new();
    match game_state.player_info(player_id) {
        Ok(player_info) => {
            let _action = {
                match msg {
                    ClientMessage::BuildRoad(edge) => {
                        match game_state.game_mut().build_road(player_id, edge) {
                            Ok(_) => broadcast(
                                game_state,
                                ServerMessage::BuildRoad(player_info, edge),
                                &mut messages,
                            ),
                            Err(e) => messages.push((player_id, ServerMessage::from(e))),
                        }
                    }
                    ClientMessage::BuildSettlement(vertex) => {
                        match game_state.game_mut().build_settlement(player_id, vertex) {
                            Ok(_) => broadcast(
                                game_state,
                                ServerMessage::BuildSettlement(player_info, vertex),
                                &mut messages,
                            ),
                            Err(e) => messages.push((player_id, ServerMessage::from(e))),
                        }
                    }
                    ClientMessage::UpgradeCity(vertex) => {
                        match game_state
                            .game_mut()
                            .upgrade_settlement_to_city(player_id, vertex)
                        {
                            Ok(_) => broadcast(
                                game_state,
                                ServerMessage::UpgradeCity(player_info, vertex),
                                &mut messages,
                            ),
                            Err(e) => messages.push((player_id, ServerMessage::from(e))),
                        }
                    }
                    ClientMessage::Discard(resources) => {
                        match game_state.game_mut().discard(player_id, resources) {
                            Ok(_) => broadcast(
                                game_state,
                                ServerMessage::Discard(player_info),
                                &mut messages,
                            ),
                            Err(e) => messages.push((player_id, ServerMessage::from(e))),
                        }
                    }
                    ClientMessage::Steal(player_option) => {
                        match player_option {
                            Some(victim_id) => {
                                match game_state.game().get_player(victim_id) {
                                    Ok(victim) => {
                                        let victim_info = PlayerInfo::from((victim, victim_id));
                                        let resource = victim.hand().random_pick();
                                        match game_state.game_mut().steal(player_id, Some(Steal::new(victim_id, resource))) {
                                            Ok(_) => {
                                                for &id in game_state.senders().keys() {
                                                    let msg = if id == player_id || id == victim_id {
                                                        ServerMessage::StealConfirmation { robber : player_info.clone(), victim : victim_info.clone(), resource}
                                                    } else {
                                                        ServerMessage::StealNotification { robber : player_info.clone(), victim : Some(victim_info.clone()) }
                                                    };
                                                    messages.push((id, msg));
                                                }
                                            }
                                            Err(e) => messages.push((player_id, ServerMessage::from(e))),
                                        }
                                    }
                                    Err(e) => messages.push((player_id, ServerMessage::from(e))),
                                }
                            }
                            None => {
                                match game_state.game_mut().steal(player_id, None) {
                                    Ok(_) => {broadcast(&game_state, ServerMessage::StealNotification {robber : player_info.clone(), victim : None}, &mut messages)}
                                    Err(e) => messages.push((player_id, ServerMessage::from(e))),
                                }
                            }
                        }
                    }
                    ClientMessage::RobberLocation(tile) => {
                        match game_state.game_mut().move_robber(player_id, tile) {
                            Ok(_) => broadcast(
                                game_state,
                                ServerMessage::NewRobberLocation(tile),
                                &mut messages,
                            ),
                            Err(e) => messages.push((player_id, ServerMessage::from(e))),
                        }
                    }
                    ClientMessage::Roll => {
                        let roll = Roll::random();
                        match game_state.game_mut().apply_roll(player_id, roll) {
                            Ok(outcome) => broadcast(
                                game_state,
                                ServerMessage::Roll(roll, outcome),
                                &mut messages,
                            ),
                            Err(e) => messages.push((player_id, ServerMessage::from(e))),
                        }
                    }
                    ClientMessage::EndTurn => match game_state.game_mut().next_player(player_id) {
                        Ok(_) => broadcast(
                            game_state,
                            ServerMessage::NextPlayer(player_info),
                            &mut messages,
                        ),
                        Err(e) => messages.push((player_id, ServerMessage::from(e))),
                    },
                    ClientMessage::Join => messages
                        .push((player_id, ServerMessage::from(GameError::InvalidGameStatus))),
                }
            };
        }
        Err(e) => messages.push((player_id, ServerMessage::from(e)))
    }

    messages
}

fn broadcast(
    game_state: &GameState,
    message: ServerMessage,
    messages: &mut Vec<(PlayerId, ServerMessage)>,
) {
    for id in 0..game_state.game().players().len() {
        messages.push((PlayerId::new(id), message.clone()))
    }
}
