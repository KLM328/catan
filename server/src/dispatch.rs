use crate::state::GameState;
use catan::{GameError, PlayerId, Roll, Steal};
use catan_protocol::{ClientMessage, PlayerInfo, ServerMessage};

pub(crate) fn apply(
    game_state: &mut GameState,
    player_id: PlayerId,
    msg: ClientMessage,
) -> Vec<(PlayerId, ServerMessage)> {
    let mut messages = Vec::new();

    let message: Result<ServerMessage, GameError> = match msg {
        ClientMessage::BuildRoad(edge) => match game_state.game_mut().build_road(player_id, edge) {
            Ok(_) => match game_state.player_info(player_id) {
                Ok(player_info) => Ok(ServerMessage::BuildRoad(player_info, edge)),
                Err(e) => Err(e)
            },
            Err(e) => Err(e)
        },
        ClientMessage::BuildSettlement(vertex) => {
            match game_state.game_mut().build_settlement(player_id, vertex) {
                Ok(_) => match game_state.player_info(player_id) {
                    Ok(player_info) => Ok(ServerMessage::BuildSettlement(player_info, vertex)),
                    Err(e) => Err(e)
                },
                Err(e) => Err(e)
            }
        }
        ClientMessage::UpgradeCity(vertex) => {
            match game_state
                .game_mut()
                .upgrade_settlement_to_city(player_id, vertex)
            {
                Ok(_) => match game_state.player_info(player_id) {
                    Ok(player_info) => Ok(ServerMessage::UpgradeCity(player_info, vertex)),
                    Err(e) => Err(e)
                },
                Err(e) => Err(e)
            }
        }

        ClientMessage::Discard(resources) => {
            match game_state.game_mut().discard(player_id, resources) {
                Ok(_) => match game_state.player_info(player_id) {
                    Ok(player_info) => Ok(ServerMessage::Discard(player_info)),
                    Err(e) => Err(e)
                },
                Err(e) => Err(e)
            }
        }

        ClientMessage::Steal(player_option) => {
            match player_option {
                Some(victim_id) => {
                    match game_state.game().get_player(victim_id) {
                        Ok(victim) => {
                            let victim = victim.clone();
                            let resource = victim.hand().random_pick();
                            match game_state.game_mut().steal(player_id, Some(Steal::new(victim_id, resource))) {
                                Ok(_) => match game_state.player_info(player_id) {
                                    Ok(player_info) => {
                                        let victim_info = PlayerInfo::from((&victim, victim_id));
                                        Ok(ServerMessage::StealConfirmation {
                                            robber: player_info.clone(),
                                            victim: victim_info.clone(),
                                            resource,
                                        })
                                    }
                                    Err(e) => Err(e)
                                },
                                Err(e) => Err(e)
                            }
                        }
                        Err(e) => Err(e)
                    }
                },
                None => match game_state.game_mut().steal(player_id, None) {
                    Ok(_) => match game_state.player_info(player_id) {
                        Ok(player_info) => Ok(
                            ServerMessage::StealNotification {
                                robber: player_info,
                                victim: None,
                            }),
                        Err(e) => Err(e)
                    },
                    Err(e) => Err(e)
                }
            }
        }

        ClientMessage::RobberLocation(tile) => {
                match game_state.game_mut().move_robber(player_id, tile) {
                    Ok(_) => Ok(ServerMessage::NewRobberLocation(tile)),
                    Err(e) => Err(e)
                }
            }
            ClientMessage::Roll => {
                let roll = Roll::random();
                match game_state.game_mut().apply_roll(player_id, roll) {
                    Ok(outcome) => Ok(ServerMessage::Roll(roll, outcome)),
                    Err(e) => Err(e)
                }
            }
            ClientMessage::EndTurn => {
                match game_state.game_mut().next_player(player_id) {
                    Ok(_) => match game_state.player_info(player_id) {
                        Ok(player_info) => Ok(ServerMessage::NextPlayer(player_info)),
                        Err(e) => Err(e)
                    },
                    Err(e) => Err(e)
                }
            },
            ClientMessage::Join => Err(GameError::InvalidGameStatus)
        };

        match message {
            Ok(msg) => {
                match msg {
                    ServerMessage::StealConfirmation { robber, victim, resource} => {
                        for &id in game_state.senders().keys(){
                            if id == robber.id || id == victim.id {
                                messages.push((id, ServerMessage::StealConfirmation {robber : robber.clone(), victim : victim.clone(), resource : resource.clone()}));
                            } else {
                                messages.push((id, ServerMessage::StealNotification {robber : robber.clone(), victim : Some(victim.clone())}))
                            }
                        }
                    }
                    msg=> broadcast(&game_state, msg, &mut messages)
                }
            }
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
