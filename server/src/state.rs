use catan::{Game, GameError, PlayerId};
use catan_protocol::{PlayerInfo, ServerMessage};
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;

pub(crate) struct GameState {
    game: Game,
    random_board : bool,
    senders: HashMap<PlayerId, Sender<ServerMessage>>,
}

impl GameState {
    pub(crate) fn new(game: Game, random_board : bool) -> Self {
        Self {
            game,
            random_board,
            senders: HashMap::new(),
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

    pub(crate) fn random_board(&self) -> bool {
        self.random_board
    }
    
    pub(crate) fn player_info(&self, player_id: PlayerId) -> Result<PlayerInfo, GameError> {
        Ok(PlayerInfo::from((self.game.get_player(player_id)? , player_id)))
    }
}
