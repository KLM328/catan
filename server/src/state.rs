use catan::{Game, GameError, GameStatus, Player, PlayerId};
use catan_protocol::{PlayerInfo, ServerError, ServerMessage, StateUpdate, Token};
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
    pub(crate) fn new(game: Game) -> Self {
        Self {
            game,
            random_board : true,
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
    


    pub(crate) fn random_board(&self) -> bool {
        self.random_board
    }

    pub(crate) fn player_info(&self, player_id: PlayerId) -> Result<PlayerInfo, GameError> {
        Ok(PlayerInfo::from((self.game.get_player(player_id)? , player_id)))
    }

    pub(crate) fn register(&mut self, token: Option<Token>, tx: Sender<ServerMessage>) -> Result<(PlayerId, Token), ServerError> {
        let (player, token) = match token {
            None => self.new_player()?,
            Some(token) => (self.player_by_token(token)?, token)
        };
        if let std::collections::hash_map::Entry::Vacant(e) = self.senders.entry(player) {
            e.insert(tx);
            if self.is_paused() && self.senders.len() == self.game.players().len() {
                self.resume();
            }
            Ok((player, token))
        } else {
            Err(ServerError::PlayerIsAlreadyConnected)
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
        if matches!(self.game.status(), GameStatus::Starting) {
            if let Some((&token, _)) = self.tokens.iter().find(|&(_, &p)| p == player) {
                self.tokens.remove(&token);
            }
            self.game.remove_player(player).unwrap();
        }
        if !self.is_paused() && self.connected_players().len() < self.game.players().len() {
            self.pause()
        }
    }

    pub(crate) fn is_paused(&self) -> bool {
        self.paused_since.is_some()
    }

    pub(crate) fn new_player(&mut self) -> Result<(PlayerId, Token), ServerError> {
        let color = self.game.next_player_color()?;
        let player_id = self.game.add_player(Player::new(color))?;
        let token = Token::new();
        self.tokens.insert(token, player_id);
        Ok((player_id, token))
    }

    pub(crate) fn player_by_token(&self, token: Token) -> Result<PlayerId, ServerError> {
        self.tokens.get(&token).copied().ok_or(ServerError::InvalidToken)
    }

    pub(crate) fn connected_players(&self) -> Vec<PlayerId> {
        let mut players : Vec<PlayerId> = self.senders.keys().copied().collect();
        players.sort_by_key(|&p| p.value());
        players
    }

    pub(crate) fn sender(&self, player: PlayerId) -> Option<Sender<ServerMessage>>{
        self.senders.iter().find(|&(&p, _)| p == player).map(|(_, s)| s.clone())
    }
    
    pub(crate) fn broadcast(&self, message: ServerMessage) -> Vec<(Sender<ServerMessage>, ServerMessage)>{
        self.connected_players().iter().map(|&p| (self.sender(p).unwrap(), message.clone())).collect()
    }
    
    pub(crate) fn state(&self) -> StateUpdate {
        StateUpdate {
            status: self.game.status(),
            current_turn: self.game.current_player_index(),
        }
    }
}
