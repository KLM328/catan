use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use crate::{InvalidAction, InvalidBoard, PlayerId, ResourceError};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum GameError {
    BoardInitialization(InvalidBoard),
    Placement(InvalidAction),
    Resource(ResourceError),
    NotEnoughResources,
    NotYourTurn,
    GameOver,
    GameIsStarting,
    GameIsNotPlaying,
    WrongRollCount,
    TiedRolls,
    NotEnoughPlayers,
    TooManyPlayers,
    PlayerNotFound(PlayerId),
    TurnDrivenByPlacement,
    InvalidGameStatus,
    PlayerDontNeedToDiscard,
    InvalidDiscardCount,
    UnauthorizedVictim,
    MustStealSomeone,
    NoOneToSteal,
    GameIsFull,
    ColorNotAvailable
}

impl From<InvalidAction> for GameError {
    fn from(e: InvalidAction) -> Self {
        Self::Placement(e)
    }
}

impl From<ResourceError> for GameError {
    fn from(e: ResourceError) -> Self {
        Self::Resource(e)
    }
}

impl From<InvalidBoard> for GameError {
    fn from(e: InvalidBoard) -> Self {
        Self::BoardInitialization(e)
    }
}

impl Display for GameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            // Délégation : l'erreur imbriquée sait déjà se décrire
            GameError::BoardInitialization(e) => write!(f, "{e}"),
            GameError::Placement(e) => write!(f, "{e}"),
            GameError::Resource(e) => write!(f, "{e}"),

            GameError::NotEnoughResources =>
                write!(f, "Ressources insuffisantes"),
            GameError::NotYourTurn =>
                write!(f, "Ce n'est pas votre tour"),
            GameError::GameOver =>
                write!(f, "La partie est terminée"),
            GameError::GameIsStarting =>
                write!(f, "La partie n'a pas encore commencé"),
            GameError::GameIsNotPlaying =>
                write!(f, "Cette action n'est possible qu'en cours de partie"),
            GameError::WrongRollCount =>
                write!(f, "Il faut un jet de dés par joueur"),
            GameError::TiedRolls =>
                write!(f, "Égalité au plus haut jet : il faut relancer"),
            GameError::NotEnoughPlayers =>
                write!(f, "Il faut au moins deux joueurs"),
            GameError::TooManyPlayers =>
                write!(f, "Six joueurs au maximum"),
            GameError::PlayerNotFound(id) =>
                write!(f, "Joueur {} introuvable", id.value()),
            GameError::TurnDrivenByPlacement =>
                write!(f, "Pendant la mise en place, le tour passe automatiquement"),
            GameError::InvalidGameStatus =>
                write!(f, "Cette action n'est pas possible maintenant"),
            GameError::PlayerDontNeedToDiscard =>
                write!(f, "Ce joueur n'a pas à défausser"),
            GameError::InvalidDiscardCount =>
                write!(f, "Le nombre de cartes défaussées ne correspond pas"),
            GameError::UnauthorizedVictim =>
                write!(f, "Ce joueur n'est pas adjacent au voleur"),
            GameError::MustStealSomeone =>
                write!(f, "Vous devez voler un joueur adjacent au voleur"),
            GameError::NoOneToSteal =>
                write!(f, "Personne à voler sur cette tuile"),
            GameError::GameIsFull =>
                write!(f, "La partie est pleine"),
            GameError::ColorNotAvailable =>
                write!(f, "Cette couleur n'est pas disponible")
        }
    }
}