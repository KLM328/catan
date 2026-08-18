mod state;
mod connection;
mod dispatch;

use std::collections::HashMap;
use catan::{Game, GameId, Scenario};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::net::TcpListener;
use tokio::spawn;
use catan_protocol::GameInfo;
pub(crate) use state::GameState;
pub(crate) use connection::handle;

pub(crate) struct Registry {
    games: HashMap<GameId, Arc<Mutex<GameState>>>,
    next_id: u32,
}

impl Registry {
    fn new() -> Self {
        Self {
            games: HashMap::new(),
            next_id: 0,
        }
    }
    pub(crate) fn create(&mut self, scenario: Scenario) -> (GameId, Arc<Mutex<GameState>>)
    {
        let id = GameId::new(self.next_id);
        self.next_id += 1;
        let game = Arc::new(Mutex::new(GameState::new(Game::new(scenario))));
        self.games.insert(id, Arc::clone(&game));
        (id, game)
    }

    pub(crate) fn get(&self, id: &GameId) -> Option<Arc<Mutex<GameState>>> {
        self.games.get(id).map(Arc::clone)
    }

    pub(crate) fn games_info(&self) -> Vec<GameInfo> {
        let mut games_info = Vec::new();
        for (&game_id, game_state) in self.games.iter() {
            let game_state = game_state.lock().unwrap();
            games_info.push(GameInfo::new(game_id, game_state.game().scenario().clone(), game_state.connected_players().len()))
        }
        games_info.sort_by_key(|game_info| game_info.id);
        games_info
    }
}



type Games = Arc<Mutex<Registry>>;
#[tokio::main]
async fn main() {
    let games : Games = Arc::new(Mutex::new(Registry::new()));

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

