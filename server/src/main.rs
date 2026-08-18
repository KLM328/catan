mod state;
mod connection;
mod dispatch;

use std::collections::HashMap;
use catan::{Game, Scenario, GameId};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::net::TcpListener;
use tokio::spawn;


pub(crate) use state::GameState;
pub(crate) use connection::handle;

type Games = Arc<Mutex<HashMap<GameId, Arc<Mutex<GameState>>>>>;
#[tokio::main]
async fn main() {
    let games : Games = Arc::new(Mutex::new(HashMap::new()));
    
    let listener = TcpListener::bind("127.0.0.1:8888").await.unwrap();
    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        println!("connexion de {addr}");
        let games = games.clone();
        spawn(async move {
            handle(socket, addr, games).await;
        });
    }
}

