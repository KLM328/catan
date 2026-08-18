mod game_error;

pub use game_error::GameError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use crate::board::BuildingKind;
use crate::{
    Board, Cost, EdgeId, Player, PlayerColor, PlayerId, ResourceCounts, Roll, Scenario,
    Steal, Terrain, TileId, VertexId,
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum StatusKind {
    Starting,
    FirstPlacementSettlement,
    FirstPlacementRoad,
    SecondPlacementSettlement,
    SecondPlacementRoad,
    AwaitingRoll,
    AwaitingDiscard,
    AwaitingSteal,
    AwaitingNewRobberLocation,
    PlayingActions,
    End,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GameStatus {
    Starting,
    FirstPlacementSettlement,
    FirstPlacementRoad,
    SecondPlacementSettlement,
    SecondPlacementRoad,
    AwaitingRoll,
    AwaitingDiscard {
        must_discard: [Option<(PlayerId, u8)>; 6],
    },
    AwaitingSteal,
    AwaitingNewRobberLocation,
    PlayingActions,
    End {
        winner: PlayerId,
    },
}

impl GameStatus {
    pub(crate) fn kind(&self) -> StatusKind {
        match self {
            GameStatus::Starting => StatusKind::Starting,
            GameStatus::FirstPlacementSettlement => StatusKind::FirstPlacementSettlement,
            GameStatus::FirstPlacementRoad => StatusKind::FirstPlacementRoad,
            GameStatus::SecondPlacementSettlement => StatusKind::SecondPlacementSettlement,
            GameStatus::SecondPlacementRoad => StatusKind::SecondPlacementRoad,
            GameStatus::AwaitingRoll => StatusKind::AwaitingRoll,
            GameStatus::AwaitingDiscard { .. } => StatusKind::AwaitingDiscard,
            GameStatus::AwaitingSteal => StatusKind::AwaitingSteal,
            GameStatus::AwaitingNewRobberLocation => StatusKind::AwaitingNewRobberLocation,
            GameStatus::PlayingActions => StatusKind::PlayingActions,
            GameStatus::End { .. } => StatusKind::End,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Serialize, Deserialize, Ord, PartialOrd)]
pub struct GameId {
    id: u32,
}

impl GameId {
    pub fn new(id: u32) -> GameId {
        GameId { id }
    }
}

impl Display for GameId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.id)
    }
}

pub struct Game {
    scenario: Scenario,
    status: GameStatus,
    players: HashMap<PlayerId, Player>,
    next_player_id: usize,
    turn_order: Vec<PlayerId>,
    current_turn: usize,
    board: Option<Board>,
}

impl Game {
    pub fn new(scenario: Scenario) -> Game {
        Game {
            scenario,
            status: GameStatus::Starting,
            turn_order: Vec::new(),
            current_turn: 0,
            players: HashMap::new(),
            board: None,
            next_player_id: 0,
        }
    }

    pub fn add_player(&mut self, player: Player) -> Result<PlayerId, GameError> {
        self.check_status(&[StatusKind::Starting])?;
        if self.players.len() >= self.scenario.max_player() {
            Err(GameError::GameIsFull)
        } else {
            if self
                .players
                .iter()
                .any(|(_, p)| p.color() == player.color())
            {
                Err(GameError::ColorNotAvailable)
            } else {
                self.players
                    .insert(PlayerId::new(self.next_player_id), player);
                self.next_player_id += 1;
                Ok(PlayerId::new(self.next_player_id - 1))
            }
        }
    }

    pub fn remove_player(&mut self, player_id: PlayerId) -> Result<(), GameError> {
        self.check_status(&[StatusKind::Starting])?;
        self.is_player(player_id)?;
        self.players.remove(&player_id);
        Ok(())
    }

    pub fn next_player_color(&self) -> Result<PlayerColor, GameError> {
        PlayerColor::ALL
            .into_iter()
            .find(|&color| self.players.iter().all(|(_, p)| p.color() != color))
            .ok_or(GameError::GameIsFull)
    }

    pub fn players(&self) -> &HashMap<PlayerId, Player> {
        &self.players
    }

    pub fn turn_order(&self) -> &[PlayerId] {
        &self.turn_order
    }

    pub fn scenario(&self) -> &Scenario {
        &self.scenario
    }

    pub fn current_player_index(&self) -> usize {
        self.current_turn
    }

    pub fn board(&self) -> Result<&Board, GameError> {
        self.board.as_ref().ok_or(GameError::GameIsStarting)
    }
    fn board_mut(&mut self) -> Result<&mut Board, GameError> {
        self.board.as_mut().ok_or(GameError::GameIsStarting)
    }
    pub fn start(&mut self, shuffled: &[Terrain]) -> Result<(), GameError> {
        self.check_status(&[StatusKind::Starting])?;
        if self.players.len() < self.scenario.min_player() {
            Err(GameError::NotEnoughPlayers)
        } else if self.players.len() > self.scenario.max_player() {
            Err(GameError::TooManyPlayers)
        } else {
            self.board = Some(self.scenario.layout(shuffled)?);
            self.set_status(GameStatus::FirstPlacementSettlement);
            Ok(())
        }
    }

    pub(crate) fn set_status(&mut self, status: GameStatus) {
        self.status = status;
    }

    pub fn status(&self) -> GameStatus {
        self.status
    }

    pub fn current_player(&self) -> PlayerId {
        self.turn_order[self.current_turn]
    }

    fn end_placement_turn(&mut self) {
        match self.status {
            GameStatus::FirstPlacementRoad => {
                if self.current_turn == self.turn_order.len() - 1 {
                    self.status = GameStatus::SecondPlacementSettlement;
                } else {
                    self.current_turn += 1;
                    self.status = GameStatus::FirstPlacementSettlement;
                }
            }
            GameStatus::SecondPlacementRoad => {
                if self.current_turn == 0 {
                    self.status = GameStatus::AwaitingRoll;
                } else {
                    self.current_turn -= 1;
                    self.status = GameStatus::SecondPlacementSettlement;
                }
            }
            _ => {}
        }
    }

    pub fn next_player(&mut self, player_id: PlayerId) -> Result<(), GameError> {
        self.check_player(player_id)?;
        match self.status {
            GameStatus::PlayingActions => {
                self.current_turn = (self.current_turn + 1) % self.turn_order.len();
                self.set_status(GameStatus::AwaitingRoll);
                Ok(())
            }
            GameStatus::FirstPlacementSettlement
            | GameStatus::FirstPlacementRoad
            | GameStatus::SecondPlacementSettlement
            | GameStatus::SecondPlacementRoad => Err(GameError::TurnDrivenByPlacement),
            _ => Err(GameError::InvalidGameStatus),
        }
    }

    pub fn set_players_order(&mut self, rolls: &[(PlayerId, Roll)]) -> Result<(), GameError> {
        self.check_status(&[StatusKind::Starting])?;
        if rolls.len() == self.players.len() {
            let best = rolls.iter().map(|(_, r)| r.value()).max().unwrap();
            if rolls.iter().filter(|(_, r)| r.value() == best).count() > 1 {
                Err(GameError::TiedRolls)
            } else {
                let first = rolls.iter().position(|(_, r)| r.value() == best).unwrap();
                let n = self.players.len();
                self.turn_order = (0..n).map(|k| rolls[(first + k) % n].0).collect();
                Ok(())
            }
        } else {
            Err(GameError::WrongRollCount)
        }
    }

    fn check_status(&self, authorized_status: &[StatusKind]) -> Result<(), GameError> {
        if authorized_status.contains(&self.status.kind()) {
            Ok(())
        } else {
            Err(GameError::InvalidGameStatus)
        }
    }

    pub fn apply_roll(&mut self, player_id: PlayerId, roll: Roll) -> Result<(), GameError> {
        self.check_status(&[StatusKind::AwaitingRoll])?;
        self.check_player(player_id)?;
        match roll.value() {
            7 => {
                let mut must_discard = [None; 6];
                self.sorted_player()
                    .iter()
                    .enumerate()
                    .filter(|&(_, (_, p))| p.hand().count() > 7)
                    .for_each(|(index, &(id, p))| {
                        must_discard[index] = Some((id, p.hand().count() / 2))
                    });

                if must_discard.iter().all(|&o| o.is_none()) {
                    self.set_status(GameStatus::AwaitingNewRobberLocation)
                } else {
                    self.set_status(GameStatus::AwaitingDiscard { must_discard })
                }
            }
            _ => {
                self.set_status(GameStatus::PlayingActions);
                self.board()?
                    .production(roll)
                    .gains()
                    .iter()
                    .for_each(|gain| {
                        self.players
                            .get_mut(&gain.player)
                            .unwrap()
                            .receive(gain.resources)
                    });
            }
        };

        Ok(())
    }

    pub fn sorted_player(&self) -> Vec<(PlayerId, &Player)> {
        let mut players: Vec<(PlayerId, &Player)> =
            self.players.iter().map(|(&id, p)| (id, p)).collect();
        players.sort_by_key(|&(id, _)| id);
        players
    }

    pub fn build_road(&mut self, player_id: PlayerId, edge: EdgeId) -> Result<(), GameError> {
        self.check_status(&[
            StatusKind::FirstPlacementRoad,
            StatusKind::SecondPlacementRoad,
            StatusKind::PlayingActions,
        ])?;

        self.check_player(player_id)?;

        match self.status {
            GameStatus::FirstPlacementRoad | GameStatus::SecondPlacementRoad => {
                self.board_mut()?.place_road(
                    Board::can_place_road_during_placement,
                    edge,
                    player_id,
                )?;
                self.end_placement_turn();
                Ok(())
            }
            GameStatus::PlayingActions => {
                self.board()?
                    .can_place_road_during_playing(edge, player_id)?;
                self.get_player_mut(player_id)?.pay(&Cost::ROAD)?;
                self.board_mut()?.place_road(
                    Board::can_place_road_during_playing,
                    edge,
                    player_id,
                )?;
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    pub fn build_settlement(
        &mut self,
        player_id: PlayerId,
        vertex: VertexId,
    ) -> Result<(), GameError> {
        self.check_status(&[
            StatusKind::FirstPlacementSettlement,
            StatusKind::SecondPlacementSettlement,
            StatusKind::PlayingActions,
        ])?;

        self.check_player(player_id)?;

        match self.status {
            GameStatus::FirstPlacementSettlement => {
                self.board_mut()?.place_settlement(
                    Board::can_place_settlement_during_placement,
                    vertex,
                    player_id,
                )?;
                self.edit_score(
                    Player::add_score,
                    player_id,
                    BuildingKind::Settlement.points(),
                )?;
                self.set_status(GameStatus::FirstPlacementRoad);
                Ok(())
            }

            GameStatus::SecondPlacementSettlement => {
                self.board_mut()?.place_settlement(
                    Board::can_place_settlement_during_placement,
                    vertex,
                    player_id,
                )?;
                self.edit_score(
                    Player::add_score,
                    player_id,
                    BuildingKind::Settlement.points(),
                )?;
                let mut resources = [0; 5];
                for r in self.board()?.resources_around(vertex) {
                    resources[r.index()] += 1;
                }
                self.get_player_mut(self.current_player())?
                    .receive(ResourceCounts::new(resources));
                self.set_status(GameStatus::SecondPlacementRoad);
                Ok(())
            }
            GameStatus::PlayingActions => {
                self.board()?
                    .can_place_settlement_during_playing(vertex, player_id)?;
                self.get_player_mut(player_id)?.pay(&Cost::SETTLEMENT)?;
                self.board_mut()?.place_settlement(
                    Board::can_place_settlement_during_playing,
                    vertex,
                    player_id,
                )?;
                self.edit_score(
                    Player::add_score,
                    player_id,
                    BuildingKind::Settlement.points(),
                )?;
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    fn check_player(&self, player_id: PlayerId) -> Result<(), GameError> {
        if player_id == self.current_player() {
            Ok(())
        } else {
            Err(GameError::NotYourTurn)
        }
    }

    fn is_player(&self, player_id: PlayerId) -> Result<(), GameError> {
        if self.players.contains_key(&player_id) {
            Ok(())
        } else {
            Err(GameError::PlayerNotFound(player_id))
        }
    }

    fn get_player_mut(&mut self, player_id: PlayerId) -> Result<&mut Player, GameError> {
        let Some(player) = self.players.get_mut(&player_id) else {
            return Err(GameError::PlayerNotFound(player_id));
        };
        Ok(player)
    }

    pub fn get_player(&self, player_id: PlayerId) -> Result<&Player, GameError> {
        let Some(player) = self.players.get(&player_id) else {
            return Err(GameError::PlayerNotFound(player_id));
        };
        Ok(player)
    }

    pub fn upgrade_settlement_to_city(
        &mut self,
        player_id: PlayerId,
        vertex: VertexId,
    ) -> Result<(), GameError> {
        self.check_status(&[StatusKind::PlayingActions])?;
        self.check_player(player_id)?;

        self.board_mut()?
            .can_upgrade_settlement_to_city(vertex, player_id)?;

        self.get_player_mut(player_id)?.pay(&Cost::CITY)?;

        self.board_mut()?
            .upgrade_settlement_to_city(vertex, player_id)?;
        self.edit_score(
            Player::remove_score,
            player_id,
            BuildingKind::Settlement.points(),
        )?;
        self.edit_score(Player::add_score, player_id, BuildingKind::City.points())?;

        Ok(())
    }

    pub fn move_robber(&mut self, player_id: PlayerId, tile: TileId) -> Result<(), GameError> {
        self.check_status(&[StatusKind::AwaitingNewRobberLocation])?;
        self.check_player(player_id)?;
        self.board_mut()?.move_robber(tile)?;
        self.set_status(GameStatus::AwaitingSteal);
        Ok(())
    }

    pub fn discard(
        &mut self,
        player_id: PlayerId,
        resources: ResourceCounts,
    ) -> Result<(), GameError> {
        self.check_status(&[StatusKind::AwaitingDiscard])?;
        self.is_player(player_id)?;
        if let GameStatus::AwaitingDiscard { mut must_discard } = self.status() {
            if let Some(index) = must_discard
                .iter()
                .position(|o| o.is_some_and(|(p, _)| p == player_id))
            {
                if must_discard[index].unwrap().1 == resources.count() {
                    self.get_player_mut(player_id)?.pay(&Cost::new(resources))?;
                    must_discard[index] = None;
                } else {
                    return Err(GameError::InvalidDiscardCount);
                }

                if must_discard.iter().all(|&o| o.is_none()) {
                    self.set_status(GameStatus::AwaitingNewRobberLocation)
                } else {
                    self.set_status(GameStatus::AwaitingDiscard { must_discard })
                }

                Ok(())
            } else {
                Err(GameError::PlayerDontNeedToDiscard)
            }
        } else {
            Err(GameError::InvalidGameStatus)
        }
    }

    fn edit_score(
        &mut self,
        fun: impl Fn(&mut Player, u8),
        player_id: PlayerId,
        amount: u8,
    ) -> Result<(), GameError> {
        self.check_status(&[
            StatusKind::PlayingActions,
            StatusKind::FirstPlacementSettlement,
            StatusKind::SecondPlacementSettlement,
        ])?;
        self.check_player(player_id)?;
        fun(self.get_player_mut(player_id)?, amount);
        self.check_victory()?;
        Ok(())
    }

    fn check_victory(&mut self) -> Result<(), GameError> {
        let winner = self
            .players
            .iter()
            .find(|&(_, p)| p.score() >= self.scenario.max_points());

        if let Some((&id, _)) = winner {
            self.set_status(GameStatus::End { winner: id });
        }

        Ok(())
    }

    pub fn steal(&mut self, player_id: PlayerId, steal: &Option<Steal>) -> Result<(), GameError> {
        self.check_status(&[StatusKind::AwaitingSteal])?;
        self.check_player(player_id)?;
        let victims: Vec<PlayerId> = self
            .board()?
            .steal_victims(player_id)
            .into_iter()
            .filter(|&p| !self.get_player(p).unwrap().hand().is_empty())
            .collect();

        match steal {
            None => {
                if victims.is_empty() {
                    self.set_status(GameStatus::PlayingActions);
                    Ok(())
                } else {
                    Err(GameError::MustStealSomeone)
                }
            }
            Some(steal) => {
                if victims.is_empty() {
                    Err(GameError::NoOneToSteal)
                } else if victims.contains(&steal.victim()) {
                    let resource: ResourceCounts = steal.resource();
                    self.get_player_mut(steal.victim())?
                        .pay(&Cost::new(resource))?;
                    self.get_player_mut(player_id)?.receive(resource);
                    self.set_status(GameStatus::PlayingActions);
                    Ok(())
                } else {
                    Err(GameError::UnauthorizedVictim)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::PlayerColor;
    use crate::{
        Building, InvalidAction, InvalidBoard, NumberToken, ResourceCounts, ResourceError, Tile,
    };

    #[test]
    fn init_game() {
        let mut game = Game::new(Scenario::test_scenario());

        assert_eq!(
            game.add_player(Player::new(PlayerColor::White)),
            Ok(PlayerId::new(0))
        );
        assert_eq!(
            game.start(&game.scenario.terrains().to_vec()),
            Err(GameError::NotEnoughPlayers)
        );
        assert_eq!(
            game.add_player(Player::new(PlayerColor::White)),
            Err(GameError::ColorNotAvailable)
        );
        assert_eq!(
            game.start(&game.scenario.terrains().to_vec()),
            Err(GameError::NotEnoughPlayers)
        );
        assert_eq!(
            game.add_player(Player::new(PlayerColor::Red)),
            Ok(PlayerId::new(1))
        );
        assert_eq!(game.start(&game.scenario.terrains().to_vec()), Ok(()));

        let mut game = Game::new(Scenario::test_scenario());
        assert_eq!(
            game.add_player(Player::new(PlayerColor::White)),
            Ok(PlayerId::new(0))
        );
        assert_eq!(
            game.add_player(Player::new(PlayerColor::Red)),
            Ok(PlayerId::new(1))
        );
        assert_eq!(
            game.add_player(Player::new(PlayerColor::Blue)),
            Ok(PlayerId::new(2))
        );
        assert_eq!(
            game.add_player(Player::new(PlayerColor::Orange)),
            Ok(PlayerId::new(3))
        );
        assert_eq!(
            game.add_player(Player::new(PlayerColor::Brown)),
            Err(GameError::GameIsFull)
        );

        let mut game = Game::new(Scenario::test_scenario());
        assert_eq!(
            game.add_player(Player::new(PlayerColor::White)),
            Ok(PlayerId::new(0))
        );
        assert_eq!(
            game.add_player(Player::new(PlayerColor::Red)),
            Ok(PlayerId::new(1))
        );
        assert_eq!(
            game.add_player(Player::new(PlayerColor::Blue)),
            Ok(PlayerId::new(2))
        );

        assert_eq!(
            game.set_players_order(&[(PlayerId::new(0), Roll::new(2, 4).unwrap())]),
            Err(GameError::WrongRollCount)
        );
        assert_eq!(
            game.set_players_order(&[
                (PlayerId::new(0), Roll::new(2, 4).unwrap()),
                (PlayerId::new(1), Roll::new(2, 5).unwrap()),
                (PlayerId::new(2), Roll::new(3, 4).unwrap()),
            ]),
            Err(GameError::TiedRolls)
        );
        assert_eq!(
            game.set_players_order(&[
                (PlayerId::new(0), Roll::new(2, 4).unwrap()),
                (PlayerId::new(1), Roll::new(6, 2).unwrap()),
                (PlayerId::new(2), Roll::new(3, 4).unwrap()),
            ]),
            Ok(())
        );
        assert_eq!(
            game.turn_order,
            vec![PlayerId::new(1), PlayerId::new(2), PlayerId::new(0)]
        );
        assert_eq!(game.current_player(), PlayerId::new(1));

        assert_eq!(
            game.next_player(game.current_player()),
            Err(GameError::InvalidGameStatus)
        );
        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(4, 2).unwrap()),
            Err(GameError::InvalidGameStatus)
        );

        assert_eq!(game.status(), GameStatus::Starting);

        assert_eq!(
            game.start(&vec![]),
            Err(GameError::BoardInitialization(
                InvalidBoard::WrongTileCount {
                    expected: 3,
                    got: 0
                }
            ))
        );
        assert_eq!(
            game.start(&vec![Terrain::Desert]),
            Err(GameError::BoardInitialization(
                InvalidBoard::WrongTileCount {
                    expected: 3,
                    got: 1
                }
            ))
        );
        assert_eq!(
            game.start(&vec![Terrain::Desert, Terrain::Forest]),
            Err(GameError::BoardInitialization(
                InvalidBoard::WrongTileCount {
                    expected: 3,
                    got: 2
                }
            ))
        );
        assert_eq!(
            game.start(&vec![
                Terrain::Desert,
                Terrain::Forest,
                Terrain::Mountain,
                Terrain::Fields
            ]),
            Err(GameError::BoardInitialization(
                InvalidBoard::WrongTileCount {
                    expected: 3,
                    got: 4
                }
            ))
        );
        assert_eq!(
            game.start(&vec![Terrain::Desert, Terrain::Forest, Terrain::Mountain]),
            Err(GameError::BoardInitialization(
                InvalidBoard::WrongDistribution
            ))
        );
        assert_eq!(game.start(&game.scenario.terrains().to_vec()), Ok(()));
        assert_eq!(
            game.board().unwrap().tiles(),
            vec![
                Tile::Forest(NumberToken::new(6).unwrap()),
                Tile::Hills(NumberToken::new(8).unwrap()),
                Tile::Desert
            ]
        );
    }

    #[test]
    fn partie_complete() {
        let mut game = Game::new(Scenario::fast_standard());

        assert_eq!(
            game.add_player(Player::new(PlayerColor::White)),
            Ok(PlayerId::new(0))
        );
        assert_eq!(
            game.add_player(Player::new(PlayerColor::Red)),
            Ok(PlayerId::new(1))
        );
        assert_eq!(
            game.add_player(Player::new(PlayerColor::Blue)),
            Ok(PlayerId::new(2))
        );

        assert_eq!(
            game.remove_player(PlayerId::new(3)),
            Err(GameError::PlayerNotFound(PlayerId::new(3)))
        );

        assert_eq!(game.remove_player(PlayerId::new(0)), Ok(()));
        assert_eq!(
            game.add_player(Player::new(PlayerColor::White)),
            Ok(PlayerId::new(3))
        );

        assert_eq!(
            game.set_players_order(&[
                (PlayerId::new(1), Roll::new(6, 2).unwrap()),
                (PlayerId::new(2), Roll::new(3, 4).unwrap()),
                (PlayerId::new(3), Roll::new(1, 4).unwrap()),
            ]),
            Ok(())
        );
        assert_eq!(game.start(&game.scenario.terrains().to_vec()), Ok(()));

        assert_eq!(game.players[&PlayerId::new(1)].score(), 0);

        assert_eq!(game.status(), GameStatus::FirstPlacementSettlement);
        assert_eq!(
            game.next_player(game.current_player()),
            Err(GameError::TurnDrivenByPlacement)
        );
        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(4, 2).unwrap()),
            Err(GameError::InvalidGameStatus)
        );

        assert_eq!(game.current_player(), PlayerId::new(1));
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(46)),
            Err(GameError::InvalidGameStatus)
        );
        assert_eq!(
            game.build_settlement(PlayerId::new(2), VertexId::new(12)),
            Err(GameError::NotYourTurn)
        );
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(120)),
            Err(GameError::Placement(InvalidAction::UnexistingVertex(
                VertexId::new(120)
            )))
        );

        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(1)),
            Ok(())
        );
        assert_eq!(game.players[&PlayerId::new(1)].score(), 1);

        assert_eq!(game.current_player(), PlayerId::new(1));
        assert_eq!(game.status(), GameStatus::FirstPlacementRoad);
        assert_eq!(
            game.next_player(game.current_player()),
            Err(GameError::TurnDrivenByPlacement)
        );
        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(4, 2).unwrap()),
            Err(GameError::InvalidGameStatus)
        );

        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(12)),
            Err(GameError::InvalidGameStatus)
        );
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(899)),
            Err(GameError::Placement(InvalidAction::UnexistingEdge(
                EdgeId::new(899)
            )))
        );
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(60)),
            Err(GameError::Placement(
                InvalidAction::RoadMustStartFromNewSettlement
            ))
        );
        assert_eq!(
            game.build_road(PlayerId::new(2), EdgeId::new(1)),
            Err(GameError::NotYourTurn)
        );
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(1)),
            Ok(())
        );

        assert_eq!(game.status(), GameStatus::FirstPlacementSettlement);
        assert_eq!(game.current_player(), PlayerId::new(2));
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(46)),
            Err(GameError::Placement(InvalidAction::TooCloseToBuilding(
                VertexId::new(1)
            )))
        );
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(1)),
            Err(GameError::Placement(InvalidAction::VertexOccupied(
                VertexId::new(1)
            )))
        );
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(11)),
            Ok(())
        );

        assert_eq!(game.status(), GameStatus::FirstPlacementRoad);
        assert_eq!(game.current_player(), PlayerId::new(2));
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(1)),
            Err(GameError::Placement(InvalidAction::EdgeOccupied(
                EdgeId::new(1)
            )))
        );
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(2)),
            Err(GameError::Placement(
                InvalidAction::RoadMustStartFromNewSettlement
            ))
        );
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(19)),
            Ok(())
        );

        assert_eq!(game.status(), GameStatus::FirstPlacementSettlement);
        assert_eq!(game.current_player(), PlayerId::new(3));
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(40)),
            Ok(())
        );

        assert_eq!(game.status(), GameStatus::FirstPlacementRoad);
        assert_eq!(game.current_player(), PlayerId::new(3));
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(50)),
            Ok(())
        );

        assert_eq!(game.status(), GameStatus::SecondPlacementSettlement);
        assert_eq!(
            game.next_player(game.current_player()),
            Err(GameError::TurnDrivenByPlacement)
        );
        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(4, 2).unwrap()),
            Err(GameError::InvalidGameStatus)
        );

        assert_eq!(game.current_player(), PlayerId::new(3));
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(20)),
            Ok(())
        );
        assert_eq!(
            game.players[&game.current_player()].hand().resources(),
            ResourceCounts::new([1, 0, 0, 1, 1])
        );

        assert_eq!(game.status(), GameStatus::SecondPlacementRoad);
        assert_eq!(
            game.next_player(game.current_player()),
            Err(GameError::TurnDrivenByPlacement)
        );
        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(4, 2).unwrap()),
            Err(GameError::InvalidGameStatus)
        );

        assert_eq!(game.current_player(), PlayerId::new(3));
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(49)),
            Err(GameError::Placement(
                InvalidAction::RoadMustStartFromNewSettlement
            ))
        );
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(46)),
            Err(GameError::Placement(
                InvalidAction::RoadMustStartFromNewSettlement
            ))
        );
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(23)),
            Ok(())
        );

        assert_eq!(game.status(), GameStatus::SecondPlacementSettlement);
        assert_eq!(game.current_player(), PlayerId::new(2));
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(44)),
            Ok(())
        );
        assert_eq!(
            game.players[&game.current_player()].hand().resources(),
            ResourceCounts::new([0, 0, 1, 1, 0])
        );

        assert_eq!(game.status(), GameStatus::SecondPlacementRoad);
        assert_eq!(game.current_player(), PlayerId::new(2));
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(54)),
            Ok(())
        );

        assert_eq!(game.status(), GameStatus::SecondPlacementSettlement);
        assert_eq!(game.current_player(), PlayerId::new(1));
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(33)),
            Ok(())
        );
        assert_eq!(game.players[&PlayerId::new(1)].score(), 2);
        assert_eq!(
            game.players[&game.current_player()].hand().resources(),
            ResourceCounts::new([1, 1, 0, 0, 1])
        );

        assert_eq!(game.status(), GameStatus::SecondPlacementRoad);
        assert_eq!(game.current_player(), PlayerId::new(1));
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(67)),
            Ok(())
        );

        assert_eq!(game.status(), GameStatus::AwaitingRoll);
        assert_eq!(game.current_player(), PlayerId::new(1));
        assert_eq!(
            game.build_road(PlayerId::new(2), EdgeId::new(8)),
            Err(GameError::InvalidGameStatus)
        );
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(7)),
            Err(GameError::InvalidGameStatus)
        );
        assert_eq!(
            game.next_player(game.current_player()),
            Err(GameError::InvalidGameStatus)
        );
        assert_eq!(
            game.upgrade_settlement_to_city(game.current_player(), VertexId::new(1)),
            Err(GameError::InvalidGameStatus)
        );

        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(4, 5).unwrap()),
            Ok(())
        );

        assert_eq!(game.status(), GameStatus::PlayingActions);

        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(4, 5).unwrap()),
            Err(GameError::InvalidGameStatus)
        );

        assert_eq!(game.current_player(), PlayerId::new(1));
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([1, 1, 1, 0, 1])
        );
        assert_eq!(
            game.build_road(PlayerId::new(2), EdgeId::new(8)),
            Err(GameError::NotYourTurn)
        );
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(8)),
            Ok(())
        );
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([0, 1, 0, 0, 1])
        );
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(7)),
            Err(GameError::Resource(ResourceError::NotEnoughResources))
        );
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(7)),
            Err(GameError::Resource(ResourceError::NotEnoughResources))
        );

        assert_eq!(game.next_player(game.current_player()), Ok(()));
        assert_eq!(game.current_player(), PlayerId::new(2));
        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(3, 6).unwrap()),
            Ok(())
        );
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([0, 1, 1, 0, 1])
        );

        assert_eq!(game.next_player(game.current_player()), Ok(()));
        assert_eq!(game.current_player(), PlayerId::new(3));
        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(3, 3).unwrap()),
            Ok(())
        );
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([0, 2, 1, 0, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(2)].hand().resources(),
            ResourceCounts::new([1, 0, 1, 1, 0])
        );

        assert_eq!(game.next_player(game.current_player()), Ok(()));
        assert_eq!(game.current_player(), PlayerId::new(1));
        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(6, 4).unwrap()),
            Ok(())
        );
        assert_eq!(
            game.players[&PlayerId::new(3)].hand().resources(),
            ResourceCounts::new([2, 0, 0, 1, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([0, 2, 1, 1, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(2)].hand().resources(),
            ResourceCounts::new([1, 0, 1, 2, 0])
        );

        assert_eq!(game.next_player(game.current_player()), Ok(()));
        assert_eq!(game.current_player(), PlayerId::new(2));
        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(4, 6).unwrap()),
            Ok(())
        );
        assert_eq!(
            game.players[&PlayerId::new(3)].hand().resources(),
            ResourceCounts::new([3, 0, 0, 1, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([0, 2, 1, 2, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(2)].hand().resources(),
            ResourceCounts::new([1, 0, 1, 3, 0])
        );

        assert_eq!(game.next_player(game.current_player()), Ok(()));
        assert_eq!(game.current_player(), PlayerId::new(3));
        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(4, 2).unwrap()),
            Ok(())
        );
        assert_eq!(
            game.players[&PlayerId::new(3)].hand().resources(),
            ResourceCounts::new([3, 0, 0, 1, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([0, 3, 1, 2, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(2)].hand().resources(),
            ResourceCounts::new([2, 0, 1, 3, 0])
        );

        assert_eq!(game.next_player(game.current_player()), Ok(()));
        assert_eq!(game.current_player(), PlayerId::new(1));
        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(3, 2).unwrap()),
            Ok(())
        );
        assert_eq!(
            game.players[&PlayerId::new(3)].hand().resources(),
            ResourceCounts::new([3, 0, 0, 2, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([1, 3, 1, 2, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(2)].hand().resources(),
            ResourceCounts::new([2, 0, 1, 3, 0])
        );
        assert_eq!(
            game.upgrade_settlement_to_city(game.current_player(), VertexId::new(20)),
            Err(GameError::Placement(InvalidAction::NotYourSettlement(
                VertexId::new(20)
            )))
        );
        assert_eq!(
            game.upgrade_settlement_to_city(game.current_player(), VertexId::new(2)),
            Err(GameError::Placement(InvalidAction::UnexistingBuilding(
                VertexId::new(2)
            )))
        );
        assert_eq!(
            game.upgrade_settlement_to_city(game.current_player(), VertexId::new(1)),
            Ok(())
        );
        assert_eq!(game.players[&PlayerId::new(1)].score(), 3);
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([1, 0, 1, 0, 1])
        );

        assert_eq!(game.next_player(game.current_player()), Ok(()));
        assert_eq!(game.status(), GameStatus::AwaitingRoll);
        assert_eq!(game.current_player(), PlayerId::new(2));

        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(3, 4).unwrap()),
            Ok(())
        );
        assert_eq!(game.status(), GameStatus::AwaitingNewRobberLocation);
        assert_eq!(
            game.next_player(game.current_player()),
            Err(GameError::InvalidGameStatus)
        );
        assert_eq!(
            game.upgrade_settlement_to_city(game.current_player(), VertexId::new(20)),
            Err(GameError::InvalidGameStatus)
        );
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(20)),
            Err(GameError::InvalidGameStatus)
        );
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(20)),
            Err(GameError::InvalidGameStatus)
        );
        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(3, 4).unwrap()),
            Err(GameError::InvalidGameStatus)
        );
        assert_eq!(
            game.move_robber(game.current_player(), TileId::new(18)),
            Err(GameError::Placement(InvalidAction::RobberMustMove))
        );

        assert_eq!(
            game.move_robber(game.current_player(), TileId::new(4)),
            Ok(())
        );

        assert_eq!(game.status(), GameStatus::AwaitingSteal);
        assert_eq!(
            game.next_player(game.current_player()),
            Err(GameError::InvalidGameStatus)
        );
        assert_eq!(
            game.upgrade_settlement_to_city(game.current_player(), VertexId::new(20)),
            Err(GameError::InvalidGameStatus)
        );
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(20)),
            Err(GameError::InvalidGameStatus)
        );
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(20)),
            Err(GameError::InvalidGameStatus)
        );
        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(3, 4).unwrap()),
            Err(GameError::InvalidGameStatus)
        );

        assert_eq!(
            game.steal(
                game.current_player(),
                &Some(Steal::new(PlayerId::new(1), ResourceCounts::default()))
            ),
            Err(GameError::UnauthorizedVictim)
        );

        assert_eq!(
            game.steal(game.current_player(), &None),
            Err(GameError::MustStealSomeone)
        );

        assert_eq!(
            game.players[&PlayerId::new(3)].hand().resources(),
            ResourceCounts::new([3, 0, 0, 2, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([1, 0, 1, 0, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(2)].hand().resources(),
            ResourceCounts::new([2, 0, 1, 3, 0])
        );

        assert_eq!(
            game.steal(
                game.current_player(),
                &Some(Steal::new(
                    PlayerId::new(3),
                    ResourceCounts::new([1, 0, 0, 0, 0])
                ))
            ),
            Ok(())
        );

        assert_eq!(
            game.players[&PlayerId::new(3)].hand().resources(),
            ResourceCounts::new([2, 0, 0, 2, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([1, 0, 1, 0, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(2)].hand().resources(),
            ResourceCounts::new([3, 0, 1, 3, 0])
        );

        assert_eq!(game.status(), GameStatus::PlayingActions);
        assert_eq!(game.current_player(), PlayerId::new(2));

        assert_eq!(
            game.players[&PlayerId::new(2)].hand().resources(),
            ResourceCounts::new([3, 0, 1, 3, 0])
        );
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(18)),
            Ok(())
        );
        assert_eq!(
            game.players[&PlayerId::new(3)].hand().resources(),
            ResourceCounts::new([2, 0, 0, 2, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([1, 0, 1, 0, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(2)].hand().resources(),
            ResourceCounts::new([2, 0, 0, 3, 0])
        );

        assert_eq!(game.next_player(game.current_player()), Ok(()));
        assert_eq!(game.status(), GameStatus::AwaitingRoll);
        assert_eq!(game.current_player(), PlayerId::new(3));
        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(3, 6).unwrap()),
            Ok(())
        );
        assert_eq!(game.status(), GameStatus::PlayingActions);
        assert_eq!(
            game.players[&PlayerId::new(3)].hand().resources(),
            ResourceCounts::new([2, 0, 0, 2, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([1, 0, 3, 0, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(2)].hand().resources(),
            ResourceCounts::new([2, 0, 0, 3, 0])
        );

        assert_eq!(game.next_player(game.current_player()), Ok(()));
        assert_eq!(game.status(), GameStatus::AwaitingRoll);
        assert_eq!(game.current_player(), PlayerId::new(1));
        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(3, 2).unwrap()),
            Ok(())
        );
        assert_eq!(
            game.players[&PlayerId::new(3)].hand().resources(),
            ResourceCounts::new([2, 0, 0, 3, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([3, 0, 3, 0, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(2)].hand().resources(),
            ResourceCounts::new([2, 0, 0, 3, 0])
        );

        assert_eq!(game.next_player(game.current_player()), Ok(()));
        assert_eq!(game.status(), GameStatus::AwaitingRoll);
        assert_eq!(game.current_player(), PlayerId::new(2));
        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(3, 2).unwrap()),
            Ok(())
        );
        assert_eq!(
            game.players[&PlayerId::new(3)].hand().resources(),
            ResourceCounts::new([2, 0, 0, 4, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([5, 0, 3, 0, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(2)].hand().resources(),
            ResourceCounts::new([2, 0, 0, 3, 0])
        );

        assert_eq!(game.next_player(game.current_player()), Ok(()));
        assert_eq!(game.status(), GameStatus::AwaitingRoll);
        assert_eq!(game.current_player(), PlayerId::new(3));

        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(5, 5).unwrap()),
            Ok(())
        );

        assert_eq!(
            game.players[&PlayerId::new(3)].hand().resources(),
            ResourceCounts::new([3, 0, 0, 4, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([5, 0, 3, 2, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(2)].hand().resources(),
            ResourceCounts::new([2, 0, 0, 4, 0])
        );

        assert_eq!(game.next_player(game.current_player()), Ok(()));
        assert_eq!(game.status(), GameStatus::AwaitingRoll);
        assert_eq!(game.current_player(), PlayerId::new(1));

        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(1, 6).unwrap()),
            Ok(())
        );
        assert_eq!(
            game.status(),
            GameStatus::AwaitingDiscard {
                must_discard: [
                    Some((PlayerId::new(1), 5)),
                    None,
                    Some((PlayerId::new(3), 4)),
                    None,
                    None,
                    None
                ]
            }
        );
        assert_eq!(
            game.discard(PlayerId::new(8), ResourceCounts::new([0, 0, 0, 0, 0])),
            Err(GameError::PlayerNotFound(PlayerId::new(8)))
        );
        assert_eq!(
            game.discard(PlayerId::new(4), ResourceCounts::new([0, 0, 0, 0, 0])),
            Err(GameError::PlayerNotFound(PlayerId::new(4)))
        );
        assert_eq!(
            game.discard(PlayerId::new(2), ResourceCounts::new([0, 0, 0, 0, 0])),
            Err(GameError::PlayerDontNeedToDiscard)
        );
        assert_eq!(
            game.status(),
            GameStatus::AwaitingDiscard {
                must_discard: [
                    Some((PlayerId::new(1), 5)),
                    None,
                    Some((PlayerId::new(3), 4)),
                    None,
                    None,
                    None
                ]
            }
        );
        assert_eq!(
            game.next_player(game.current_player()),
            Err(GameError::InvalidGameStatus)
        );
        assert_eq!(
            game.upgrade_settlement_to_city(game.current_player(), VertexId::new(20)),
            Err(GameError::InvalidGameStatus)
        );
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(20)),
            Err(GameError::InvalidGameStatus)
        );
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(20)),
            Err(GameError::InvalidGameStatus)
        );
        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(3, 4).unwrap()),
            Err(GameError::InvalidGameStatus)
        );

        assert_eq!(
            game.discard(PlayerId::new(3), ResourceCounts::new([7, 0, 0, 0, 0])),
            Err(GameError::InvalidDiscardCount)
        );
        assert_eq!(
            game.discard(PlayerId::new(3), ResourceCounts::new([1, 0, 0, 0, 0])),
            Err(GameError::InvalidDiscardCount)
        );
        assert_eq!(
            game.discard(PlayerId::new(3), ResourceCounts::new([4, 0, 0, 0, 0])),
            Err(GameError::Resource(ResourceError::NotEnoughResources))
        );

        assert_eq!(
            game.discard(PlayerId::new(3), ResourceCounts::new([2, 0, 0, 2, 0])),
            Ok(())
        );
        assert_eq!(
            game.status(),
            GameStatus::AwaitingDiscard {
                must_discard: [Some((PlayerId::new(1), 5)), None, None, None, None, None]
            }
        );
        assert_eq!(
            game.discard(PlayerId::new(3), ResourceCounts::new([0, 0, 0, 0, 0])),
            Err(GameError::PlayerDontNeedToDiscard)
        );
        assert_eq!(
            game.discard(PlayerId::new(1), ResourceCounts::new([5, 0, 0, 0, 0])),
            Ok(())
        );
        assert_eq!(game.status(), GameStatus::AwaitingNewRobberLocation);

        assert_eq!(
            game.players[&PlayerId::new(3)].hand().resources(),
            ResourceCounts::new([1, 0, 0, 2, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([0, 0, 3, 2, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(2)].hand().resources(),
            ResourceCounts::new([2, 0, 0, 4, 0])
        );

        assert_eq!(
            game.move_robber(game.current_player(), game.board().unwrap().robber()),
            Err(GameError::Placement(InvalidAction::RobberMustMove))
        );
        assert_eq!(
            game.move_robber(game.current_player(), TileId::new(8)),
            Ok(())
        );

        assert_eq!(game.status(), GameStatus::AwaitingSteal);

        assert_eq!(game.steal(game.current_player(), &None), Ok(()));

        assert_eq!(
            game.players[&PlayerId::new(3)].hand().resources(),
            ResourceCounts::new([1, 0, 0, 2, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([0, 0, 3, 2, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(2)].hand().resources(),
            ResourceCounts::new([2, 0, 0, 4, 0])
        );
        assert_eq!(game.next_player(game.current_player()), Ok(()));
        assert_eq!(game.status(), GameStatus::AwaitingRoll);
        assert_eq!(game.current_player(), PlayerId::new(2));

        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(4, 1).unwrap()),
            Ok(())
        );

        assert_eq!(
            game.players[&PlayerId::new(3)].hand().resources(),
            ResourceCounts::new([1, 0, 0, 3, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([2, 0, 3, 2, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(2)].hand().resources(),
            ResourceCounts::new([2, 0, 0, 4, 0])
        );

        assert_eq!(game.next_player(game.current_player()), Ok(()));
        assert_eq!(game.status(), GameStatus::AwaitingRoll);
        assert_eq!(game.current_player(), PlayerId::new(3));

        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(2, 1).unwrap()),
            Ok(())
        );

        assert_eq!(
            game.players[&PlayerId::new(3)].hand().resources(),
            ResourceCounts::new([1, 0, 0, 3, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([3, 0, 3, 2, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(2)].hand().resources(),
            ResourceCounts::new([2, 0, 0, 4, 1])
        );

        assert_eq!(game.next_player(game.current_player()), Ok(()));

        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(6, 6).unwrap()),
            Ok(())
        );

        assert_eq!(
            game.players[&PlayerId::new(3)].hand().resources(),
            ResourceCounts::new([1, 0, 0, 3, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([3, 0, 3, 2, 2])
        );
        assert_eq!(
            game.players[&PlayerId::new(2)].hand().resources(),
            ResourceCounts::new([2, 0, 0, 4, 1])
        );

        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(68)),
            Ok(())
        );
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(7)),
            Ok(())
        );
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(51)),
            Ok(())
        );

        assert_eq!(game.players[&PlayerId::new(1)].score(), 5);

        assert_eq!(
            game.players[&PlayerId::new(3)].hand().resources(),
            ResourceCounts::new([1, 0, 0, 3, 1])
        );
        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([0, 0, 0, 0, 0])
        );
        assert_eq!(
            game.players[&PlayerId::new(2)].hand().resources(),
            ResourceCounts::new([2, 0, 0, 4, 1])
        );

        assert_eq!(game.next_player(game.current_player()), Ok(()));
        assert_eq!(game.status(), GameStatus::AwaitingRoll);
        assert_eq!(game.current_player(), PlayerId::new(2));

        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(2, 5).unwrap()),
            Ok(())
        );
        assert_eq!(game.status(), GameStatus::AwaitingNewRobberLocation);
        assert_eq!(
            game.move_robber(game.current_player(), TileId::new(0)),
            Ok(())
        );
        assert_eq!(game.status(), GameStatus::AwaitingSteal);
        assert_eq!(
            game.steal(
                game.current_player(),
                &Some(Steal::new(PlayerId::new(1), ResourceCounts::default()))
            ),
            Err(GameError::NoOneToSteal)
        );

        assert_eq!(
            game.players[&PlayerId::new(1)].hand().resources(),
            ResourceCounts::new([0, 0, 0, 0, 0])
        );

        assert_eq!(game.steal(game.current_player(), &None), Ok(()));
        assert_eq!(game.status(), GameStatus::PlayingActions);

        assert_eq!(game.next_player(game.current_player()), Ok(()));
        assert_eq!(game.status(), GameStatus::AwaitingRoll);
        assert_eq!(game.current_player(), PlayerId::new(3));

        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(4, 2).unwrap()),
            Ok(())
        );

        assert_eq!(game.next_player(game.current_player()), Ok(()));
        assert_eq!(game.status(), GameStatus::AwaitingRoll);
        assert_eq!(game.current_player(), PlayerId::new(1));

        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(4, 6).unwrap()),
            Ok(())
        );

        assert_eq!(game.next_player(game.current_player()), Ok(()));
        assert_eq!(game.status(), GameStatus::AwaitingRoll);
        assert_eq!(game.current_player(), PlayerId::new(2));

        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(2, 2).unwrap()),
            Ok(())
        );

        assert_eq!(game.next_player(game.current_player()), Ok(()));
        assert_eq!(game.status(), GameStatus::AwaitingRoll);
        assert_eq!(game.current_player(), PlayerId::new(3));

        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(4, 4).unwrap()),
            Ok(())
        );

        assert_eq!(game.next_player(game.current_player()), Ok(()));
        assert_eq!(game.status(), GameStatus::AwaitingRoll);
        assert_eq!(game.current_player(), PlayerId::new(1));

        assert_eq!(
            game.apply_roll(game.current_player(), Roll::new(4, 4).unwrap()),
            Ok(())
        );

        assert_eq!(
            game.upgrade_settlement_to_city(game.current_player(), VertexId::new(33)),
            Ok(())
        );
        assert_eq!(
            game.status(),
            GameStatus::End {
                winner: PlayerId::new(1)
            }
        );
    }

    #[allow(unused)]
    fn print_builds(game: &Game) {
        println!("Road");
        println!(
            "{:?}",
            game.board()
                .unwrap()
                .roads()
                .iter()
                .enumerate()
                .filter(|(_, o)| o.is_some())
                .collect::<Vec<(usize, &Option<PlayerId>)>>()
        );
        println!("Buildings");
        println!(
            "{:?}",
            game.board()
                .unwrap()
                .buildings()
                .iter()
                .enumerate()
                .filter(|(_, o)| o.is_some())
                .collect::<Vec<(usize, &Option<Building>)>>()
        );
    }
}
