use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{watch, Notify};
use tokio::time::{sleep_until, timeout, Instant};
use catan_protocol::{ClientMessage, ServerMessage};
use crate::ConnectionState;

enum Outcome {Lost, Shutdown}
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const BACKOFF_MIN: Duration = Duration::from_millis(500);
const BACKOFF_MAX: Duration = Duration::from_secs(30);

pub(crate) async fn run<A: ToSocketAddrs + Clone>(srv_addr: A, mut to_server_rx : Receiver<ClientMessage>, to_ui_tx : Sender<ServerMessage>, ctx: egui::Context, conn_tx : watch::Sender<ConnectionState>, force_retry : Arc<Notify>) {
    let mut backoff = BACKOFF_MIN;
    let mut attempt = 0;

    loop {
        attempt += 1;
        if conn_tx.send(ConnectionState::Connecting { attempt }).is_err() {
            return;
        }
        ctx.request_repaint();


        match timeout(CONNECT_TIMEOUT, TcpStream::connect(srv_addr.clone())).await {
            Ok(Ok(socket)) => {
                backoff = BACKOFF_MIN;
                attempt = 0;
                let _ = conn_tx.send(ConnectionState::Connected);
                match session(socket, &mut to_server_rx, &to_ui_tx, &ctx).await {
                    Outcome::Shutdown => return,
                    Outcome::Lost => {},
                }
            }
            Ok(Err(e)) => eprintln!("Connexion refusée : {e}"),
            Err(_) => eprintln!("Connexion : délai dépassé"),
        }

        ctx.request_repaint();

        let deadline = Instant::now() + backoff;
        let _ = conn_tx.send(ConnectionState::Retrying { at: deadline.into_std(), attempt });
        loop{
            tokio::select! {
                _ = sleep_until(deadline) => break,
                msg = to_server_rx.recv() => {
                    match msg {
                        None => return,
                        Some(_) => continue
                    }
                }
                _ = force_retry.notified() => break
            }
        }
        backoff = (backoff * 2).min(BACKOFF_MAX);
    }
}

async fn session(socket : TcpStream, to_server_rx : &mut Receiver<ClientMessage>, to_ui_tx : &Sender<ServerMessage>, ctx : &egui::Context) -> Outcome {
    let (reader, mut writer) = socket.into_split();
    let to_ui = to_ui_tx.clone();
    let ctx = ctx.clone();

    let mut reader_task = tokio::spawn(async move {
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            match buf_reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => match serde_json::from_str::<ServerMessage>(&line) {
                    Ok(msg) => {
                        if to_ui.send(msg).await.is_err() {
                            break
                        }
                        ctx.request_repaint();
                    },
                    Err(e) => {
                        eprintln!("Désérialization : {e} - {line}")
                    }
                },
                Err(e) => {
                    eprintln!("Lecture : {e}"); break
                }
            }
        }
    });

    let outcome = loop {
        tokio::select! {
            _ = &mut reader_task => break Outcome::Lost,
            msg = to_server_rx.recv() => {
                let Some(msg) = msg else {break Outcome::Shutdown};
                let mut json = serde_json::to_string(&msg).unwrap();
                json.push('\n');
                if writer.write_all(json.as_bytes()).await.is_err() {
                    break Outcome::Lost
                }
            }
        }
    };

    reader_task.abort();
    outcome
}
