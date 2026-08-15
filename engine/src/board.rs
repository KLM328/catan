use crate::{ConnectedEdges, EdgeId, PlayerId, Resource, ResourceCounts};
use crate::{Roll, TileId, Topology, VertexId};
use serde::{Deserialize, Serialize};
use std::fmt;

mod building;
mod production;
mod tile;

pub use crate::board::building::{Building, BuildingKind};

pub use crate::board::production::{Gain, Production};
pub use crate::board::tile::{NumberToken, Terrain, TerrainTokenMismatch, Tile};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvalidBoard {
    WrongTileCount { expected: usize, got: usize },
    WrongDistribution,
    NoDesert,
    TerrainToken(TerrainTokenMismatch),
    InvalidRobber,
}

impl fmt::Display for InvalidBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::WrongTileCount { expected, got } => write!(
                f,
                "nombre de tuiles invalide : attendu {expected}, obtenu {got}"
            ),
            Self::WrongDistribution => write!(f, "la répartition ne correspond pas au scénario"),
            Self::NoDesert => write!(f, "aucun désert dans la répartition"),
            Self::TerrainToken(e) => write!(f, "{e}"),
            Self::InvalidRobber => write!(f, "Position du voleur invalide"),
        }
    }
}

impl From<TerrainTokenMismatch> for InvalidBoard {
    fn from(e: TerrainTokenMismatch) -> Self {
        Self::TerrainToken(e)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvalidAction {
    // structurel
    UnexistingTile(TileId),
    UnexistingVertex(VertexId),
    UnexistingEdge(EdgeId),
    UnexistingBuilding(VertexId),
    RobberNotOnDesert,
    IsNotSettlement(VertexId),

    // règles de pose
    VertexOccupied(VertexId),
    TooCloseToBuilding(VertexId),
    EdgeOccupied(EdgeId),
    NotConnected,
    RoadMustStartFromNewSettlement,
    NotYourSettlement(VertexId),
    RobberMustMove,
}

impl fmt::Display for InvalidAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidAction::RobberNotOnDesert => {
                write!(f, "Le voleur ne peut pas être déplacé sur un désert")
            }
            InvalidAction::UnexistingTile(tile_id) => {
                write!(f, "La tuile {} n'existe pas", tile_id.value())
            }
            InvalidAction::UnexistingVertex(vertex_id) => {
                write!(f, "Le vertex {} n'existe pas", vertex_id.value())
            }
            InvalidAction::IsNotSettlement(vertex_id) => {
                write!(
                    f,
                    "ce n'est pas une colonie sur l'emplacement {}",
                    vertex_id.value()
                )
            }
            InvalidAction::UnexistingBuilding(vertex_id) => {
                write!(
                    f,
                    "Aucune construction sur l'emplacement {}",
                    vertex_id.value()
                )
            }
            InvalidAction::UnexistingEdge(edge_id) => {
                write!(f, "L'arête {} n'existe pas", edge_id.value())
            }
            InvalidAction::VertexOccupied(verted_id) => {
                write!(f, "Le noeud {} est occupé", verted_id.value())
            }
            InvalidAction::TooCloseToBuilding(vertex_id) => {
                write!(
                    f,
                    "Le noeud est trop pproche du noeud {} où il y a déjà une construction",
                    vertex_id.value()
                )
            }
            InvalidAction::EdgeOccupied(edge_id) => {
                write!(f, "l'arête {} est déjà occupée", edge_id.value())
            }
            InvalidAction::NotConnected => {
                write!(f, "pas connecté aux autres constructions du joueur")
            }
            InvalidAction::RoadMustStartFromNewSettlement => {
                write!(f, "la route doit commencer par une nouvelle colonie")
            }
            InvalidAction::NotYourSettlement(vertex_id) => {
                write!(
                    f,
                    "ce n'est pas votre construction sur le noeud {}",
                    vertex_id.value()
                )
            }
            InvalidAction::RobberMustMove => {
                write!(f, "Le voleur doit être déplacé")
            }
        }
    }
}
impl std::error::Error for InvalidAction {}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Board {
    topology: Topology,
    tiles: Vec<Tile>,
    buildings: Vec<Option<Building>>,
    roads: Vec<Option<PlayerId>>,
    robber: TileId,
}
impl Board {

    pub(crate) fn new(
        topology: Topology,
        tiles: Vec<Tile>,
        robber: TileId,
    ) -> Result<Board, InvalidBoard> {
        if tiles.len() != topology.hexes().len() {
            Err(InvalidBoard::WrongTileCount {
                expected: topology.hexes().len(),
                got: tiles.len(),
            })
        } else if robber.value() >= tiles.len() || !matches!(tiles[robber.value()], Tile::Desert) {
            Err(InvalidBoard::InvalidRobber)
        } else {
            Ok(Board {
                tiles,
                buildings: vec![None; topology.vertex_count()],
                roads: vec![None; topology.edge_count()],
                robber,
                topology,
            })
        }
    }

    pub fn robber(&self) -> TileId {
        self.robber
    }

    pub fn tiles(&self) -> &[Tile] {
        &self.tiles
    }

    pub fn topology(&self) -> &Topology {
        &self.topology
    }

    pub fn buildings(&self) -> &[Option<Building>] {
        &self.buildings
    }

    pub fn roads(&self) -> &[Option<PlayerId>] {
        &self.roads
    }

    pub(crate) fn production(&self, roll: Roll) -> Production {
        let mut production = Production::default();

        for (index, tile) in self.tiles.iter().enumerate() {
            if index == self.robber.value() {
                continue;
            }
            if tile.number().map(|n| n.value()) != Some(roll.value()) {
                continue;
            }
            let Some(resource) = tile.resource() else {
                continue;
            };

            for vertex in &self.topology.tile_vertices()[index] {
                let mut resources = [0;5];

                if let Some(building) = self.buildings[vertex.value()] {
                    resources[resource.index()] += building.kind().amount();
                        production.add_gain(Gain {
                        player: building.owner(),
                        resources : ResourceCounts::new(resources)
                    });
                }
            }
        }
        production.sort();
        production
    }

    pub(crate) fn can_place_settlement_during_playing(
        &self,
        vertex: VertexId,
        player: PlayerId,
    ) -> Result<(), InvalidAction> {
        if vertex.value() >= self.topology.vertex_count() {
            Err(InvalidAction::UnexistingVertex(vertex))
        } else if self.buildings[vertex.value()].is_some() {
            Err(InvalidAction::VertexOccupied(vertex))
        } else {
            let mut vertices = self.topology.vertex_neighbors(vertex);
            let connected_edges = vertex.connected_edges(self.topology());
            vertices.push(vertex);
            if !vertices.iter().all(|v| self.buildings[v.value()].is_none()) {
                Err(InvalidAction::TooCloseToBuilding(
                    *vertices
                        .iter()
                        .find(|&&v| self.buildings[v.value()].is_some())
                        .unwrap(),
                ))
            } else if connected_edges
                .iter()
                .any(|&e_id| self.roads[e_id.value()] == Some(player))
            {
                Ok(())
            } else {
                Err(InvalidAction::NotConnected)
            }
        }
    }

    pub(crate) fn can_place_settlement_during_placement(
        &self,
        vertex: VertexId,
        _: PlayerId,
    ) -> Result<(), InvalidAction> {
        if vertex.value() >= self.topology.vertex_count() {
            Err(InvalidAction::UnexistingVertex(vertex))
        } else if self.buildings[vertex.value()].is_some() {
            Err(InvalidAction::VertexOccupied(vertex))
        } else {
            let mut vertices = self.topology.vertex_neighbors(vertex);
            vertices.push(vertex);
            if !vertices.iter().all(|v| self.buildings[v.value()].is_none()) {
                Err(InvalidAction::TooCloseToBuilding(
                    *vertices
                        .iter()
                        .find(|&&v| self.buildings[v.value()].is_some())
                        .unwrap(),
                ))
            } else {
                Ok(())
            }
        }
    }

    pub(crate) fn can_place_road_during_placement(
        &self,
        edge: EdgeId,
        player: PlayerId,
    ) -> Result<(), InvalidAction> {
        if let Some(target_edge) = self.roads.get(edge.value()) {
            if target_edge.is_some() {
                Err(InvalidAction::EdgeOccupied(edge))
            } else {
                let endpoints = self.topology.edges_endpoints()[edge.value()];

                let player_buildings = endpoints
                    .iter()
                    .filter(|&&v| {
                        if let Some(building) = self.buildings[v.value()] {
                            building.owner() == player
                        } else {
                            false
                        }
                    })
                    .copied()
                    .collect::<Vec<VertexId>>();

                if player_buildings.is_empty() {
                    Err(InvalidAction::RoadMustStartFromNewSettlement)
                } else if player_buildings.iter().any(|&v| {
                    v.connected_edges(self.topology())
                        .iter()
                        .all(|&e| self.roads[e.value()] != Some(player))
                }) {
                    Ok(())
                } else {
                    Err(InvalidAction::RoadMustStartFromNewSettlement)
                }
            }
        } else {
            Err(InvalidAction::UnexistingEdge(edge))
        }
    }

    pub(crate) fn can_place_road_during_playing(
        &self,
        edge: EdgeId,
        player: PlayerId,
    ) -> Result<(), InvalidAction> {
        if let Some(target_edge) = self.roads.get(edge.value()) {
            if target_edge.is_some() {
                Err(InvalidAction::EdgeOccupied(edge))
            } else {
                self.is_connected_to_road(player, edge)?;
                Ok(())
            }
        } else {
            Err(InvalidAction::UnexistingEdge(edge))
        }
    }

    fn is_connected_to_road(&self, player_id: PlayerId, object : impl ConnectedEdges) -> Result<(), InvalidAction> {

        if object.connected_edges(self.topology())
            .iter()
            .any(|&e| self.roads[e.value()] == Some(player_id))
        {
            Ok(())
        } else {
            Err(InvalidAction::NotConnected)
        }

    }

    pub(crate) fn place_road(
        &mut self,
        check_function: impl Fn(&Self, EdgeId, PlayerId) -> Result<(), InvalidAction>,
        edge: EdgeId,
        player: PlayerId,
    ) -> Result<(), InvalidAction> {
        if self.roads.get(edge.value()).is_none() {
            Err(InvalidAction::UnexistingEdge(edge))
        } else {
            check_function(self, edge, player)?;
            self.roads[edge.value()] = Some(player);
            Ok(())
        }
    }

    pub(crate) fn place_settlement(
        &mut self,
        check_function: impl Fn(&Self, VertexId, PlayerId) -> Result<(), InvalidAction>,
        vertex: VertexId,
        player: PlayerId,
    ) -> Result<(), InvalidAction> {
        check_function(self, vertex, player)?;
        self.buildings[vertex.value()] = Some(Building::new(BuildingKind::Settlement, player));
        Ok(())
    }

    pub(crate) fn move_robber(&mut self, tile_id: TileId) -> Result<(), InvalidAction> {
        let option_tile = self.tiles.get(tile_id.value());
        match option_tile {
            Some(_) => {
                if self.robber == tile_id {
                    Err(InvalidAction::RobberMustMove)
                } else {
                    self.robber = tile_id;
                    Ok(())
                }
            }
            None => Err(InvalidAction::UnexistingTile(tile_id)),
        }
    }

    pub(crate) fn can_upgrade_settlement_to_city(
        &self,
        vertex: VertexId,
        player: PlayerId,
    ) -> Result<(), InvalidAction> {
        if vertex.value() >= self.topology.vertex_count() {
            Err(InvalidAction::UnexistingVertex(vertex))
        } else {
            if let Some(building) = self.buildings[vertex.value()] {
                if matches!(building.kind(), BuildingKind::Settlement) {
                    if building.owner() == player {
                        Ok(())
                    } else {
                        Err(InvalidAction::NotYourSettlement(vertex))
                    }
                } else {
                    Err(InvalidAction::IsNotSettlement(vertex))
                }
            } else {
                Err(InvalidAction::UnexistingBuilding(vertex))
            }
        }
    }

    pub(crate) fn upgrade_settlement_to_city(
        &mut self,
        vertex: VertexId,
        player: PlayerId,
    ) -> Result<(), InvalidAction> {
        self.can_upgrade_settlement_to_city(vertex, player)?;
        self.buildings[vertex.value()] = Some(Building::new(BuildingKind::City, player));
        Ok(())
    }

    pub(crate) fn resources_around(&self, vertex: VertexId) -> Vec<Resource> {
        self.topology
            .tile_vertices()
            .iter()
            .enumerate()
            .filter(|(_, vs)| vs.contains(&vertex))
            .filter_map(|(i, _)| self.tiles()[i].resource())
            .collect()
    }

    pub fn steal_victims(&self, player_id: PlayerId) -> Vec<PlayerId> {


        let buildings: Vec<Building> = self.topology().tile_vertices()
            [self.robber().value()]
            .iter()
            .map(|&v| self.buildings()[v.value()])
            .filter(|&o| o.is_some())
            .flatten()
            .collect();

        let mut victims: Vec<PlayerId> = buildings
            .iter()
            .map(|b| b.owner())
            .filter(|victims_id: &PlayerId| victims_id.value() != player_id.value())
            .collect();

        victims.sort_by_key(|p| p.value());
        victims.dedup();
        victims
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::board::production::{Gain, Production};
    use crate::player::PlayerId;
    use crate::NumberToken;

    pub(crate) fn init_board_without_buildings() -> Board {
        let topology = Topology::test_topology();
        Board::new(
            topology,
            vec![
                Tile::Forest(NumberToken::new(6).unwrap()),
                Tile::Hills(NumberToken::new(8).unwrap()),
                Tile::Desert,
            ],
            TileId::new(2),
        )
        .unwrap()
    }

    pub(crate) fn init_board() -> Board {
        let mut board = init_board_without_buildings();
        board
            .place_settlement(
                Board::can_place_settlement_during_placement,
                VertexId::new(4),
                PlayerId::new(1),
            )
            .unwrap();
        board
            .upgrade_settlement_to_city(VertexId::new(4), PlayerId::new(1))
            .unwrap();

        board
            .place_settlement(
                Board::can_place_settlement_during_placement,
                VertexId::new(8),
                PlayerId::new(1),
            )
            .unwrap();
        board.move_robber(TileId::new(1)).unwrap();

        board
            .place_road(
                Board::can_place_road_during_placement,
                EdgeId::new(5),
                PlayerId::new(1),
            )
            .unwrap();

        board
            .place_road(
                Board::can_place_road_during_placement,
                EdgeId::new(9),
                PlayerId::new(1),
            )
            .unwrap();
        board
    }

    #[test]
    fn test_production() {
        let board = init_board();
        let mut expected_production = Production::default();
        expected_production.add_gain(Gain {
            player: PlayerId::new(1),
            resources: ResourceCounts::new([2,0,0,0,0]),

        });
        assert_eq!(
            board.production(Roll::new(2, 4).unwrap()),
            expected_production
        )
    }

    #[test]
    fn test_production_with_robber() {
        let board = init_board();
        let expected_production = Production::default();
        assert_eq!(
            board.production(Roll::new(4, 4).unwrap()),
            expected_production
        )
    }

    #[test]
    fn test_production_with_no_buildings() {
        let board = init_board();
        let expected_production = Production::default();
        assert_eq!(
            board.production(Roll::new(4, 4).unwrap()),
            expected_production
        )
    }

    #[test]
    fn test_can_place_settlement_during_placement_ok() {
        let board = init_board_without_buildings();
        assert_eq!(
            board.can_place_settlement_during_placement(VertexId::new(0), PlayerId::new(1)),
            Ok(())
        );
    }

    #[test]
    fn test_can_place_settlement_during_playing_ko_road() {
        let board = init_board_without_buildings();
        assert_eq!(
            board.can_place_settlement_during_playing(VertexId::new(0), PlayerId::new(1)),
            Err(InvalidAction::NotConnected)
        );
    }

    #[test]
    fn test_can_place_building_ko_neighbor() {
        let board = init_board();
        assert_eq!(
            board.can_place_settlement_during_placement(VertexId::new(3), PlayerId::new(0)),
            Err(InvalidAction::TooCloseToBuilding(VertexId::new(4)))
        );
        assert_eq!(
            board.can_place_settlement_during_playing(VertexId::new(3), PlayerId::new(0)),
            Err(InvalidAction::TooCloseToBuilding(VertexId::new(4)))
        );
    }

    #[test]
    fn test_upgrade_settlement_ok() {
        let mut board = init_board();
        board
            .upgrade_settlement_to_city(VertexId::new(8), PlayerId::new(1))
            .unwrap();
        assert_eq!(board.buildings[8].unwrap().kind(), BuildingKind::City);
    }

    #[test]
    fn test_upgrade_settlement_ko_wrong_player() {
        let mut board = init_board();
        assert!(matches!(
            board.upgrade_settlement_to_city(VertexId::new(8), PlayerId::new(0)),
            Err(InvalidAction::NotYourSettlement(_))
        ));
    }

    #[test]
    fn test_upgrade_settlement_ko_wrong_kind() {
        let mut board = init_board();
        assert!(
            board
                .upgrade_settlement_to_city(VertexId::new(4), PlayerId::new(1))
                .is_err()
        )
    }

    #[test]
    fn test_can_place_road_during_placement_ok() {
        let mut board = init_board();

        assert_eq!(
            board.place_settlement(
                Board::can_place_settlement_during_placement,
                VertexId::new(6),
                PlayerId::new(1)
            ),
            Ok(())
        );

        assert_eq!(
            board.can_place_road_during_placement(
                EdgeId::new(6),
                PlayerId::new(1)
            ),
            Ok(())
        );
    }

    #[test]
    fn test_can_place_road_during_placement_ko() {
        let board = init_board();
        assert_eq!(
            board.can_place_road_during_placement(
                EdgeId::new(9),
                PlayerId::new(0)
            ),
            Err(InvalidAction::EdgeOccupied(EdgeId::new(9)))
        );
        assert_eq!(
            board.can_place_road_during_placement(
                EdgeId::new(11),
                PlayerId::new(1)
            ),
            Err(InvalidAction::RoadMustStartFromNewSettlement)
        );
    }

    #[test]
    fn test_can_place_road_during_placement_ko_two_road_to_the_same_building() {
        let board = init_board();
        assert!(matches!(
            board.can_place_road_during_placement(
                EdgeId::new(4),
                PlayerId::new(1)
            ),
            Err(InvalidAction::RoadMustStartFromNewSettlement)
        ));
        assert!(matches!(
            board.can_place_road_during_placement(
                EdgeId::new(5),
                PlayerId::new(1)
            ),
            Err(InvalidAction::EdgeOccupied(_))
        ));
        assert_eq!(
            board.can_place_road_during_placement(
                EdgeId::new(0),
                PlayerId::new(1)
            ),
            Err(InvalidAction::RoadMustStartFromNewSettlement)
        );
    }

    #[test]
    fn test_can_place_road_during_playing_ko() {
        let board = init_board();
        assert!(matches!(
            board.can_place_road_during_playing(EdgeId::new(13), PlayerId::new(1)),
            Err(InvalidAction::NotConnected)
        ));
        assert!(matches!(
            board.can_place_road_during_playing(EdgeId::new(10), PlayerId::new(0)),
            Err(InvalidAction::NotConnected)
        ));
        assert!(matches!(
            board.can_place_road_during_playing(EdgeId::new(0), PlayerId::new(0)),
            Err(InvalidAction::NotConnected)
        ));
    }

    #[test]
    fn test_can_place_road_during_playing_ok() {
        let board = init_board();
        assert!(
            board
                .can_place_road_during_playing(EdgeId::new(4), PlayerId::new(1))
                .is_ok()
        );
        assert_eq!(
            board.can_place_road_during_playing(EdgeId::new(10), PlayerId::new(1)),
            Ok(())
        );
    }

    #[test]
    fn test_place_road_during_playing_ok() {
        let mut board = init_board();
        board
            .place_road(Board::can_place_road_during_playing, EdgeId::new(0), PlayerId::new(1))
            .unwrap();
        board
            .place_road(Board::can_place_road_during_playing, EdgeId::new(10), PlayerId::new(1))
            .unwrap();
        assert_eq!(board.roads[0].unwrap(), PlayerId::new(1));
        assert_eq!(board.roads[10].unwrap(), PlayerId::new(1));
    }

    #[test]
    fn test_place_road_during_playing_ko() {
        let mut board = init_board();
        assert!(
            board
                .place_road(Board::can_place_road_during_playing, EdgeId::new(3), PlayerId::new(1))
                .is_err()
        );
        assert!(
            board
                .place_road(Board::can_place_road_during_playing, EdgeId::new(7), PlayerId::new(1))
                .is_err()
        );
        assert_eq!(board.roads[3], None);
        assert_eq!(board.roads[7], None);
    }

    #[test]
    fn test_place_road_during_placement_ko() {
        let mut board = init_board();
        assert!(
            board
                .place_road(
                    Board::can_place_road_during_placement,
                    EdgeId::new(0),
                    PlayerId::new(1)
                )
                .is_err()
        );
        assert!(
            board
                .place_road(
                    Board::can_place_road_during_placement,
                    EdgeId::new(4),
                    PlayerId::new(1)
                )
                .is_err()
        );
        assert_eq!(board.roads[0], None);
        assert_eq!(board.roads[4], None);
    }

    #[test]
    fn test_place_road_during_placement_ok() {
        let mut board = init_board();
        assert_eq!(
            board.place_settlement(
                Board::can_place_settlement_during_placement,
                VertexId::new(6),
                PlayerId::new(1)
            ),
            Ok(())
        );

        assert_eq!(
            board.place_road(
                Board::can_place_road_during_placement,
                EdgeId::new(6),
                PlayerId::new(1)
            ),
            Ok(())
        );
        assert_eq!(board.roads[6].unwrap(), PlayerId::new(1));
    }

    #[test]
    fn test_resources_around() {
        let board = init_board();
        let expected_resources: Vec<Resource> = vec![Resource::Wood];
        assert_eq!(board.resources_around(VertexId::new(4)), expected_resources);
        let expected_resources: Vec<Resource> = vec![Resource::Brick];
        assert_eq!(board.resources_around(VertexId::new(8)), expected_resources);
        let expected_resources: Vec<Resource> = vec![Resource::Wood, Resource::Brick];
        assert_eq!(board.resources_around(VertexId::new(0)), expected_resources);
    }
}
