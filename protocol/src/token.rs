use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Eq, Copy, Clone)]
pub struct Token {
    value : Uuid
}

impl Token {
    pub fn new() -> Self {
        Self {
            value : Uuid::new_v4()
        }
    }
}

impl Default for Token {
    fn default() -> Self {
        Self::new()
    }
}