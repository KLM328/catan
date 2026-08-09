use std::fmt;
use rand::RngExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub struct InvalidDices(pub u8, pub u8);

impl fmt::Display for InvalidDices {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} et {} n'est pas un lancé de dé valide", self.0, self.1)
    }
}
impl std::error::Error for InvalidDices {}



#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(try_from = "(u8, u8)", into = "(u8, u8)")]
pub struct Roll {
    dice1: u8,
    dice2: u8,
}

impl TryFrom<(u8, u8)> for Roll {
    type Error = InvalidDices;

    fn try_from(value: (u8, u8)) -> Result<Self, Self::Error> {
        Roll::new(value.0, value.1)
    }
}

impl From<Roll> for (u8, u8) {
    fn from(value: Roll) -> Self {
        (value.dice1, value.dice2)
    }
}

impl Roll {
    pub(crate) fn value(self) -> u8 {
        self.dice1 + self.dice2
    }

    pub(crate) fn new(dice1: u8, dice2: u8) -> Result<Self, InvalidDices> {
        if matches!(dice1, 1..=6) && matches!(dice2, 1..=6) {
            Ok(Self { dice1, dice2 })
        } else {
            Err(InvalidDices(dice1, dice2))
        }
    }

    pub fn random() -> Self {
        let mut rng = rand::rng();
        Self::new(rng.random_range(1..=6), rng.random_range(1..=6)).unwrap()
    }

    pub fn dice1(&self) -> u8 {
        self.dice1
    }
    pub fn dice2(&self) -> u8 {
        self.dice2
    }
}

impl PartialEq for Roll {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}
impl Eq for Roll {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roll() {
        assert!(matches!(Roll::random().value(), 2..=12));
    }

    #[test]
    fn accept_valid_dice_rolls() {
        assert!(Roll::new(1, 1).is_ok());
        assert!(Roll::new(6, 6).is_ok());
    }

    #[test]
    fn reject_invalid_dice_rolls() {
        assert!(Roll::new(0, 0).is_err());
        assert!(Roll::new(7, 7).is_err());
        assert!(Roll::new(13, 13).is_err());
    }

    #[test]
    fn test_roll_serialization() {
        for i in 1..=6 {
            for j in 1..=6 {
                let roll = Roll::new(i, j).unwrap();
                let json = serde_json::to_string(&roll).unwrap();
                println!("{}", json);
                let back: Roll = serde_json::from_str(&json).unwrap();
                assert_eq!(roll, back);
            }
        }

    }

    #[test]
    fn test_roll_deserialization_reject_invalid_dice_rolls() {
        assert!(serde_json::from_str::<Roll>("(1, 0)").is_err());
        assert!(serde_json::from_str::<Roll>("(0, 1)").is_err());

        assert!(serde_json::from_str::<Roll>("(3, 7)").is_err());
        assert!(serde_json::from_str::<Roll>("(7, 3)").is_err());
    }

}