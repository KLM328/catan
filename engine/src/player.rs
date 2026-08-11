use serde::{Deserialize, Serialize};
use crate::{Hand, ResourceError, ResourceCounts};

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum PlayerColor {
    Red,
    Blue,
    White,
    Orange,
    Green,
    Brown
}

impl PlayerColor {
    pub fn color_name(self) -> &'static str{
        match self {
            PlayerColor::Red => {"Rouge"}
            PlayerColor::Blue => {"Bleu"}
            PlayerColor::White => {"Blanc"}
            PlayerColor::Orange => {"Orange"}
            PlayerColor::Green => {"Vert"}
            PlayerColor::Brown => {"Marron"}
        }
    }
    
    pub const ALL : [Self; 6] = [Self::Red, Self::Blue, Self::White, Self::Orange, Self::Green, Self::Brown];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(usize);
impl PlayerId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn value(self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    color: PlayerColor,
    hand: Hand,
    score: u8,
}

impl Player {
    pub fn new(color: PlayerColor) -> Self {
        Self {
            color,
            hand: Hand::default(),
            score: 0,
        }
    }

    pub fn color(&self) -> PlayerColor {
        self.color
    }

    pub fn hand(&self) -> &Hand {
        &self.hand
    }

    pub(crate) fn receive(&mut self, resources : ResourceCounts) {

        self.hand.add(resources);
    }

    pub(crate) fn pay(&mut self, cost : &crate::Cost) -> Result<(), ResourceError> {
        self.hand.pay(cost)
    }

    pub fn can_pay(&self, cost : &crate::Cost) -> Result<(), ResourceError> {
        self.hand.can_pay(cost)
    }

    pub fn score(&self) -> u8 {
        self.score
    }

    pub(crate) fn add_score(&mut self, amount : u8){
        self.score += amount;
    }

    pub(crate) fn remove_score(&mut self, amount : u8){
        self.score -= amount;
    }
}
