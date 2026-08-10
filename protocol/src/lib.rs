use catan::{
    Building, EdgeId, Game, GameStatus, Hand, PlayerColor, PlayerId, Resource,
    ResourceCounts, Roll, RollOutcome, Scenario, Tile, TileId, VertexId,
};

pub enum ClientMessage {
    BuildRoad(EdgeId),
    BuildSettlement(VertexId),
    UpgradeCity(VertexId),
    Discard(ResourceCounts),
    Steal(Option<PlayerId>),
    RobberLocation(TileId),
    Roll,
    EndTurn,
    Join, //à réfléchir plus en détails plus tard
}

pub struct PlayerInfo {
    pub id: PlayerId,
    pub color: PlayerColor,
    pub hand_count: u8,
}

pub enum ServerMessage {
    BuildRoad(PlayerInfo, EdgeId),
    BuildSettlement(PlayerInfo, VertexId),
    UpgradeCity(PlayerInfo, VertexId),
    StealNotification {
        robber: PlayerInfo,
        victim: PlayerInfo,
    },
    StealConfirmation {
        robber: PlayerInfo,
        victim: PlayerInfo,
        resource: Resource,
    },
    Discard(PlayerInfo),
    Roll(Roll, RollOutcome),
    NextPlayer(PlayerInfo),
    GameEnd {
        winner: PlayerId,
    },
    PlayerJoined(PlayerInfo),
    Leave(PlayerId),
    GameView {
        scenario: Scenario,
        tiles: Vec<Tile>,
        buildings: Vec<Option<Building>>,
        roads: Vec<Option<PlayerId>>,
        players: Vec<PlayerInfo>,
        game_status: GameStatus,
        turn_order: Vec<PlayerId>,
        current_turn: usize,
        hand: Hand,
    },
    LobbyView {
        players: Vec<PlayerInfo>,
        scenario: Scenario,
    },
    StartGame,
}

impl From<(&Game, PlayerId)> for ServerMessage {
    fn from((game, viewer) : (&Game, PlayerId)) -> Self {
        match game.status() {
            GameStatus::Starting => ServerMessage::LobbyView{ players : game.players().iter().enumerate().map(|(index, p)| PlayerInfo{color : p.color(), id : PlayerId::new(index), hand_count : 0}).collect(),
                scenario : game.scenario().clone()
            },
            _ => {
                let board = game.board().unwrap();
                ServerMessage::GameView {
                    scenario: game.scenario().clone(),
                    tiles: board.tiles().to_vec(),
                    buildings: board.buildings().to_vec(),
                    roads: board.roads().to_vec(),
                    players : game.players().iter().enumerate().map(|(index, p)| PlayerInfo{color : p.color(), id : PlayerId::new(index), hand_count : p.hand().count()}).collect(),
                    game_status : game.status(),
                    turn_order : game.turn_order().to_vec(),
                    current_turn : game.current_player_index(),
                    hand : game.get_player(viewer).unwrap().hand().clone(),

                }
            }
        }
    }
}
