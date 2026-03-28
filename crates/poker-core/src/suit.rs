use std::{convert::TryFrom, fmt};

use crate::PokerError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    pub const ALL: [Suit; 4] = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];

    pub fn from_symbol(symbol: char) -> Result<Self, PokerError> {
        let normalized = symbol.to_ascii_lowercase();
        match normalized {
            'c' => Ok(Suit::Clubs),
            'd' => Ok(Suit::Diamonds),
            'h' => Ok(Suit::Hearts),
            's' => Ok(Suit::Spades),
            other => Err(PokerError::InvalidSuitChar(other)),
        }
    }

    pub fn symbol(self) -> char {
        match self {
            Suit::Clubs => 'c',
            Suit::Diamonds => 'd',
            Suit::Hearts => 'h',
            Suit::Spades => 's',
        }
    }
}

impl TryFrom<char> for Suit {
    type Error = PokerError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Self::from_symbol(value)
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}
