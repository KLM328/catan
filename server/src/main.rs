mod state;
mod connection;
mod dispatch;

use catan::{Game, Scenario};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::net::TcpListener;
use tokio::spawn;


pub(crate) use state::GameState;
pub(crate) use connection::handle;
#[tokio::main]
async fn main() {
    let game_state : Arc<Mutex<GameState>> = Arc::new(Mutex::new(GameState::new(Game::new(Scenario::standard()))));


    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        println!("connexion de {addr}");
        let game_state = game_state.clone();
        spawn(async move {
            handle(socket, addr, game_state).await;
        });
    }
}

