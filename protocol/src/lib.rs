use catan::{Board, EdgeId, Game, GameStatus, Hand, Player, PlayerColor, PlayerId, ResourceCounts, Roll, Scenario, TileId, VertexId};
use serde::{Deserialize, Serialize};

mod server_error;
mod token;

pub use server_error::ServerError;
pub use token::Token;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ClientMessage {
    BuildRoad(EdgeId),
    BuildSettlement(VertexId),
    UpgradeCity(VertexId),
    Discard(ResourceCounts),
    Steal(Option<PlayerId>),
    RobberLocation(TileId),
    Roll,
    EndTurn,
    Join {token : Option<Token>},
    StartGame,
    Sync
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PlayerInfo {
    pub id: PlayerId,
    pub color: PlayerColor,
    pub hand_count: u8,
}
impl From<(&Player, PlayerId)> for PlayerInfo {
    fn from((player, id): (&Player, PlayerId)) -> Self {
        Self {
            color : player.color(),
            hand_count : player.hand().count(),
            id
        }
    }
}

impl PlayerInfo {
    pub fn color(&self) -> PlayerColor {
        self.color
    }

    pub fn id(&self) -> PlayerId {
        self.id
    }

    pub fn hand_count(&self) -> u8 {
        self.hand_count
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GameSnapshot {
    pub scenario: Scenario,
    pub board : Board,
    pub player_id: PlayerId,
    pub players: Vec<PlayerInfo>,
    pub game_status: GameStatus,
    pub turn_order: Vec<PlayerId>,
    pub current_turn: usize,
    pub hand: Hand,
}

impl GameSnapshot {
    fn new(game: &Game, player_id: PlayerId) -> Self {
        Self {
            scenario: game.scenario().clone(),
            board : game.board().unwrap().clone(),
            players: game
                .players()
                .iter()
                .map(|(&id, p)| PlayerInfo {
                    color: p.color(),
                    id,
                    hand_count: p.hand().count(),
                })
                .collect(),
            game_status: game.status(),
            turn_order: game.turn_order().to_vec(),
            current_turn: game.current_player_index(),
            hand: game.get_player(player_id).unwrap().hand().clone(),
            player_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct StateUpdate {
    pub status: GameStatus,
    pub current_turn: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ServerMessage {
    HandUpdate(Hand),
    BuildRoad(PlayerInfo, EdgeId, StateUpdate),
    BuildSettlement(PlayerInfo, VertexId, StateUpdate),
    UpgradeCity(PlayerInfo, VertexId),
    Steal {
        robber: PlayerInfo,
        victim: Option<PlayerInfo>,
        state : StateUpdate,
    },
    Discard(PlayerInfo, StateUpdate),
    NewRobberLocation(TileId, StateUpdate),
    Roll(Roll, StateUpdate),
    NextPlayer(StateUpdate),
    GameEnd(StateUpdate),
    PlayerJoined(PlayerInfo),
    Leave(PlayerId),
    Sync(GameSnapshot),
    LobbyView {
        player_id: PlayerId,
        players: Vec<PlayerInfo>,
        scenario: Scenario,
    },
    StartGame(Vec<(PlayerId, Roll)>),
    Error(ServerError),
    JoinGame(Token),
    PauseGame,
    ResumeGame,
}

impl From<(&Game, PlayerId)> for ServerMessage {
    fn from((game, viewer): (&Game, PlayerId)) -> Self {
        match game.status() {
            GameStatus::Starting => ServerMessage::LobbyView {
                players: game
                    .players()
                    .iter()
                    .map(|(&id, p)| PlayerInfo {
                        color: p.color(),
                        id,
                        hand_count: 0,
                    })
                    .collect(),
                scenario: game.scenario().clone(),
                player_id: viewer,
            },
            _ => {
                ServerMessage::Sync(GameSnapshot::new(game, viewer))
            }
        }
    }
}

impl From<ServerError> for ServerMessage {
    fn from(error: ServerError) -> Self {
        ServerMessage::Error(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use catan::{GameError, Terrain};

    fn assert_serde_roundtrip<T>(message: T)
    where
        T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + PartialEq,
    {
        let json = serde_json::to_string(&message).expect("Échec de la sérialisation");
        let back: T = serde_json::from_str(&json).expect("Échec de la désérialisation");
        assert_eq!(message, back);
    }

    // Helper pour générer un PlayerInfo factice
    fn dummy_player_info(id: usize) -> PlayerInfo {
        PlayerInfo {
            id: PlayerId::new(id),
            color: PlayerColor::Red, // Assurez-vous d'adapter selon la variante de PlayerColor disponible dans `catan`
            hand_count: 5,
        }
    }

    #[test]
    fn test_client_messages_roundtrip() {
        let messages = vec![
            ClientMessage::BuildRoad(EdgeId::new(1)),
            ClientMessage::BuildSettlement(VertexId::new(2)),
            ClientMessage::UpgradeCity(VertexId::new(3)),
            ClientMessage::Discard(ResourceCounts::default()),
            ClientMessage::Steal(Some(PlayerId::new(1))),
            ClientMessage::Steal(None),
            ClientMessage::RobberLocation(TileId::new(4)),
            ClientMessage::Roll,
            ClientMessage::EndTurn,
            ClientMessage::Join{token : None},
        ];

        for msg in messages {
            assert_serde_roundtrip(msg);
        }
    }

    #[test]
    fn test_server_messages_roundtrip() {
        let p1 = dummy_player_info(0);
        let p2 = dummy_player_info(1);
        
        let state = StateUpdate {
            status : GameStatus::PlayingActions,
            current_turn : 1,
        };

        let messages = vec![
            ServerMessage::BuildRoad(dummy_player_info(0), EdgeId::new(1), state),
            ServerMessage::BuildSettlement(dummy_player_info(0), VertexId::new(2), state),
            ServerMessage::UpgradeCity(dummy_player_info(0), VertexId::new(3)),
            ServerMessage::HandUpdate(Hand::default()),
            ServerMessage::Steal {
                robber: dummy_player_info(0),
                victim: Some(dummy_player_info(1)),
                state
            },
            ServerMessage::Discard(dummy_player_info(0), state),
            ServerMessage::Roll(
                Roll::new(4, 6).unwrap(),
                state
            ),
            ServerMessage::NextPlayer(state),
            ServerMessage::GameEnd(StateUpdate {
                current_turn : 0,
                status : GameStatus::End {winner : PlayerId::new(0)},
            }),
            ServerMessage::PlayerJoined(dummy_player_info(0)),
            ServerMessage::Leave(PlayerId::new(0)),
            ServerMessage::LobbyView {
                players: vec![p1, p2],
                scenario: Scenario::standard(),
                player_id: PlayerId::new(0),
            },
            ServerMessage::StartGame(vec![
                (PlayerId::new(0), Roll::new(4, 6).unwrap()),
                (PlayerId::new(1), Roll::new(4, 5).unwrap()),
            ]),
            ServerMessage::from(ServerError::from(GameError::InvalidGameStatus))
        ];

        for msg in messages {
            assert_serde_roundtrip(msg);
        }
    }

    #[test]
    fn test_server_message_game_view_roundtrip() {
        let mut game = Game::new(Scenario::standard());
        game.add_player(Player::new(PlayerColor::Red));
        game.add_player(Player::new(PlayerColor::White));

        let terrains : Vec<Terrain> = game.scenario().terrains().iter().copied().collect();

        game.start(&terrains).unwrap();


        let game_view = ServerMessage::Sync(


            GameSnapshot{
                scenario: Scenario::standard(),
                board : game.board().unwrap().clone(),
                players: vec![dummy_player_info(0)],
                game_status: GameStatus::Starting,
                turn_order: vec![PlayerId::new(0)],
                current_turn: 0,
                hand: Hand::default(),
                player_id: PlayerId::new(0),
            }
        );

        assert_serde_roundtrip(game_view);
    }
}
