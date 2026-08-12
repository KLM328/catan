use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Eq, Copy, Clone)]
pub struct Token {
}

impl Token {
    pub fn new() -> Self {
        Self {}
    }
}