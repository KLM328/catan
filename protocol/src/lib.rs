use catan::{Building, EdgeId, Game, GameStatus, Hand, Player, PlayerColor, PlayerId, ResourceCounts, Roll, RollOutcome, Scenario, Tile, TileId, VertexId};
use serde::{Deserialize, Serialize};
use crate::server_error::ServerError;

pub mod server_error;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
    StartGame,
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ServerMessage {
    BuildRoad(PlayerInfo, EdgeId),
    BuildSettlement(PlayerInfo, VertexId),
    UpgradeCity(PlayerInfo, VertexId),
    StealNotification {
        robber: PlayerInfo,
        victim: Option<PlayerInfo>,
    },
    StealConfirmation {
        robber: PlayerInfo,
        victim: PlayerInfo,
        resource: ResourceCounts,
    },
    Discard(PlayerInfo),
    NewRobberLocation(TileId),
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
        player_id: PlayerId,
        players: Vec<PlayerInfo>,
        game_status: GameStatus,
        turn_order: Vec<PlayerId>,
        current_turn: usize,
        hand: Hand,
    },
    LobbyView {
        player_id: PlayerId,
        players: Vec<PlayerInfo>,
        scenario: Scenario,
    },
    StartGame(Vec<Roll>),
    Error(ServerError),
}

impl From<(&Game, PlayerId)> for ServerMessage {
    fn from((game, viewer): (&Game, PlayerId)) -> Self {
        match game.status() {
            GameStatus::Starting => ServerMessage::LobbyView {
                players: game
                    .players()
                    .iter()
                    .enumerate()
                    .map(|(index, p)| PlayerInfo {
                        color: p.color(),
                        id: PlayerId::new(index),
                        hand_count: 0,
                    })
                    .collect(),
                scenario: game.scenario().clone(),
                player_id: viewer,
            },
            _ => {
                let board = game.board().unwrap();
                ServerMessage::GameView {
                    scenario: game.scenario().clone(),
                    tiles: board.tiles().to_vec(),
                    buildings: board.buildings().to_vec(),
                    roads: board.roads().to_vec(),
                    players: game
                        .players()
                        .iter()
                        .enumerate()
                        .map(|(index, p)| PlayerInfo {
                            color: p.color(),
                            id: PlayerId::new(index),
                            hand_count: p.hand().count(),
                        })
                        .collect(),
                    game_status: game.status(),
                    turn_order: game.turn_order().to_vec(),
                    current_turn: game.current_player_index(),
                    hand: game.get_player(viewer).unwrap().hand().clone(),
                    player_id: viewer,
                }
            }
        }
    }
}

impl From<ServerError> for ServerMessage {
    fn from(error: ServerError) -> Self {
        ServerMessage::Error(ServerError::from(error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use catan::{BuildingKind, GameError, Production};

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
            ClientMessage::Join,
        ];

        for msg in messages {
            assert_serde_roundtrip(msg);
        }
    }

    #[test]
    fn test_server_messages_roundtrip() {
        let p1 = dummy_player_info(0);
        let p2 = dummy_player_info(1);

        let messages = vec![
            ServerMessage::BuildRoad(dummy_player_info(0), EdgeId::new(1)),
            ServerMessage::BuildSettlement(dummy_player_info(0), VertexId::new(2)),
            ServerMessage::UpgradeCity(dummy_player_info(0), VertexId::new(3)),
            ServerMessage::StealNotification {
                robber: dummy_player_info(0),
                victim: Some(dummy_player_info(1)),
            },
            ServerMessage::StealConfirmation {
                robber: dummy_player_info(0),
                victim: dummy_player_info(1),
                resource: ResourceCounts::new([1,0,0,0,0]), // Adaptez avec une ressource existante dans `catan::Resource`
            },
            ServerMessage::Discard(dummy_player_info(0)),
            ServerMessage::Roll(
                Roll::new(4, 6).unwrap(),
                RollOutcome::Production(Production::new(&[(
                    dummy_player_info(0).id,
                    [0, 2, 0, 0, 1],
                )])),
            ),
            ServerMessage::NextPlayer(dummy_player_info(1)),
            ServerMessage::GameEnd {
                winner: PlayerId::new(0),
            },
            ServerMessage::PlayerJoined(dummy_player_info(0)),
            ServerMessage::Leave(PlayerId::new(0)),
            ServerMessage::LobbyView {
                players: vec![p1, p2],
                scenario: Scenario::standard(),
                player_id: PlayerId::new(0),
            },
            ServerMessage::StartGame(vec![
                Roll::new(4, 6).unwrap(),
                Roll::new(4, 5).unwrap(),
            ]),
            ServerMessage::from(ServerError::from(GameError::InvalidGameStatus))
        ];

        for msg in messages {
            assert_serde_roundtrip(msg);
        }
    }

    #[test]
    fn test_server_message_game_view_roundtrip() {
        // Test dédié à GameView car sa structure est plus complexe
        let game_view = ServerMessage::GameView {
            scenario: Scenario::standard(),
            tiles: vec![],
            buildings: vec![
                None,
                Some(Building::new(BuildingKind::Settlement, PlayerId::new(0))),
            ],
            roads: vec![None, Some(PlayerId::new(0))],
            players: vec![dummy_player_info(0)],
            game_status: GameStatus::Starting, // Adaptez selon les variantes de GameStatus
            turn_order: vec![PlayerId::new(0)],
            current_turn: 0,
            hand: Hand::default(),
            player_id: PlayerId::new(0),
        };

        assert_serde_roundtrip(game_view);
    }
}
