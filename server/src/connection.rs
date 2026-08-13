use crate::dispatch::apply;
use crate::state::GameState;
use catan::{GameStatus, PlayerId};
use catan_protocol::ServerError;
use catan_protocol::{ClientMessage, PlayerInfo, ServerMessage, Token};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::spawn;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

pub(crate) async fn handle(
    socket: TcpStream,
    addr: std::net::SocketAddr,
    game_state: Arc<Mutex<GameState>>,
) {
    let (reader, mut writer) = socket.into_split();
    let mut buf_reader = BufReader::take(BufReader::new(reader), 2048);
    let mut line = String::new();

    match buf_reader.read_line(&mut line).await {
        Ok(0) => {
            println!("{addr} s'est déconnecté avant de rejoindre");
            return;
        }
        Ok(_) => {}
        Err(e) => {
            eprintln!("Erreur de lecture depuis {addr} : {e}");
            return;
        }
    }

    let (player_id, mut rx, tx) = match join_phase(&line, &game_state, &mut writer).await {
        Some(triplet) => triplet,
        None => return,
    };

    spawn(async move {
        while let Some(msg) = rx.recv().await {
            let mut json = serde_json::to_string(&msg).unwrap();
            json.push('\n');
            writer.write_all(json.as_bytes()).await.unwrap();
        }
    });

    loop {
        line.clear();
        match buf_reader.read_line(&mut line).await {
            Ok(0) => {
                let outgoing: Vec<(Sender<ServerMessage>, ServerMessage)> = {
                    let mut outgoing = Vec::new();
                    let mut g = game_state.lock().unwrap();
                    g.unregister(player_id);

                    outgoing.extend(
                        g.senders()
                            .values()
                            .map(|sender| (sender.clone(), ServerMessage::Leave(player_id))),
                    );
                    if !matches!(g.game().status(), GameStatus::Starting) && g.is_paused()
                    {
                        outgoing.extend(
                            g.senders()
                                .values()
                                .map(|sender| (sender.clone(), ServerMessage::PauseGame)),
                        );
                    }
                    outgoing
                };

                for (sender, message) in outgoing {
                    sender.send(message).await.unwrap();
                }
                println!("{player_id} s'est déconnecté");

                return;
            }
            Ok(_) => {
                let message = serde_json::from_str::<ClientMessage>(&line);
                match message {
                    Ok(message) => {
                        let outgoing: Vec<(Sender<ServerMessage>, ServerMessage)> = {
                            let mut g = game_state.lock().unwrap();
                            apply(&mut g, player_id, message)
                                .into_iter()
                                .filter_map(|(id, msg)| {
                                    g.senders().get(&id).map(|tx| (tx.clone(), msg))
                                })
                                .collect()
                        };

                        for (tx, msg) in outgoing {
                            let _ = tx.send(msg).await;
                        }
                    }
                    Err(_) => {
                        let msg = ServerMessage::from(ServerError::InvalidMessageFormat);
                        let _ = tx.send(msg).await;
                    }
                }
            }
            Err(e) => {
                eprintln!("Erreur de lecture depuis {addr} : {e}");
                return;
            }
        }
    }
}

async fn join_phase(
    line: &str,
    game_state: &Arc<Mutex<GameState>>,
    writer: &mut OwnedWriteHalf,
) -> Option<(PlayerId, Receiver<ServerMessage>, Sender<ServerMessage>)> {
    match serde_json::from_str(line) {
        Ok(incoming_message) => {
            let result = {
                let mut g = game_state.lock().unwrap();
                generate_message(incoming_message, &mut g)
            };

            match result {
                Ok((player_id, rx, tx, outgoing)) => {
                    for (sender, msg) in outgoing {
                        let _ = sender.send(msg).await;
                    }
                    Some((player_id, rx, tx))
                }
                Err(e) => {
                    let msg = ServerMessage::from(e);
                    let mut json = serde_json::to_string(&msg).unwrap();
                    json.push('\n');
                    writer.write_all(json.as_bytes()).await.unwrap();
                    None
                }
            }
        }
        Err(_) => {
            let msg = ServerMessage::from(ServerError::InvalidMessageFormat);
            let mut json = serde_json::to_string(&msg).unwrap();
            json.push('\n');
            writer.write_all(json.as_bytes()).await.unwrap();
            None
        }
    }


}

fn join_game(
    game_state: &mut GameState,
    token: Option<Token>,
) -> Result<(PlayerId, Token), ServerError> {
    match token {
        Some(token) => match game_state.player_by_token(token) {
            Some(player_id) => {
                if game_state.connected_players().contains(&player_id) {
                    Err(ServerError::PlayerIsAlreadyConnected)
                } else {
                    Ok((player_id, token))
                }
            }
            None => Err(ServerError::InvalidToken),
        },
        None => {
            game_state.new_player()
        }
    }
}

fn generate_message(
    incoming_message: ClientMessage,
    game_state: &mut GameState,
) -> Result<(PlayerId, Receiver<ServerMessage>, Sender<ServerMessage>, Vec<(Sender<ServerMessage>, ServerMessage)>), ServerError> {
    let mut outgoing = Vec::new();

    match incoming_message {
        ClientMessage::Join { token } => {
            let (player_id, token) = join_game(game_state, token)?;
            let (tx, rx) = mpsc::channel(32);
            game_state.register(player_id, tx.clone())?;
            outgoing.push((tx.clone(), ServerMessage::JoinGame(token)));
            outgoing.push((tx.clone(), ServerMessage::from((game_state.game(), player_id))));
            outgoing.extend(game_state.senders().iter().filter(|&(&p, _)| p != player_id).map(|(_, sender)| {
                (
                    sender.clone(),
                    ServerMessage::PlayerJoined(PlayerInfo::from((
                        game_state.game().get_player(player_id).unwrap(),
                        player_id,
                    ))),
                )
            }));
            if !(matches!(game_state.game().status(), GameStatus::Starting) || game_state.is_paused()) {
                outgoing.extend(
                    game_state
                        .senders()
                        .values()
                        .map(|sender| (sender.clone(), ServerMessage::ResumeGame)),
                );
            }

            Ok((player_id, rx, tx, outgoing))
        }

        _ => Err(ServerError::InvalidMessageType),
    }
}
