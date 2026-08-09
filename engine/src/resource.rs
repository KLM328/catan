mod cost;
mod hand;
mod counts;
mod steal;

use serde::{Deserialize, Serialize};
pub use cost::Cost;
pub use hand::{Hand, ResourceError};
pub use counts::ResourceCounts;
pub use steal::Steal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Resource {
    Wood,
    Stone,
    Brick,
    Wheat,
    Wool
}

impl Resource {

    pub const ALL: [Resource; 5] = [Resource::Wood, Resource::Stone, Resource::Brick, Resource::Wheat, Resource::Wool];
    pub(crate) fn index(self) -> usize {
        match self {
            Resource::Wood => 0,
            Resource::Stone => 1,
            Resource::Brick => 2,
            Resource::Wheat => 3,
            Resource::Wool => 4
        }
    }

    // pub(crate) fn from_index(index: usize) -> Option<Resource> {
    //     match index {
    //         0 => Some(Resource::Wood),
    //         1 => Some(Resource::Stone),
    //         2 => Some(Resource::Brick),
    //         3 => Some(Resource::Wheat),
    //         4 => Some(Resource::Wool),
    //         _ => None
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_roundtrip() {
        for r in Resource::ALL {
            let json = serde_json::to_string(&r).unwrap();
            let back: Resource = serde_json::from_str(&json).unwrap();
            assert_eq!(r, back);
            println!("{json}");
        }
    }
}