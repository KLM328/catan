use crate::dispatch::apply;
use crate::state::GameState;
use catan::{Player, PlayerId};
use catan_protocol::{ClientMessage, ServerMessage};
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

    let (player_id, mut rx) = match join_phase(&line, &game_state, &mut writer).await {
        Some(pair) => pair,
        None => return,
    };

    spawn(async move {
        while let Some(msg) = rx.recv().await {
            let mut json = serde_json::to_string(&msg).unwrap();
            json.push('\n');
            writer.write_all(json.as_bytes()).await.unwrap();
        }
    });

    let (tx, msg) = {
        let g = game_state.lock().unwrap();
        (
            g.senders().get(&player_id).unwrap().clone(),
            ServerMessage::from((g.game(), player_id)),
        )
    };
    tx.send(msg).await.unwrap();

    loop {
        line.clear();
        match buf_reader.read_line(&mut line).await {
            Ok(0) => {
                println!("{addr} s'est déconnecté");
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
                                .filter_map(|(id, msg)| g.senders().get(&id).map(|tx| (tx.clone(), msg)))
                                .collect()
                        };

                        for (tx, msg) in outgoing {
                            let _ = tx.send(msg).await;
                        }
                    }
                    Err(_) => {
                        let msg = ServerMessage::Error("On attend un message de type ClientMessage".to_string());
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
    line: &String,
    game_state: &Arc<Mutex<GameState>>,
    writer: &mut OwnedWriteHalf,
) -> Option<(PlayerId, Receiver<ServerMessage>)> {
    match serde_json::from_str(line) {
        Ok(ClientMessage::Join) => {
            let player_result = {
                let mut g = game_state.lock().unwrap();
                let color = g.game().next_player_color().unwrap();
                g.game_mut().add_player(Player::new(color))
            };

            if let Ok(player_id) = player_result {
                let (tx, rx) = mpsc::channel(32);
                {
                    let mut g = game_state.lock().unwrap();
                    g.senders_mut().insert(player_id, tx);
                }
                Some((player_id, rx))
            } else {
                let msg = ServerMessage::from(player_result.unwrap_err());
                let json = serde_json::to_string(&msg).unwrap();
                writer.write_all(json.as_bytes()).await.unwrap();
                None
            }
        }

        _ => {
            let msg = ServerMessage::Error("On attend un message de type Join".to_string());
            let json = serde_json::to_string(&msg).unwrap();
            writer.write_all(json.as_bytes()).await.unwrap();
            None
        }
    }
}
