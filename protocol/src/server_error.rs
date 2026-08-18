use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use catan::GameError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServerError {
    NotTheHost,
    InvalidMessageFormat,
    InvalidMessageType,
    Rules(GameError),
    GamePaused,
    InvalidToken,
    PlayerIsAlreadyConnected,
    ServerOffline,
    ConnexionRefused,
    ConnexionTimedOut,
    GameNotFound,
}

impl From<GameError> for ServerError {
    fn from(value: GameError) -> Self {
        ServerError::Rules(value)
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::NotTheHost => write!(f, "Vous n'ête pas l'host de la partie"),
            ServerError::InvalidMessageFormat => write!(f, "Le message n'est pas au bon format"),
            ServerError::InvalidMessageType => write!(f, "Le message n'est pas du bon type"),
            ServerError::Rules(e) => e.fmt(f),
            ServerError::GamePaused => write!(f, "La partie est en pause"),
            ServerError::InvalidToken => write!(f, "Le jeton de connexion est invalide"),
            ServerError::PlayerIsAlreadyConnected => write!(f, "vous êtes déjà connecté à cette partie"),
            ServerError::ServerOffline => write!(f, "Le serveur est hors ligne"),
            ServerError::ConnexionRefused => write!(f, "Connexion au serveur refusé"),
            ServerError::ConnexionTimedOut => write!(f, "Connexion au serveur impossible, vérifiez votre connexion Internet"),
            ServerError::GameNotFound => write!(f, "Partie introuvable"),
        }
    }
}