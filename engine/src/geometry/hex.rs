#[cfg(test)]
use std::fmt;
use serde::{Deserialize, Serialize};
use crate::geometry::{HexDirection, DIRS};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(test)]
pub struct InvalidHexCorner(pub usize);
#[cfg(test)]
impl fmt::Display for InvalidHexCorner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} n'est pas un corner valide (0-5)", self.0)
    }
}
#[cfg(test)]
impl std::error::Error for InvalidHexCorner {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HexCorner(usize);

impl HexCorner {
    pub const ALL: [HexCorner; 6] = [HexCorner(0), HexCorner(1), HexCorner(2),
        HexCorner(3), HexCorner(4), HexCorner(5)];

    #[cfg(test)]
    pub(crate) fn new(corner : usize) -> Result<HexCorner, InvalidHexCorner> {
        if matches!(corner, 0..=5) {
            Ok(HexCorner(corner))
        }else{
            Err(InvalidHexCorner(corner))
        }
    }

    pub(crate) fn value(self) -> usize {
        self.0
    }
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Hex {
    q: i8,
    r: i8,
}

impl Hex {
    pub(crate) fn new(q: i8, r: i8) -> Hex {
        Hex{q, r}
    }

    pub(crate) fn q(self) -> i8 {
        self.q
    }
    pub(crate) fn r(self) -> i8 {
        self.r
    }

    pub(crate) fn neighbor(self, dir : HexDirection) -> Hex{
        Hex::new(self.q + DIRS[dir.0].0, self.r + DIRS[dir.0].1)
    }

    pub(crate) fn corner_hexes(self, corner : HexCorner) -> [Hex; 3] {
        let mut hexes = [self, self.neighbor(HexDirection::new(corner.0).unwrap()), self.neighbor(HexDirection::new((corner.0 + 1) % 6).unwrap())];
        hexes.sort();
        hexes
    }
    
    pub(crate) fn edge_hexes(self, dir : HexDirection) -> [Hex; 2] {
        let mut hexes = [self, self.neighbor(dir)];
        hexes.sort();
        hexes
    }


}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hexcorner_new() {
        assert!(HexCorner::new(0).is_ok());
        assert!(HexCorner::new(5).is_ok());
        assert!(HexCorner::new(6).is_err());
    }

    #[test]
    fn test_hex_neighbor() {
        let hex = Hex{q: 0, r: 0};
        assert_eq!(hex.neighbor(HexDirection(0)), Hex{q: 1, r: 0});
        assert_eq!(hex.neighbor(HexDirection(1)), Hex{q: 1, r: -1});
        assert_eq!(hex.neighbor(HexDirection(2)), Hex{q: 0, r: -1});
        assert_eq!(hex.neighbor(HexDirection(3)), Hex{q: -1, r: 0});
        assert_eq!(hex.neighbor(HexDirection(4)), Hex{q: -1, r: 1});
        assert_eq!(hex.neighbor(HexDirection(5)), Hex{q: 0, r: 1});
    }

    #[test]
    fn test_hex_corner_hexes() {
        let hex1 = Hex{q: 0, r: 0};
        let hex2 = hex1.neighbor(HexDirection(0));
        assert_eq!(hex1.corner_hexes(HexCorner::new(0).unwrap()), hex2.corner_hexes(HexCorner::new(2).unwrap()))
    }

}