use std::{fmt, str::FromStr};

use crate::{card::parse_compact_cards, cards_are_unique, Card, CardParseError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    cards: [Card; 5],
}

impl Board {
    pub const LEN: usize = 5;

    pub fn new(cards: [Card; 5]) -> Result<Self, BoardParseError> {
        let mut cards = cards;
        cards.sort_unstable();
        if !cards_are_unique(&cards) {
            return Err(BoardParseError::DuplicateCard);
        }
        Ok(Self { cards })
    }

    pub fn cards(&self) -> &[Card; 5] {
        &self.cards
    }

    pub fn contains(&self, card: Card) -> bool {
        self.cards.contains(&card)
    }

    pub fn is_disjoint_with(&self, hole_cards: &HoleCards) -> bool {
        self.cards.iter().all(|card| !hole_cards.contains(*card))
    }

    pub fn street(&self) -> Option<Street> {
        Some(Street::River)
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for card in &self.cards {
            write!(f, "{card}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardParseError {
    Card(CardParseError),
    WrongCount { expected: usize, found: usize },
    DuplicateCard,
}

impl fmt::Display for BoardParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoardParseError::Card(err) => write!(f, "{err}"),
            BoardParseError::WrongCount { expected, found } => {
                write!(f, "expected {expected} cards, found {found}")
            }
            BoardParseError::DuplicateCard => write!(f, "board contains duplicate cards"),
        }
    }
}

impl std::error::Error for BoardParseError {}

impl From<CardParseError> for BoardParseError {
    fn from(value: CardParseError) -> Self {
        BoardParseError::Card(value)
    }
}

impl FromStr for Board {
    type Err = BoardParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cards = parse_compact_cards(s)?;
        if cards.len() != Self::LEN {
            return Err(BoardParseError::WrongCount {
                expected: Self::LEN,
                found: cards.len(),
            });
        }
        Self::new(cards.try_into().expect("length checked"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HoleCards {
    cards: [Card; 2],
}

impl HoleCards {
    pub const LEN: usize = 2;

    pub fn new(cards: [Card; 2]) -> Result<Self, HoleCardsParseError> {
        let mut cards = cards;
        cards.sort_unstable();
        if !cards_are_unique(&cards) {
            return Err(HoleCardsParseError::DuplicateCard);
        }
        Ok(Self { cards })
    }

    pub fn cards(&self) -> &[Card; 2] {
        &self.cards
    }

    pub fn contains(&self, card: Card) -> bool {
        self.cards.contains(&card)
    }

    pub fn overlaps_board(&self, board: &Board) -> bool {
        !board.is_disjoint_with(self)
    }
}

impl fmt::Display for HoleCards {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for card in &self.cards {
            write!(f, "{card}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoleCardsParseError {
    Card(CardParseError),
    WrongCount { expected: usize, found: usize },
    DuplicateCard,
}

impl fmt::Display for HoleCardsParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HoleCardsParseError::Card(err) => write!(f, "{err}"),
            HoleCardsParseError::WrongCount { expected, found } => {
                write!(f, "expected {expected} cards, found {found}")
            }
            HoleCardsParseError::DuplicateCard => write!(f, "hole cards contain duplicate cards"),
        }
    }
}

impl std::error::Error for HoleCardsParseError {}

impl From<CardParseError> for HoleCardsParseError {
    fn from(value: CardParseError) -> Self {
        HoleCardsParseError::Card(value)
    }
}

impl FromStr for HoleCards {
    type Err = HoleCardsParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cards = parse_compact_cards(s)?;
        if cards.len() != Self::LEN {
            return Err(HoleCardsParseError::WrongCount {
                expected: Self::LEN,
                found: cards.len(),
            });
        }
        Self::new(cards.try_into().expect("length checked"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_and_hole_cards_parse_and_sort_canonically() {
        let board: Board = "Ks As Qd Jc Tc".parse().unwrap();
        assert_eq!(board.to_string(), "TcJcQdKsAs");

        let hole: HoleCards = "Ks As".parse().unwrap();
        assert_eq!(hole.to_string(), "AsKs");
        assert!(board.is_disjoint_with(&hole));
    }

    #[test]
    fn rejects_duplicate_cards() {
        assert!("AsAs".parse::<HoleCards>().is_err());
        assert!("AsAsKsQdJc".parse::<Board>().is_err());
    }
}
