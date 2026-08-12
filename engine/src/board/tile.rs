mod token;

use std::fmt;
use serde::{Deserialize, Serialize};
pub use token::NumberToken;
use crate::resource::Resource;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainTokenMismatch {
    DesertHasNoToken,
    MissingToken(Terrain),
}

impl fmt::Display for TerrainTokenMismatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DesertHasNoToken => write!(f, "un désert ne peut pas porter de jeton"),
            Self::MissingToken(t)  => write!(f, "{t:?} doit porter un jeton"),
        }
    }
}
impl std::error::Error for TerrainTokenMismatch {}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Terrain { Desert, Forest, Mountain, Hills, Pasture, Fields }

impl Terrain {

    pub const ALL: [Terrain; 6] = [Terrain::Desert, Terrain::Forest, Terrain::Mountain, Terrain::Hills, Terrain::Pasture, Terrain::Fields];
    pub(crate) fn into_tile(self, token: Option<NumberToken>) -> Result<Tile, TerrainTokenMismatch> {
        match (self, token) {
            (Terrain::Desert, None) => Ok(Tile::Desert),
            (Terrain::Desert, Some(_)) => Err(TerrainTokenMismatch::DesertHasNoToken),
            (_, None) => Err(TerrainTokenMismatch::MissingToken(self)),
            (Terrain::Forest, Some(n)) => Ok(Tile::Forest(n)),
            (Terrain::Mountain, Some(n)) => Ok(Tile::Mountain(n)),
            (Terrain::Hills, Some(n)) => Ok(Tile::Hills(n)),
            (Terrain::Pasture, Some(n)) => Ok(Tile::Pasture(n)),
            (Terrain::Fields, Some(n)) => Ok(Tile::Fields(n)),
        }
    }
    pub(crate) fn resource(self) -> Option<Resource> {
        match self {
            Terrain::Desert => None,
            Terrain::Forest => Some(Resource::Wood),
            Terrain::Mountain => Some(Resource::Stone),
            Terrain::Hills => Some(Resource::Brick),
            Terrain::Pasture => Some(Resource::Wool),
            Terrain::Fields => Some(Resource::Wheat),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tile {
    Desert,
    Forest(NumberToken),
    Mountain(NumberToken),
    Hills(NumberToken),
    Pasture(NumberToken),
    Fields(NumberToken),
}


impl Tile {
    pub fn terrain(self) -> Terrain{
        match self {
            Tile::Desert => Terrain::Desert,
            Tile::Forest(_) => Terrain::Forest,
            Tile::Mountain(_) => Terrain::Mountain,
            Tile::Hills(_) => Terrain::Hills,
            Tile::Pasture(_) => Terrain::Pasture,
            Tile::Fields(_) => Terrain::Fields,
        }
    }
    pub(crate) fn resource(self) -> Option<Resource> {
        self.terrain().resource()
    }

    pub fn number(self) -> Option<NumberToken> {
        match self {
            Tile::Forest(n) => Some(n),
            Tile::Mountain(n) => Some(n),
            Tile::Hills(n) => Some(n),
            Tile::Pasture(n) => Some(n),
            Tile::Fields(n) => Some(n),
            Tile::Desert => None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_and_terrain_never_disagree() {
        let t = NumberToken::new(8).unwrap();
        for terrain in Terrain::ALL {
            let token = if terrain == Terrain::Desert { None } else { Some(t) };
            let tile = terrain.into_tile(token).unwrap();
            assert_eq!(tile.resource(), terrain.resource(), "désaccord pour {terrain:?}");
            assert_eq!(tile.terrain(), terrain);
            assert_eq!(tile.number(), token);
        }
    }

    #[test]
    fn test_number_token() {
        assert_eq!(Tile::Fields(NumberToken::new(8).unwrap()).number(), NumberToken::new(8).ok());
        assert_eq!(Tile::Desert.number(), None);

    }
}