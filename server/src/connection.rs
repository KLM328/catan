use crate::dispatch::apply;
use crate::state::GameState;
use catan::{PlayerId};
use catan_protocol::{GameInfo, ServerError};
use catan_protocol::{ClientMessage, ServerMessage, Token};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, Take};
use tokio::net::tcp::{OwnedReadHalf};
use tokio::net::TcpStream;
use tokio::spawn;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Sender};
use crate::Games;

pub(crate) async fn handle(
    socket: TcpStream,
    addr: std::net::SocketAddr,
    games: Games,
) {
    let (reader, mut writer) = socket.into_split();
    let (tx, mut rx) = mpsc::channel::<ServerMessage>(32);

    const MAX_LINE: u64 = 2048;

    let mut buf_reader : Take<BufReader<OwnedReadHalf>> = BufReader::new(reader).take(MAX_LINE);
    let mut line = String::new();

    spawn(async move {
        while let Some(msg) = rx.recv().await {
            let mut json = serde_json::to_string(&msg).unwrap();
            json.push('\n');
            if writer.write_all(json.as_bytes()).await.is_err(){
                break
            }
        }
    });
    let games_info = {
        let mut games_info = Vec::new();
        let games = games.lock().unwrap();
        for (&game_id, game_state) in games.iter() {
            let game_state = game_state.lock().unwrap();
            games_info.push(GameInfo::new(game_id, game_state.game().scenario().clone(), game_state.connected_players().iter().count()))
        }
        games_info.sort_by_key(|game_info| game_info.id);
        games_info
    };
    send(vec![(tx.clone(), ServerMessage::GameList(games_info))]).await;


    let (game_state, player_id) = loop {
        line.clear();
        match buf_reader.read_line(&mut line).await {
            Ok(0) => {
                println!("{addr} s'est déconnecté avant de rejoindre");
                return;
            }
            Ok(_) => {
                match serde_json::from_str::<ClientMessage>(&line) {
                    Ok(incoming_message) => {
                        if let Some((game_state, player_id)) = join_phase(incoming_message, &games, tx.clone()).await {
                            break (game_state, player_id);
                        } else {
                            continue
                        }
                    }
                    Err(_) => {
                        send(vec![(tx.clone(), ServerMessage::from(ServerError::InvalidMessageFormat))]).await;
                    }
                }
            }
            Err(e) => {
                eprintln!("Erreur de lecture depuis {addr} : {e}");
                return;
            }
        }
    };

    session(&mut buf_reader, &game_state, player_id, &tx, addr).await;

    let outgoing = {let mut game_state = game_state.lock().unwrap();
        quit_game(&mut game_state, player_id)
    };
    send(outgoing).await;

}

async fn join_phase(msg : ClientMessage, games : &Games, tx : Sender<ServerMessage>) -> Option<(Arc<Mutex<GameState>>, PlayerId)>{
    match msg {
        ClientMessage::Join {token, game_id} => {
            let game_state = {
                let g = games.lock().unwrap();
                g.get(&game_id).cloned()
            };

            match game_state {
                Some(game_state) => {
                    let (player_id, outgoing) = {let mut game_state = game_state.lock().unwrap();
                        join_game(&mut game_state, token, tx.clone())
                    };
                    send(outgoing).await;
                    match player_id {
                        Some(player_id) => Some((game_state, player_id)),
                         None => None
                    }

                }
                None => {
                    send(vec![(tx.clone(), ServerMessage::from(ServerError::GameNotFound))]).await;
                    None
                }
            }

        }
        _ => {
            send(vec![(tx.clone(), ServerMessage::from(ServerError::InvalidMessageType))]).await;
            None
        }
    }
}

async fn session(buf_reader : &mut Take<BufReader<OwnedReadHalf>>, game_state: &Arc<Mutex<GameState>>, player_id : PlayerId, tx : &Sender<ServerMessage>, addr : std::net::SocketAddr) {
    let mut line = String::new();
    loop {
        line.clear();
        match buf_reader.read_line(&mut line).await {
            Ok(0) => {
                return;
            }
            Ok(_) => {
                match serde_json::from_str::<ClientMessage>(&line) {
                    Ok(incoming_message) => {
                        let outgoing = { let mut game_state = game_state.lock().unwrap();
                            apply(&mut game_state, player_id, incoming_message)
                        };
                        send(outgoing).await;
                    }
                    Err(_) => {
                        send(vec![(tx.clone(), ServerMessage::from(ServerError::InvalidMessageFormat))]).await;
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

fn join_game(game_state: &mut GameState, token : Option<Token>, tx : Sender<ServerMessage>) -> (Option<PlayerId>, Vec<(Sender<ServerMessage>, ServerMessage)>){
    let mut outgoing = Vec::new();
    match game_state.register(token, tx.clone()) {
        Ok((player_id, token)) => {
            outgoing.push((tx.clone(), ServerMessage::JoinGame(token)));
            for connected_player in game_state.connected_players() {
                if connected_player == player_id {
                    outgoing.push((tx.clone(), ServerMessage::from((game_state.game(), player_id))))
                } else {
                    outgoing.push((game_state.sender(connected_player).unwrap(), ServerMessage::PlayerJoined(game_state.player_info(player_id).unwrap())))
                }
            }
            (Some(player_id), outgoing)
        }
        Err(e) => {
            outgoing.push((tx, ServerMessage::Error(e)));
            (None, outgoing)
        }
    }
}

fn quit_game(game_state: &mut GameState, player_id: PlayerId) -> Vec<(Sender<ServerMessage>, ServerMessage)>{
    let mut outgoing = Vec::new();
    game_state.unregister(player_id);
    for player in game_state.connected_players() {
        outgoing.push((game_state.sender(player).unwrap(), ServerMessage::Leave(player_id)))
    }


    outgoing
}

async fn send(outgoing : Vec<(Sender<ServerMessage>, ServerMessage)>) {
    for (tx, msg) in outgoing {
        tx.send(msg).await.unwrap()
    }
}