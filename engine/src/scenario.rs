use crate::{Board, InvalidBoard, NumberToken, Terrain, TileId, Topology};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Scenario {
    topology: Topology,
    terrain_bag: Vec<Terrain>,
    token_sequence: Vec<NumberToken>,
    max_points : u8,
}

impl Scenario {
    pub fn terrains(&self) -> &[Terrain] {
        &self.terrain_bag
    }
    pub fn standard() -> Scenario {
        let hexes = Topology::spiral(2);

        Scenario {
            topology: Topology::from_hexes(&hexes),
            terrain_bag: vec![
                Terrain::Forest,
                Terrain::Fields,
                Terrain::Forest,
                Terrain::Pasture,
                Terrain::Pasture,
                Terrain::Forest,
                Terrain::Fields,
                Terrain::Pasture,
                Terrain::Hills,
                Terrain::Pasture,
                Terrain::Hills,
                Terrain::Fields,
                Terrain::Hills,
                Terrain::Mountain,
                Terrain::Fields,
                Terrain::Mountain,
                Terrain::Forest,
                Terrain::Mountain,
                Terrain::Desert,
            ],
            token_sequence: vec![
                5u8, 2u8, 6u8, 3u8, 8u8, 10u8, 9u8, 12u8, 11u8, 4u8, 8u8, 10u8, 9u8, 4u8, 5u8, 6u8,
                3u8, 11u8,
            ]
            .iter()
            .map(|&i| NumberToken::new(i).unwrap())
            .collect(),
            max_points : 10,
        }
    }



    pub(crate) fn layout(&self, shuffled: &[Terrain]) -> Result<Board, InvalidBoard> {
        if shuffled.len() != self.terrain_bag.len() {
            return Err(InvalidBoard::WrongTileCount {
                expected: self.terrain_bag.len(),
                got: shuffled.len(),
            });
        }
        let occ_shuffled = Scenario::occ_terrains(shuffled);
        let occ_bag = Scenario::occ_terrains(&self.terrain_bag);

        if occ_shuffled != occ_bag {
            return Err(InvalidBoard::WrongDistribution);
        }

        let mut tiles = Vec::new();
        let mut robber: Option<TileId> = None;
        let mut tokens = self.token_sequence.iter().copied();

        for (i, &terrain) in shuffled.iter().enumerate() {
                let token = if terrain == Terrain::Desert {
                if robber.is_none() {
                    robber = Some(TileId::new(i));
                }
                None
            } else {
                tokens.next()
            };

            tiles.push(terrain.into_tile(token)?);
        }

        Board::new(self.topology.clone(), tiles, robber.ok_or(InvalidBoard::NoDesert)?)
    }

    fn occ_terrains(terrains: &[Terrain]) -> HashMap<Terrain, usize> {
        let mut occ_terrains = HashMap::new();
        for terrain in terrains {
            *occ_terrains.entry(*terrain).or_insert(0) += 1;
        }
        occ_terrains
    }

    #[cfg(test)]
    pub(crate) fn fast_standard() -> Scenario {
        let hexes = Topology::spiral(2);

        Scenario {
            topology: Topology::from_hexes(&hexes),
            terrain_bag: vec![
                Terrain::Forest,
                Terrain::Fields,
                Terrain::Forest,
                Terrain::Pasture,
                Terrain::Pasture,
                Terrain::Forest,
                Terrain::Fields,
                Terrain::Pasture,
                Terrain::Hills,
                Terrain::Pasture,
                Terrain::Hills,
                Terrain::Fields,
                Terrain::Hills,
                Terrain::Mountain,
                Terrain::Fields,
                Terrain::Mountain,
                Terrain::Forest,
                Terrain::Mountain,
                Terrain::Desert,
            ],
            token_sequence: vec![
                5u8, 2u8, 6u8, 3u8, 8u8, 10u8, 9u8, 12u8, 11u8, 4u8, 8u8, 10u8, 9u8, 4u8, 5u8, 6u8,
                3u8, 11u8,
            ]
                .iter()
                .map(|&i| NumberToken::new(i).unwrap())
                .collect(),
            max_points : 6,
        }
    }

    pub(crate) fn max_points(&self) -> u8 {
        self.max_points
    }

    #[cfg(test)]
    pub(crate) fn test_scenario() -> Scenario {
        Scenario {topology : Topology::test_topology(), terrain_bag : vec![Terrain::Forest, Terrain::Hills, Terrain::Desert], token_sequence : vec![NumberToken::new(6).unwrap(), NumberToken::new(8).unwrap()], max_points : 10}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_scenario_standard() {
        let scenario = Scenario::standard();
        assert_eq!(scenario.terrain_bag.len(), 19);
        assert_eq!(scenario.token_sequence.len(), 18);

        let mut occ_bag = HashMap::new();
        for terrain in scenario.terrain_bag {
            *occ_bag.entry(terrain).or_insert(0) += 1;
        }
        assert_eq!(occ_bag[&Terrain::Desert], 1);
        assert_eq!(occ_bag[&Terrain::Pasture], 4);
        assert_eq!(occ_bag[&Terrain::Fields], 4);
        assert_eq!(occ_bag[&Terrain::Hills], 3);
        assert_eq!(occ_bag[&Terrain::Mountain], 3);
        assert_eq!(occ_bag[&Terrain::Forest], 4);

        let mut occ_sequence = HashMap::new();
        for token in scenario.token_sequence {
            *occ_sequence.entry(token).or_insert(0) += 1;
        }

        for i in 3..=11 {
            if i == 7 {
                continue;
            }
            assert_eq!(occ_sequence[&NumberToken::new(i).unwrap()], 2);
        }
        assert_eq!(occ_sequence[&NumberToken::new(2).unwrap()], 1);
        assert_eq!(occ_sequence[&NumberToken::new(12).unwrap()], 1);
    }

    #[test]
    fn robber_and_tokens_follow_the_desert() {
        let scenario = Scenario::standard();

        for desert_pos in 0..scenario.terrain_bag.len() {
            let mut bag = scenario.terrain_bag.clone();
            let current = bag.iter().position(|t| *t == Terrain::Desert).unwrap();
            bag.swap(current, desert_pos);              // même répartition, désert déplacé

            let board = scenario.layout(&bag).unwrap();

            assert_eq!(board.robber(), TileId::new(desert_pos));

            let mut expected = scenario.token_sequence.iter().copied();
            for (i, tile) in board.tiles().iter().enumerate() {
                if i == desert_pos {
                    assert_eq!(tile.number(), None, "le désert ne doit pas avoir de jeton");
                } else {
                    assert_eq!(tile.number(), expected.next(), "décalage en position {i}");
                }
            }
            assert!(expected.next().is_none(), "des jetons n'ont pas été posés");
        }
    }

    #[test]
    fn rejects_wrong_tile_count() {
        let scenario = Scenario::standard();
        let short = &scenario.terrain_bag[..18];
        assert_eq!(scenario.layout(short),
                   Err(InvalidBoard::WrongTileCount { expected: 19, got: 18 }));
    }

    #[test]
    fn rejects_wrong_distribution() {
        let scenario = Scenario::standard();
        let mut bag = scenario.terrain_bag.clone();
        let i = bag.iter().position(|t| *t == Terrain::Pasture).unwrap();
        bag[i] = Terrain::Forest;
        assert_eq!(scenario.layout(&bag), Err(InvalidBoard::WrongDistribution));
    }

    #[test]
    fn same_arrangement_gives_same_board() {
        let scenario = Scenario::standard();
        let bag = scenario.terrain_bag.clone();
        assert_eq!(scenario.layout(&bag).unwrap(), scenario.layout(&bag).unwrap());
    }

    #[test]
    fn standard_never_puts_red_numbers_side_by_side() {
        let scenario = Scenario::standard();

        for desert_pos in 0..scenario.terrain_bag.len() {
            let mut bag = scenario.terrain_bag.clone();
            let current = bag.iter().position(|t| *t == Terrain::Desert).unwrap();
            bag.swap(current, desert_pos);
            let board = scenario.layout(&bag).unwrap();

            for (a, b) in board.topology().adjacent_tile_pairs() {
                let is_red = |t: TileId| matches!(
                board.tiles()[t.value()].number().map(|n| n.value()), Some(6) | Some(8));
                assert!(!(is_red(a) && is_red(b)),
                        "numéros rouges adjacents, désert en position {desert_pos}");
            }
        }
    }

}
