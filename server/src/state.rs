use catan::{Game, GameError, PlayerId};
use catan_protocol::{PlayerInfo, ServerError, ServerMessage, Token};
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;
use tokio::time::Instant;

pub(crate) struct GameState {
    game: Game,
    random_board : bool,
    senders: HashMap<PlayerId, Sender<ServerMessage>>,
    paused_since: Option<Instant>,
    tokens: HashMap<Token, PlayerId>
}

impl GameState {
    pub(crate) fn new(game: Game, random_board : bool) -> Self {
        Self {
            game,
            random_board,
            senders: HashMap::new(),
            paused_since: None,
            tokens: HashMap::new()
        }
    }

    pub(crate) fn game(&self) -> &Game {
        &self.game
    }

    pub(crate) fn game_mut(&mut self) -> &mut Game {
        &mut self.game
    }

    pub(crate) fn tokens(&self) -> &HashMap<Token, PlayerId> {
        &self.tokens
    }

    pub(crate) fn tokens_mut(&mut self) -> &mut HashMap<Token, PlayerId> {
        &mut self.tokens
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

    pub(crate) fn paused_since(&self) -> Option<Instant> {
        self.paused_since
    }
    pub(crate) fn set_paused_since(&mut self, paused_since: Option<Instant>) {
        self.paused_since = paused_since;
    }

    pub(crate) fn player_info(&self, player_id: PlayerId) -> Result<PlayerInfo, GameError> {
        Ok(PlayerInfo::from((self.game.get_player(player_id)? , player_id)))
    }

    pub(crate) fn register(&mut self, player: PlayerId, tx: Sender<ServerMessage>) -> Result<(), ServerError> {
        if self.senders.contains_key(&player) {
            Err(ServerError::PlayerIsAlreadyConnected)
        } else {
            self.senders.insert(player, tx);
            if self.is_paused() && self.senders.len() == self.game.players().len() {
                self.resume();
            }
            Ok(())
        }
    }
    
    fn pause(&mut self){
        self.paused_since = Some(Instant::now());
    }
    
    fn resume(&mut self){
        self.paused_since = None;
    }
    pub(crate) fn unregister(&mut self, player: PlayerId) {
        self.senders.remove(&player);
        if self.is_paused() {
            self.pause()
        }
    }

    pub(crate) fn is_paused(&self) -> bool {
        self.paused_since.is_some()
    }
}
