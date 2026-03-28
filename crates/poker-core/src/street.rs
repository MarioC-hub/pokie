use std::{convert::TryFrom, fmt, str::FromStr};

use crate::PokerError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Street {
    Preflop,
    Flop,
    Turn,
    River,
}

impl Street {
    pub const ALL: [Street; 4] = [Street::Preflop, Street::Flop, Street::Turn, Street::River];

    pub const fn board_cards(self) -> usize {
        match self {
            Street::Preflop => 0,
            Street::Flop => 3,
            Street::Turn => 4,
            Street::River => 5,
        }
    }

    pub const fn next(self) -> Option<Self> {
        match self {
            Street::Preflop => Some(Street::Flop),
            Street::Flop => Some(Street::Turn),
            Street::Turn => Some(Street::River),
            Street::River => None,
        }
    }
}

impl FromStr for Street {
    type Err = PokerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "preflop" => Ok(Street::Preflop),
            "flop" => Ok(Street::Flop),
            "turn" => Ok(Street::Turn),
            "river" => Ok(Street::River),
            _ => Err(PokerError::InvalidCardFormat(s.trim().to_string())),
        }
    }
}

impl TryFrom<&str> for Street {
    type Error = PokerError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl fmt::Display for Street {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Street::Preflop => "preflop",
            Street::Flop => "flop",
            Street::Turn => "turn",
            Street::River => "river",
        };
        write!(f, "{name}")
    }
}
