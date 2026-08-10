use serde::{Deserialize, Serialize};
use crate::player::PlayerId;
use crate::ResourceCounts;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gain {
    pub player: PlayerId,
    pub resources: ResourceCounts
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Production {
    gains: Vec<Gain>,
}

impl Production {

    pub fn new(entries: &[(PlayerId, [u8; 5])]) -> Self {
        let mut production = Self::default();
        for &(player, counts) in entries {
            production.add_gain(Gain { player, resources: ResourceCounts::new(counts)});
        }
        production.sort();
        production
    }

    pub(crate) fn add_gain(&mut self, gain: Gain) {
        if let Some(existing) = self.gains.iter_mut()
            .find(|g| g.player == gain.player)
        {
            existing.resources.add(&gain.resources);
        } else {
            self.gains.push(gain);
        }
    }

    pub(crate) fn gains(&self) -> &[Gain] { &self.gains }

    pub fn sort(&mut self) {
        self.gains.sort_by_key(|g| g.player.value());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_add_gain() {
        let mut production = Production::default();
        production.add_gain(Gain { player: PlayerId::new(0), resources: ResourceCounts::new([2,0,0,0,0]) });
        assert_eq!(production.gains.len(), 1);
    }

    #[test]
    fn test_production_is_empty() {
        let production = Production::default();
        assert!(production.gains.is_empty());    }

    #[test]
    fn test_production_gains() {
        let mut production = Production::default();
        production.add_gain(Gain { player: PlayerId::new(0), resources: ResourceCounts::new([2,0,0,0,0]) });
        assert_eq!(production.gains()[0], Gain { player: PlayerId::new(0), resources: ResourceCounts::new([2,0,0,0,0]) });
    }
}