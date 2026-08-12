use crate::{PlayerId, ResourceCounts};

pub struct Steal {victim : PlayerId, resource : ResourceCounts}

impl Steal {
    pub fn new(victim : PlayerId, resource : ResourceCounts) -> Steal {
        Steal {victim, resource}
    }

    pub fn victim(&self) -> PlayerId {
        self.victim
    }

    pub fn resource(&self) -> ResourceCounts {
        self.resource
    }
}