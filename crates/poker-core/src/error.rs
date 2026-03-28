use std::{error::Error, fmt};

use crate::Street;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PokerError {
    InvalidRankChar(char),
    InvalidSuitChar(char),
    InvalidCardFormat(String),
    DuplicateHoleCards,
    DuplicateBoardCards,
    InvalidBoardLength { expected: usize, actual: usize, street: Street },
    BoardContainsHoleCard,
}

impl fmt::Display for PokerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PokerError::InvalidRankChar(ch) => write!(f, "invalid rank character '{ch}'"),
            PokerError::InvalidSuitChar(ch) => write!(f, "invalid suit character '{ch}'"),
            PokerError::InvalidCardFormat(input) => write!(f, "invalid card format: {input}"),
            PokerError::DuplicateHoleCards => write!(f, "hole cards must be distinct"),
            PokerError::DuplicateBoardCards => write!(f, "board cards must be distinct"),
            PokerError::InvalidBoardLength { expected, actual, street } => write!(
                f,
                "board for {street} must contain {expected} cards, found {actual}"
            ),
            PokerError::BoardContainsHoleCard => write!(f, "board shares a card with the hole cards"),
        }
    }
}

impl Error for PokerError {}
