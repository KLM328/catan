use catan::{Board, Building, Cost, GameError, GameStatus, Hand, PlayerId, ResourceError};
use catan_protocol::{GameSnapshot, PlayerInfo};

pub(crate) struct GameView {
    board: Board,
    my_id: PlayerId,
    my_hand: Hand,
    players: Vec<PlayerInfo>,
    status: GameStatus,
    turn_order: Vec<PlayerId>,
    current_turn: usize,
}

impl From<GameSnapshot> for GameView {
    fn from(value: GameSnapshot) -> Self {
        Self {
            board: value.board,
            my_id: value.player_id,
            my_hand: value.hand,
            players: value.players,
            status: value.game_status,
            turn_order: value.turn_order,
            current_turn: value.current_turn,
        }
    }
}

impl GameView {

    pub(crate) fn current_player(&self) -> PlayerId {
        self.turn_order[self.current_turn]
    }

    pub(crate) fn turn_order(&self) -> &[PlayerId] {
        &self.turn_order
    }

    pub(crate) fn get_player(&self, id: PlayerId) -> Option<&PlayerInfo> {
        self.players.iter().find(|p| p.id == id)
    }

    pub(crate) fn board(&self) -> &Board {
        &self.board
    }

    pub(crate) fn score(&self, player: &PlayerInfo) -> u8 {
        self.board.buildings().iter().flatten().filter(|&&b| b.owner() == player.id).map(|&b| b.kind().points()).sum()
    }
    
    pub(crate) fn status(&self) -> &GameStatus {
        &self.status
    }

    pub(crate) fn steal_victims(&self, player_id: PlayerId) -> Result<Vec<PlayerId>, GameError>{

        let buildings: Vec<Building> = self.board.topology().tile_vertices()
            [self.board.robber().value()]
            .iter()
            .map(|&v| self.board.buildings()[v.value()])
            .filter(|&o| o.is_some())
            .flatten()
            .collect();

        let mut victims: Vec<PlayerId> = buildings
            .iter()
            .map(|b| b.owner())
            .filter(|victims_id: &PlayerId| victims_id.value() != player_id.value())
            .filter(|&p| self.get_player(p).unwrap().hand_count() > 0)
            .collect();

        victims.sort_by_key(|p| p.value());
        victims.dedup();
        Ok(victims)
    }

    pub fn get_next_player(&self) -> PlayerId {
        self.turn_order[(self.current_turn + 1) % self.turn_order.len()]
    }

    pub fn can_pay(&self, cost : &Cost) -> Result<(), ResourceError> {
        self.my_hand.can_pay(cost)
    }
    
    pub(crate) fn my_hand(&self) -> &Hand {
        &self.my_hand
    }
    
    pub(crate) fn my_id(&self) -> PlayerId {
        self.my_id
    }
    

}