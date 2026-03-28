use std::fmt;

use crate::{Card, HoleCards, PokerError, Street};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Board {
    cards: Vec<Card>,
}

impl Board {
    pub fn from_cards(cards: impl IntoIterator<Item = Card>) -> Result<Self, PokerError> {
        let cards: Vec<Card> = cards.into_iter().collect();
        if cards.len() > 5 {
            return Err(PokerError::InvalidBoardLength {
                expected: 5,
                actual: cards.len(),
                street: Street::River,
            });
        }
        if has_duplicates(&cards) {
            return Err(PokerError::DuplicateBoardCards);
        }
        Ok(Self { cards })
    }

    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn street(&self) -> Option<Street> {
        match self.cards.len() {
            0 => Some(Street::Preflop),
            3 => Some(Street::Flop),
            4 => Some(Street::Turn),
            5 => Some(Street::River),
            _ => None,
        }
    }

    pub fn validate_for_street(&self, street: Street) -> Result<(), PokerError> {
        let expected = street.board_cards();
        let actual = self.cards.len();
        if actual != expected {
            return Err(PokerError::InvalidBoardLength { expected, actual, street });
        }
        if has_duplicates(&self.cards) {
            return Err(PokerError::DuplicateBoardCards);
        }
        Ok(())
    }

    pub fn validate_against_hole(&self, hole: HoleCards) -> Result<(), PokerError> {
        for board_card in &self.cards {
            if hole.contains(*board_card) {
                return Err(PokerError::BoardContainsHoleCard);
            }
        }
        Ok(())
    }
}

fn has_duplicates(cards: &[Card]) -> bool {
    for i in 0..cards.len() {
        for j in (i + 1)..cards.len() {
            if cards[i] == cards[j] {
                return true;
            }
        }
    }
    false
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, card) in self.cards.iter().enumerate() {
            if index > 0 {
                write!(f, " ")?;
            }
            write!(f, "{card}")?;
        }
        Ok(())
    }
}
