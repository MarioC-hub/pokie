use std::fmt;

use crate::{Board, Card, PokerError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HoleCards {
    cards: [Card; 2],
}

impl HoleCards {
    pub fn new(card_a: Card, card_b: Card) -> Result<Self, PokerError> {
        if card_a == card_b {
            return Err(PokerError::DuplicateHoleCards);
        }
        Ok(Self { cards: [card_a, card_b] })
    }

    pub const fn cards(self) -> [Card; 2] {
        self.cards
    }

    pub fn contains(self, card: Card) -> bool {
        self.cards[0] == card || self.cards[1] == card
    }

    pub fn validate(self) -> Result<(), PokerError> {
        if self.cards[0] == self.cards[1] {
            Err(PokerError::DuplicateHoleCards)
        } else {
            Ok(())
        }
    }

    pub fn validate_against_board(self, board: &Board) -> Result<(), PokerError> {
        board.validate_against_hole(self)
    }
}

impl fmt::Display for HoleCards {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.cards[0], self.cards[1])
    }
}
