#![forbid(unsafe_code)]

//! Exact weighted combo range handling for Pokie.
//!
//! The public surface is intentionally small and deterministic:
//! - `WeightedEntry` stores a concrete `HoleCards`/weight pair
//! - `WeightedRange` canonicalizes, normalizes, and serializes exact combos
//! - compatibility aliases preserve the older `ComboRange`/`ExactCombo` names

use std::{fmt, str::FromStr};

pub use poker_core::{
    Board, Card, CardParseError, CardSetError, HoleCards, HoleCardsParseError, PokerError, Rank,
    Suit,
};

/// A single explicit hole-card combo and its weight.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeightedEntry {
    pub hand: HoleCards,
    pub hole_cards: HoleCards,
    pub weight: f64,
}

/// Back-compat alias for older call sites.
pub type WeightedCombo = WeightedEntry;
/// Back-compat alias for older call sites.
pub type ExactCombo = WeightedEntry;
/// Back-compat alias for older call sites.
pub type PrivateHand = HoleCards;

impl WeightedEntry {
    pub fn new(hole_cards: HoleCards, weight: f64) -> Result<Self, RangeError> {
        if !weight.is_finite() || weight <= 0.0 {
            return Err(RangeError::InvalidWeight { weight });
        }
        Ok(Self {
            hand: hole_cards,
            hole_cards,
            weight,
        })
    }

    pub fn hand(&self) -> HoleCards {
        self.hand
    }

    pub fn hole_cards(&self) -> HoleCards {
        self.hole_cards
    }

    pub fn weight(&self) -> f64 {
        self.weight
    }
}

impl fmt::Display for WeightedEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if approx_eq(self.weight, 1.0) {
            write!(f, "{}", self.hole_cards)
        } else {
            write!(f, "{}:{}", self.hole_cards, format_weight(self.weight))
        }
    }
}

impl FromStr for WeightedEntry {
    type Err = RangeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_entry(s)
    }
}

/// Canonical exact-combo range.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRange {
    entries: Vec<WeightedEntry>,
}

/// Back-compat alias for older call sites.
pub type ComboRange = WeightedRange;

impl WeightedRange {
    pub fn new(entries: Vec<WeightedEntry>) -> Result<Self, RangeError> {
        canonicalize_entries(entries)
    }

    pub fn from_pairs(entries: Vec<(HoleCards, f64)>) -> Result<Self, RangeError> {
        Self::new(
            entries
                .into_iter()
                .map(|(hole_cards, weight)| WeightedEntry {
                    hand: hole_cards,
                    hole_cards,
                    weight,
                })
                .collect(),
        )
    }

    /// Compatibility alias for older fixtures and tests.
    pub fn from_hands(entries: Vec<(HoleCards, f64)>) -> Result<Self, RangeError> {
        Self::from_pairs(entries)
    }

    pub fn parse(text: &str) -> Result<Self, RangeError> {
        text.parse()
    }

    /// Compatibility alias for older fixtures and tests.
    pub fn parse_exact(text: &str) -> Result<Self, RangeError> {
        Self::parse(text)
    }

    pub fn entries(&self) -> &[WeightedEntry] {
        &self.entries
    }

    /// Compatibility alias for older fixtures and tests.
    pub fn combos(&self) -> &[WeightedEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn total_weight(&self) -> f64 {
        self.entries.iter().map(|entry| entry.weight).sum()
    }

    pub fn normalized(&self) -> Result<Self, RangeError> {
        Self::new(self.entries.clone())
    }

    pub fn weight_of(&self, hole_cards: &HoleCards) -> Option<f64> {
        self.entries
            .iter()
            .find(|entry| &entry.hole_cards == hole_cards)
            .map(|entry| entry.weight)
    }

    pub fn contains(&self, hole_cards: &HoleCards) -> bool {
        self.weight_of(hole_cards).is_some()
    }

    pub fn first_board_overlap(&self, board: &Board) -> Option<Card> {
        for entry in &self.entries {
            for card in entry.hole_cards.cards() {
                if board.contains(card) {
                    return Some(card);
                }
            }
        }
        None
    }

    pub fn validate_against_board(&self, board: &Board) -> Result<(), RangeError> {
        if let Some(card) = self.first_board_overlap(board) {
            let hand = self
                .entries
                .iter()
                .find(|entry| entry.hole_cards.cards().contains(&card))
                .expect("first_board_overlap reported a card from an entry")
                .hole_cards;
            return Err(RangeError::BoardConflict {
                hand: hand.to_string(),
                card,
            });
        }
        Ok(())
    }

    pub fn compatible_pairings<'a>(
        &'a self,
        other: &'a Self,
        board: &Board,
    ) -> Result<Vec<WeightedPairing<'a>>, RangeError> {
        self.validate_against_board(board)?;
        other.validate_against_board(board)?;

        let mut pairings = Vec::new();
        for left in &self.entries {
            for right in &other.entries {
                if !left.hole_cards.overlaps(right.hole_cards) {
                    pairings.push(WeightedPairing {
                        left,
                        right,
                        combined_weight: left.weight * right.weight,
                    });
                }
            }
        }

        if pairings.is_empty() {
            return Err(RangeError::NoCompatiblePairings);
        }
        Ok(pairings)
    }

    pub fn remove_board_blocked_combos(&self, board: &Board) -> Result<Self, RangeError> {
        let filtered = self
            .entries
            .iter()
            .copied()
            .filter(|entry| !entry.hole_cards.overlaps_board(board))
            .collect::<Vec<_>>();
        Self::new(filtered)
    }

    pub fn remove_overlaps_with_range(&self, other: &Self) -> Result<Self, RangeError> {
        let filtered = self
            .entries
            .iter()
            .copied()
            .filter(|left| {
                other
                    .entries
                    .iter()
                    .all(|right| !left.hole_cards.overlaps(right.hole_cards))
            })
            .collect::<Vec<_>>();
        Self::new(filtered)
    }

    pub fn canonical_string(&self) -> String {
        self.entries
            .iter()
            .map(|entry| format!("{}:{}", entry.hole_cards, format_weight(entry.weight)))
            .collect::<Vec<_>>()
            .join(",")
    }
}

impl fmt::Display for WeightedRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.canonical_string())
    }
}

impl FromStr for WeightedRange {
    type Err = RangeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let entries = split_tokens(s)
            .into_iter()
            .map(parse_entry)
            .collect::<Result<Vec<_>, _>>()?;
        Self::new(entries)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RangeError {
    EmptyRange,
    InvalidComboSyntax { token: String },
    InvalidWeightSyntax { token: String },
    InvalidWeight { weight: f64 },
    BoardConflict { hand: String, card: Card },
    NoCompatiblePairings,
}

impl fmt::Display for RangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RangeError::EmptyRange => write!(f, "range is empty"),
            RangeError::InvalidComboSyntax { token } => {
                write!(f, "invalid exact-combo token {token:?}")
            }
            RangeError::InvalidWeightSyntax { token } => {
                write!(f, "invalid combo weight in token {token:?}")
            }
            RangeError::InvalidWeight { weight } => write!(f, "invalid combo weight {weight}"),
            RangeError::BoardConflict { hand, card } => {
                write!(f, "hand {hand} conflicts with board card {card}")
            }
            RangeError::NoCompatiblePairings => {
                write!(f, "ranges have no compatible private-hand pairings")
            }
        }
    }
}

impl std::error::Error for RangeError {}

#[derive(Debug, Clone, Copy)]
pub struct WeightedPairing<'a> {
    left: &'a WeightedEntry,
    right: &'a WeightedEntry,
    combined_weight: f64,
}

impl<'a> WeightedPairing<'a> {
    pub const fn left(self) -> WeightedEntry {
        *self.left
    }

    pub const fn right(self) -> WeightedEntry {
        *self.right
    }

    pub const fn combined_weight(self) -> f64 {
        self.combined_weight
    }
}

fn split_tokens(input: &str) -> impl Iterator<Item = &str> {
    input
        .split([',', ';', '|'])
        .map(str::trim)
        .filter(|token| !token.is_empty())
}

fn parse_entry(text: &str) -> Result<WeightedEntry, RangeError> {
    let token = text.trim();
    if token.is_empty() {
        return Err(RangeError::InvalidComboSyntax {
            token: text.to_string(),
        });
    }

    let (cards_text, weight_text) = if let Some((cards, weight)) = token.rsplit_once('@') {
        (cards.trim(), Some(weight.trim()))
    } else if let Some((cards, weight)) = token.rsplit_once(':') {
        (cards.trim(), Some(weight.trim()))
    } else {
        (token, None)
    };

    let hole_cards = HoleCards::from_str(cards_text).map_err(|_| RangeError::InvalidComboSyntax {
        token: token.to_string(),
    })?;

    let weight = match weight_text {
        Some(text) => text
            .parse::<f64>()
            .map_err(|_| RangeError::InvalidWeightSyntax {
                token: token.to_string(),
            })?,
        None => 1.0,
    };

    WeightedEntry::new(hole_cards, weight)
}

fn canonicalize_entries(entries: Vec<WeightedEntry>) -> Result<WeightedRange, RangeError> {
    if entries.is_empty() {
        return Err(RangeError::EmptyRange);
    }

    let mut ordered = Vec::with_capacity(entries.len());
    for entry in entries {
        if !entry.weight.is_finite() || entry.weight <= 0.0 {
            return Err(RangeError::InvalidWeight { weight: entry.weight });
        }
        ordered.push(entry);
    }

    ordered.sort_by(|left, right| left.hole_cards.cards().cmp(&right.hole_cards.cards()));

    let mut merged: Vec<WeightedEntry> = Vec::with_capacity(ordered.len());
    for entry in ordered {
        if let Some(last) = merged.last_mut() {
            if last.hole_cards == entry.hole_cards {
                last.weight += entry.weight;
                continue;
            }
        }
        merged.push(entry);
    }

    let total: f64 = merged.iter().map(|entry| entry.weight).sum();
    if !total.is_finite() || total <= 0.0 {
        return Err(RangeError::EmptyRange);
    }

    for entry in &mut merged {
        entry.weight /= total;
    }

    Ok(WeightedRange { entries: merged })
}

fn format_weight(weight: f64) -> String {
    let mut text = weight.to_string();
    if text.contains('.') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.push('0');
        }
    }
    text
}

fn approx_eq(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-12
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn parse_and_canonicalize_exact_combo_ranges() {
        let range = WeightedRange::parse_exact("KsAs:2,AsKs:2,AhKh:1").unwrap();
        assert_eq!(range.len(), 2);
        assert_eq!(range.canonical_string(), "KhAh:0.2,KsAs:0.8");
        assert_close(range.total_weight(), 1.0, 1e-12);
        assert_eq!(range.weight_of(&HoleCards::from_str("AsKs").unwrap()), Some(0.8));
        assert_eq!(range.entries()[0].hand, range.entries()[0].hole_cards);
    }

    #[test]
    fn parse_exact_accepts_bare_combos() {
        let range = WeightedRange::parse_exact("AsKs").unwrap();
        assert_eq!(range.len(), 1);
        assert_eq!(range.canonical_string(), "KsAs:1");
        assert_eq!(range.entries()[0].weight, 1.0);
    }

    #[test]
    fn overlap_filtering_is_exact() {
        let board = Board::from_str("As Kd Qh Jc Ts").unwrap();
        let range = WeightedRange::parse_exact("AsKs:2,QhJh:1,9c8c:1").unwrap();
        assert_eq!(range.first_board_overlap(&board), Some(Card::from_str("Qh").unwrap()));
        assert_eq!(range.validate_against_board(&board).unwrap_err(), RangeError::BoardConflict {
            hand: "JhQh".to_string(),
            card: Card::from_str("Qh").unwrap(),
        });

        let filtered = range.remove_board_blocked_combos(&board).unwrap();
        assert_eq!(filtered.canonical_string(), "8c9c:1");
    }

    #[test]
    fn range_overlap_filtering_is_exact() {
        let a = WeightedRange::parse_exact("AsKs:1,AhKh:1,9c8c:1").unwrap();
        let b = WeightedRange::parse_exact("AsQd:1").unwrap();
        let filtered = a.remove_overlaps_with_range(&b).unwrap();
        assert_eq!(filtered.canonical_string(), "8c9c:0.5,KhAh:0.5");
    }

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        let delta = (actual - expected).abs();
        assert!(
            delta <= tolerance,
            "expected {expected} ± {tolerance}, got {actual} (delta {delta})"
        );
    }
}
