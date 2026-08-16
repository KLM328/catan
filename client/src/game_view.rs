use catan::{Board, Building, Cost, EdgeId, GameError, GameStatus, Hand, PlayerId, ResourceError, TileId, VertexId};
use catan_protocol::{GameSnapshot, PlayerInfo};

#[derive(Debug, Clone)]
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

    pub fn can_pay(&self, cost : &Cost) -> Result<(), ResourceError> {
        self.my_hand.can_pay(cost)
    }

    pub(crate) fn my_hand(&self) -> &Hand {
        &self.my_hand
    }

    pub(crate) fn my_id(&self) -> PlayerId {
        self.my_id
    }

    pub(crate) fn win(&mut self, winner: PlayerId) {
        self.status = GameStatus::End {winner}
    }

    pub(crate) fn update_player(&mut self, player: PlayerInfo) {
        if let Some(p) = self.players.iter_mut().find(|p| p.id == player.id) {
            *p = player.clone();
        }
    }

    pub(crate) fn set_status(&mut self, status : GameStatus){
        self.status = status;
    }

    pub(crate) fn set_building(&mut self, vertex : VertexId, building: Building){
        self.board.apply_settlement(vertex, building);
    }

    pub(crate) fn set_road(&mut self, edge : EdgeId, player : PlayerId){
        self.board.apply_road(edge, player);
    }

    pub(crate) fn set_robber(&mut self, tile : TileId){
        self.board.apply_robber(tile);
    }

    pub(crate) fn set_current_turn(&mut self, player : PlayerId) -> Result<(), GameError>{
        match self.turn_order.iter().position(|&p| p == player) {
            None => {Err(GameError::PlayerNotFound(player))}
            Some(index) => {self.current_turn = index; Ok(())}
        }
    }
    
    pub(crate) fn players(&self) -> &[PlayerInfo] {
        &self.players
    }


}