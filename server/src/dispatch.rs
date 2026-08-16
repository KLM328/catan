use tokio::sync::mpsc::Sender;
use crate::state::GameState;
use catan::{PlayerId, Roll, Steal};
use catan_protocol::{ClientMessage, ServerMessage};
use catan_protocol::ServerError;

pub(crate) fn apply(
    game_state: &mut GameState,
    player_id: PlayerId,
    msg: ClientMessage,
) -> Vec<(Sender<ServerMessage>, ServerMessage)> {
    let messages: Result<Vec<(Sender<ServerMessage>, ServerMessage)>, ServerError> = build_messages(game_state, player_id, msg);

    match messages {
        Ok(messages) => messages,
        Err(error) => {
            vec![(game_state.sender(player_id).unwrap(), ServerMessage::from(error))]
        }
    }
}

fn build_messages(
    game_state: &mut GameState,
    player_id: PlayerId,
    msg: ClientMessage,
) -> Result<Vec<(Sender<ServerMessage>, ServerMessage)>, ServerError> {
    let mut messages = Vec::new();
    if game_state.connected_players().len() < game_state.game().players().len() {
        return Err(ServerError::GamePaused);
    }
    match msg {
        ClientMessage::BuildRoad(edge) => {
            game_state.game_mut().build_road(player_id, edge)?;
            messages.extend(game_state.broadcast(ServerMessage::BuildRoad(game_state.player_info(player_id)?, edge, game_state.state())));
            messages.push((game_state.sender(player_id).unwrap(), ServerMessage::HandUpdate(game_state.game().get_player(player_id)?.hand().clone())))
        }
        ClientMessage::BuildSettlement(vertex) => {
            game_state.game_mut().build_settlement(player_id, vertex)?;
            messages.extend(game_state.broadcast(ServerMessage::BuildSettlement(game_state.player_info(player_id)?, vertex, game_state.state())));
            messages.push((game_state.sender(player_id).unwrap(), ServerMessage::HandUpdate(game_state.game().get_player(player_id)?.hand().clone())))
        }
        ClientMessage::UpgradeCity(vertex) => {
            game_state.game_mut().upgrade_settlement_to_city(player_id, vertex)?;
            messages.extend(game_state.broadcast(ServerMessage::UpgradeCity(game_state.player_info(player_id)?, vertex)));
            messages.push((game_state.sender(player_id).unwrap(), ServerMessage::HandUpdate(game_state.game().get_player(player_id)?.hand().clone())))
        }

        ClientMessage::Discard(resources) => {
            game_state.game_mut().discard(player_id, resources)?;
            messages.extend(game_state.broadcast(ServerMessage::Discard(game_state.player_info(player_id)?, game_state.state())));
            messages.push((game_state.sender(player_id).unwrap(), ServerMessage::HandUpdate(game_state.game().get_player(player_id)?.hand().clone())))
        }

        ClientMessage::Steal(player_option) => {
            let steal: Option<Steal> = match player_option {
                Some(victim_id) => {
                    let resource = game_state.game().get_player(victim_id)?.hand().random_pick();
                    Some(Steal::new(victim_id, resource))
                }
                None => None
            };
            game_state.game_mut().steal(player_id, &steal)?;
            match steal {
                None => messages.extend(game_state.broadcast(ServerMessage::Steal { robber: game_state.player_info(player_id)?, victim: None, state : game_state.state() })),
                Some(steal) => {
                    messages.extend(game_state.broadcast(ServerMessage::Steal { robber: game_state.player_info(player_id)?, victim: Some(game_state.player_info(steal.victim())?), state : game_state.state() }));
                    for id in game_state.connected_players(){
                        if steal.victim() == id || id == player_id {
                            messages.push((game_state.sender(id).unwrap(), ServerMessage::HandUpdate(game_state.game().get_player(id)?.hand().clone())))
                        }
                    }
                },
            }

        }

        ClientMessage::RobberLocation(tile) => {
            game_state.game_mut().move_robber(player_id, tile)?;
            messages.extend(game_state.broadcast(ServerMessage::NewRobberLocation(tile, game_state.state())));
        }
        ClientMessage::Roll => {
            let roll = Roll::random();
            game_state.game_mut().apply_roll(player_id, roll)?;
            messages.extend(game_state.broadcast(ServerMessage::Roll(roll, game_state.state())));

            for player_id in game_state.connected_players() {
                messages.push((game_state.sender(player_id).unwrap(), ServerMessage::HandUpdate(game_state.game().get_player(player_id)?.hand().clone())))
            }
        }
        ClientMessage::EndTurn => {
            game_state.game_mut().next_player(player_id)?;
            messages.extend(game_state.broadcast(ServerMessage::NextPlayer(game_state.state())));
        }
        ClientMessage::StartGame => {
            if player_id == game_state.game().sorted_player()[0].0 {
                let rolls: Vec<(PlayerId, Roll)> = game_state.game().sorted_player().iter().map(|&(id, _)| (id, Roll::random())).collect();
                game_state.game_mut().set_players_order(&rolls)?;
                let tiles = if game_state.random_board() { game_state.game().scenario().shuffled_terrains() } else { game_state.game().scenario().terrains().to_vec() };
                game_state.game_mut().start(&tiles)?;
                messages.extend(game_state.broadcast(ServerMessage::StartGame(rolls)));
                for player_id in game_state.connected_players() {
                    messages.push((game_state.sender(player_id).unwrap(), ServerMessage::from((game_state.game(), player_id))))
                }
            } else {
                return Err(ServerError::NotTheHost);
            }
        }

        ClientMessage::Join { .. } => return Err(ServerError::InvalidMessageType),
        ClientMessage::Sync => {
            messages.push((game_state.sender(player_id).unwrap(), ServerMessage::from((game_state.game(), player_id))));
        }
    };
    Ok(messages)
}
