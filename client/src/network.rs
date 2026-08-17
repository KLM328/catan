use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::spawn;
use tokio::sync::mpsc::{Receiver, Sender};
use catan_protocol::{ClientMessage, ServerError, ServerMessage};

pub(crate) async fn run<A: ToSocketAddrs>(srv_addr: A, mut to_server_rx : Receiver<ClientMessage>, to_ui_tx : Sender<ServerMessage>, ctx: egui::Context) {

    if let Ok(socket) = TcpStream::connect(srv_addr).await {
        let (reader, mut writer) = socket.into_split();

        let mut buf_reader = tokio::io::BufReader::new(reader);
        let mut line = String::new();

        spawn(async move {
            loop {
                line.clear();
                match buf_reader.read_line(&mut line).await {
                    Ok(0) => {
                        println!("server offline");
                        break
                    }
                    Ok(_) => {
                        let message = serde_json::from_str::<ServerMessage>(&line);
                        match message {
                            Ok(message) => {
                                to_ui_tx.send(message).await.unwrap();
                                ctx.request_repaint();
                            }
                            Err(_) => {
                                eprintln!("Erreur de deserialization d'un message provenant du serveur : {line}");
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Erreur de lecture : {e}");
                        break
                    }
                }
            }
        });

        loop {
            while let Some(msg) = to_server_rx.recv().await {
                let mut json = serde_json::to_string(&msg).unwrap();
                json.push('\n');
                writer.write_all(json.as_bytes()).await.unwrap();
            }
        }
    } else {
        to_ui_tx.send(ServerMessage::Error(ServerError::ServerOffline)).await.unwrap();
    }
}
