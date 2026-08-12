use serde::{Deserialize, Serialize};
use catan::GameError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServerError {
    NotTheHost,
    InvalidMessageFormat,
    InvalidMessageType,
    Rules(GameError),
    GamePaused,
    InvalidToken
}

impl From<GameError> for ServerError {
    fn from(value: GameError) -> Self {
        ServerError::Rules(value)
    }
}