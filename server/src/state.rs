use std::collections::HashMap;
use tokio::sync::mpsc::Sender;
use catan::{Game, PlayerId};
use catan_protocol::ServerMessage;

pub(crate) struct GameState {
    game: Game,
    senders : HashMap<PlayerId, Sender<ServerMessage>>,
}

impl GameState {
    pub(crate) fn new(game: Game) -> Self {
        Self {
            game,
            senders : HashMap::new(),
        }
    }
    
    pub(crate) fn game(&self) -> &Game {
        &self.game
    }
    
    pub(crate) fn game_mut(&mut self) -> &mut Game {
        &mut self.game
    }
    
    pub(crate) fn senders(&self) -> &HashMap<PlayerId, Sender<ServerMessage>> {
        &self.senders
    }
    
    pub(crate) fn senders_mut(&mut self) -> &mut HashMap<PlayerId, Sender<ServerMessage>> {
        &mut self.senders
    }
}