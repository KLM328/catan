use std::fmt;
use crate::{Cost, Resource};
use crate::resource::counts::ResourceCounts;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceError {
    NotEnoughResources,
    IsEmpty,
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResourceError::NotEnoughResources => write!(f, "Not enough resources"),
            ResourceError::IsEmpty => write!(f, "Resource is empty"),
        }
    }
}
impl std::error::Error for ResourceError {}

#[derive(Default, Clone)]
pub struct Hand(ResourceCounts);

impl Hand {
    pub(crate) fn pay(&mut self, cost: &Cost) -> Result<(), ResourceError> {
        if !self.0.try_subtract(&cost.resources()) {
            return Err(ResourceError::NotEnoughResources);
        }
        Ok(())
    }

    pub fn can_pay(&self, cost: &Cost) -> Result<(), ResourceError> {
        if !self.0.covers(&cost.resources()) {
            return Err(ResourceError::NotEnoughResources);
        }
        Ok(())
    }

    pub fn resources(&self) -> ResourceCounts {
        self.0
    }

    pub fn count(&self) -> u8 {
        self.0.count()
    }

    pub fn amount(&self, resource: Resource) -> u8 {
        self.0.amount(resource)
    }

    pub(crate) fn add(&mut self, resources : ResourceCounts){
        self.0.add(&resources);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.count() == 0
    }

    pub(crate) fn get_resource(&self, index: u8) -> Result<ResourceCounts, ResourceError> {
        if self.is_empty() {
            Err(ResourceError::IsEmpty)
        } else {
            Ok(self.0.get_resource(index))
        }

    }

    pub fn random_pick(&self) -> u8 {
        rand::random::<u8>() % self.count()
    }


}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hand_pay_with_not_enough_resources() -> () {
        let mut hand = Hand::default();
        assert_eq!(hand.pay(&Cost::ROAD), Err(ResourceError::NotEnoughResources));
        assert_eq!(hand.resources(), ResourceCounts::default());
    }

    #[test]
    fn test_hand_pay_with_enough_resources() -> () {
        let mut hand = Hand::default();
        hand.add(ResourceCounts::new([1, 0, 0, 0, 0]));
        hand.add(ResourceCounts::new([0, 0, 1, 0, 0]));
        assert_eq!(hand.pay(&Cost::ROAD), Ok(()));
        assert_eq!(hand.resources(), ResourceCounts::default());
    }

    #[test]
    fn test_hand_add() -> () {
        let mut hand = Hand::default();
        hand.add(ResourceCounts::new([0, 0, 1, 0, 0]));
        assert_eq!(hand.resources(), ResourceCounts::new([0, 0, 1, 0, 0]));
    }

    #[test]
    fn test_get_resource() {
        let hand = Hand(ResourceCounts::new([0,1,2,5,0]));
        assert_eq!(hand.get_resource(0), Ok(ResourceCounts::new([0,1,0,0,0])));
        assert_eq!(hand.get_resource(1), Ok(ResourceCounts::new([0,0,1,0,0])));
        assert_eq!(hand.get_resource(2), Ok(ResourceCounts::new([0,0,1,0,0])));
        assert_eq!(hand.get_resource(8), Ok(ResourceCounts::new([0,1,0,0,0])));

        let hand = Hand(ResourceCounts::default());
        assert_eq!(hand.get_resource(4), Err(ResourceError::IsEmpty));
    }
}
