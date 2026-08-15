use serde::{Deserialize, Serialize};
use crate::player::PlayerId;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]

pub struct Building { kind: BuildingKind, owner: PlayerId }

impl Building {
    pub fn new(kind: BuildingKind, owner: PlayerId) -> Self {
        Self { kind, owner }
    }
    
    pub fn kind(&self) -> BuildingKind { self.kind }

    pub fn owner(&self) -> PlayerId { self.owner }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildingKind { Settlement, City }

impl BuildingKind {
    pub(crate) fn amount(&self) -> u8 {
        match self { BuildingKind::Settlement => 1, BuildingKind::City => 2 }
    }

    pub fn points(&self) -> u8 {
        match self { BuildingKind::Settlement => 1, BuildingKind::City => 2}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::PlayerId;

    #[test]
    fn test_building_kind_amount() {
        assert_eq!(BuildingKind::Settlement.amount(), 1);
        assert_eq!(BuildingKind::City.amount(), 2);
    }

    #[test]
    fn test_building_new() {
        let b = Building::new(BuildingKind::Settlement, PlayerId::new(0));
        assert_eq!(b.kind, BuildingKind::Settlement);
        assert_eq!(b.owner, PlayerId::new(0));
    }
    
    
}