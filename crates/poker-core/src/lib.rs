#![forbid(unsafe_code)]

use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    pub const ALL: [Self; 4] = [Self::Clubs, Self::Diamonds, Self::Hearts, Self::Spades];

    pub fn from_char(value: char) -> Result<Self, CardParseError> {
        match value.to_ascii_lowercase() {
            'c' => Ok(Self::Clubs),
            'd' => Ok(Self::Diamonds),
            'h' => Ok(Self::Hearts),
            's' => Ok(Self::Spades),
            _ => Err(CardParseError::InvalidSuit(value)),
        }
    }

    pub fn as_char(self) -> char {
        match self {
            Self::Clubs => 'c',
            Self::Diamonds => 'd',
            Self::Hearts => 'h',
            Self::Spades => 's',
        }
    }

    pub fn parse(value: char) -> Result<Self, CardParseError> {
        Self::from_char(value)
    }

    pub fn index(self) -> usize {
        match self {
            Self::Clubs => 0,
            Self::Diamonds => 1,
            Self::Hearts => 2,
            Self::Spades => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rank {
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
    Ace = 14,
}

impl Rank {
    pub const ALL: [Self; 13] = [
        Self::Two,
        Self::Three,
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::Eight,
        Self::Nine,
        Self::Ten,
        Self::Jack,
        Self::Queen,
        Self::King,
        Self::Ace,
    ];

    pub fn from_char(value: char) -> Result<Self, CardParseError> {
        match value.to_ascii_uppercase() {
            '2' => Ok(Self::Two),
            '3' => Ok(Self::Three),
            '4' => Ok(Self::Four),
            '5' => Ok(Self::Five),
            '6' => Ok(Self::Six),
            '7' => Ok(Self::Seven),
            '8' => Ok(Self::Eight),
            '9' => Ok(Self::Nine),
            'T' => Ok(Self::Ten),
            'J' => Ok(Self::Jack),
            'Q' => Ok(Self::Queen),
            'K' => Ok(Self::King),
            'A' => Ok(Self::Ace),
            _ => Err(CardParseError::InvalidRank(value)),
        }
    }

    pub fn as_char(self) -> char {
        match self {
            Self::Two => '2',
            Self::Three => '3',
            Self::Four => '4',
            Self::Five => '5',
            Self::Six => '6',
            Self::Seven => '7',
            Self::Eight => '8',
            Self::Nine => '9',
            Self::Ten => 'T',
            Self::Jack => 'J',
            Self::Queen => 'Q',
            Self::King => 'K',
            Self::Ace => 'A',
        }
    }

    pub fn parse(value: char) -> Result<Self, CardParseError> {
        Self::from_char(value)
    }

    pub fn value(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Card {
    rank: Rank,
    suit: Suit,
}

impl Card {
    pub fn new(rank: Rank, suit: Suit) -> Self {
        Self { rank, suit }
    }

    pub fn parse(value: &str) -> Result<Self, CardParseError> {
        value.parse()
    }

    pub fn parse_exact(value: &str) -> Result<Self, CardParseError> {
        Self::parse(value)
    }

    pub fn rank(self) -> Rank {
        self.rank
    }

    pub fn suit(self) -> Suit {
        self.suit
    }

    pub fn index(self) -> u8 {
        (self.suit.index() as u8) * 13 + (self.rank.value() - 2)
    }

    pub fn bit_mask(self) -> u64 {
        1u64 << self.index()
    }

    pub fn mask(self) -> u64 {
        self.bit_mask()
    }

    pub fn from_index(index: u8) -> Result<Self, PokerError> {
        if index >= 52 {
            return Err(PokerError::InvalidCardIndex(index));
        }
        let suit = match index / 13 {
            0 => Suit::Clubs,
            1 => Suit::Diamonds,
            2 => Suit::Hearts,
            _ => Suit::Spades,
        };
        let rank = match (index % 13) + 2 {
            2 => Rank::Two,
            3 => Rank::Three,
            4 => Rank::Four,
            5 => Rank::Five,
            6 => Rank::Six,
            7 => Rank::Seven,
            8 => Rank::Eight,
            9 => Rank::Nine,
            10 => Rank::Ten,
            11 => Rank::Jack,
            12 => Rank::Queen,
            13 => Rank::King,
            _ => Rank::Ace,
        };
        Ok(Self { rank, suit })
    }

    pub fn deck() -> Vec<Self> {
        (0u8..52)
            .map(|index| Self::from_index(index).expect("deck indices are always valid"))
            .collect()
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.rank.as_char(), self.suit.as_char())
    }
}

impl FromStr for Card {
    type Err = CardParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut chars = value.chars();
        let rank = Rank::from_char(chars.next().ok_or(CardParseError::WrongLength)?)?;
        let suit = Suit::from_char(chars.next().ok_or(CardParseError::WrongLength)?)?;
        if chars.next().is_some() {
            return Err(CardParseError::WrongLength);
        }
        Ok(Self::new(rank, suit))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HoleCards {
    first: Card,
    second: Card,
}

pub type PrivateHand = HoleCards;
pub type Player = PlayerRole;
pub type PlayerPosition = PlayerRole;
pub type ParseCardError = PokerError;

impl HoleCards {
    pub fn new(card_a: Card, card_b: Card) -> Result<Self, CardSetError> {
        if card_a == card_b {
            return Err(CardSetError::DuplicateCard(card_a));
        }
        let (first, second) = if card_a <= card_b { (card_a, card_b) } else { (card_b, card_a) };
        Ok(Self { first, second })
    }

    pub fn cards(self) -> [Card; 2] {
        [self.first, self.second]
    }

    pub fn iter(self) -> std::array::IntoIter<Card, 2> {
        self.cards().into_iter()
    }

    pub fn contains(self, card: Card) -> bool {
        self.first == card || self.second == card
    }

    pub fn overlaps(self, other: Self) -> bool {
        self.overlaps_hole_cards(other)
    }

    pub fn intersects(self, other: &Self) -> bool {
        self.overlaps_hole_cards(*other)
    }

    pub fn validate(self) -> Result<(), CardSetError> {
        if self.first == self.second {
            Err(CardSetError::DuplicateCard(self.first))
        } else {
            Ok(())
        }
    }

    pub fn parse(value: &str) -> Result<Self, PokerError> {
        value.parse().map_err(PokerError::HoleCards)
    }

    pub fn parse_exact(value: &str) -> Result<Self, PokerError> {
        Self::parse(value)
    }

    pub fn overlaps_hole_cards(self, other: Self) -> bool {
        self.contains(other.first) || self.contains(other.second)
    }

    pub fn conflicts_with(self, other: Self) -> bool {
        self.overlaps_hole_cards(other)
    }

    pub fn intersects_hole_cards(self, other: Self) -> bool {
        self.overlaps_hole_cards(other)
    }

    pub fn overlaps_hand(self, other: &Self) -> bool {
        self.overlaps_hole_cards(*other)
    }

    pub fn overlaps_board(self, board: &Board) -> bool {
        board.cards.iter().any(|card| self.contains(*card))
    }

    pub fn conflicts_with_board(self, board: &Board) -> bool {
        self.overlaps_board(board)
    }

    pub fn validate_against_board(self, board: &Board) -> Result<(), PokerError> {
        validate_hole_cards_against_board(self, board)
    }

    pub fn bit_mask(self) -> u64 {
        self.first.bit_mask() | self.second.bit_mask()
    }

    pub fn mask(self) -> u64 {
        self.bit_mask()
    }

    pub fn intersects_mask(self, mask: u64) -> bool {
        (self.bit_mask() & mask) != 0
    }
}

impl fmt::Display for HoleCards {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.first, self.second)
    }
}

impl FromStr for HoleCards {
    type Err = HoleCardsParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let compact: String = value.chars().filter(|char| !char.is_whitespace() && *char != ',').collect();
        if compact.len() != 4 {
            return Err(HoleCardsParseError::WrongLength { input: value.to_string() });
        }
        let first = Card::from_str(&compact[0..2]).map_err(HoleCardsParseError::Card)?;
        let second = Card::from_str(&compact[2..4]).map_err(HoleCardsParseError::Card)?;
        HoleCards::new(first, second).map_err(HoleCardsParseError::CardSet)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    cards: Vec<Card>,
}

impl Board {
    pub fn new(mut cards: Vec<Card>) -> Result<Self, CardSetError> {
        if !(3..=5).contains(&cards.len()) {
            return Err(CardSetError::WrongBoardLength(cards.len()));
        }
        cards.sort();
        for pair in cards.windows(2) {
            if pair[0] == pair[1] {
                return Err(CardSetError::DuplicateCard(pair[0]));
            }
        }
        Ok(Self { cards })
    }

    pub fn from_cards(cards: impl IntoIterator<Item = Card>) -> Result<Self, CardSetError> {
        Self::new(cards.into_iter().collect())
    }

    pub fn from_compact(value: &str) -> Result<Self, PokerError> {
        value.parse()
    }

    pub fn parse(value: &str) -> Result<Self, PokerError> {
        value.parse()
    }

    pub fn parse_exact(value: &str) -> Result<Self, PokerError> {
        Self::parse(value)
    }

    pub fn parse_compact(value: &str) -> Result<Self, PokerError> {
        Self::from_compact(value)
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

    pub fn iter(&self) -> std::slice::Iter<'_, Card> {
        self.cards.iter()
    }

    pub fn contains(&self, card: Card) -> bool {
        self.cards.contains(&card)
    }

    pub fn bit_mask(&self) -> u64 {
        self.cards.iter().fold(0u64, |mask, card| mask | card.bit_mask())
    }

    pub fn mask(&self) -> u64 {
        self.bit_mask()
    }

    pub fn street(&self) -> Result<Street, PokerError> {
        match self.cards.len() {
            3 => Ok(Street::Flop),
            4 => Ok(Street::Turn),
            5 => Ok(Street::River),
            actual => Err(PokerError::InvalidBoardLength(actual)),
        }
    }

    pub fn validate_for_street(&self, street: Street) -> Result<(), PokerError> {
        let found = self.street()?;
        if found != street {
            return Err(PokerError::BoardStreetMismatch {
                expected: street,
                found,
            });
        }
        Ok(())
    }

    pub fn validate_against_hole(&self, hole_cards: HoleCards) -> Result<(), PokerError> {
        validate_hole_cards_against_board(hole_cards, self)
    }

    pub fn to_river_array(&self) -> Result<[Card; 5], PokerError> {
        self.cards
            .clone()
            .try_into()
            .map_err(|_| PokerError::InvalidBoardLength(self.cards.len()))
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

impl FromStr for Board {
    type Err = PokerError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let compact: String = value.chars().filter(|char| !char.is_whitespace() && *char != ',').collect();
        if compact.len() % 2 != 0 {
            return Err(PokerError::InvalidBoardLiteral(value.to_string()));
        }
        let mut cards = Vec::new();
        for index in (0..compact.len()).step_by(2) {
            cards.push(Card::from_str(&compact[index..index + 2]).map_err(PokerError::Card)?);
        }
        Self::new(cards).map_err(PokerError::CardSet)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Street {
    Flop,
    Turn,
    River,
}

impl Street {
    pub const ALL: [Self; 3] = [Self::Flop, Self::Turn, Self::River];

    pub const fn board_cards(self) -> usize {
        match self {
            Self::Flop => 3,
            Self::Turn => 4,
            Self::River => 5,
        }
    }
}

impl fmt::Display for Street {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Flop => write!(f, "flop"),
            Self::Turn => write!(f, "turn"),
            Self::River => write!(f, "river"),
        }
    }
}

impl FromStr for Street {
    type Err = PokerError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "flop" => Ok(Self::Flop),
            "turn" => Ok(Self::Turn),
            "river" => Ok(Self::River),
            other => Err(PokerError::InvalidBoardLiteral(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PlayerRole {
    Oop,
    Ip,
}

impl PlayerRole {
    pub const ALL: [Self; 2] = [Self::Oop, Self::Ip];

    pub fn opponent(self) -> Self {
        match self {
            Self::Oop => Self::Ip,
            Self::Ip => Self::Oop,
        }
    }
}

impl fmt::Display for PlayerRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Oop => write!(f, "oop"),
            Self::Ip => write!(f, "ip"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardParseError {
    WrongLength,
    InvalidRank(char),
    InvalidSuit(char),
}

impl fmt::Display for CardParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CardParseError::WrongLength => write!(f, "card literal must be exactly two characters"),
            CardParseError::InvalidRank(rank) => write!(f, "invalid rank {rank}"),
            CardParseError::InvalidSuit(suit) => write!(f, "invalid suit {suit}"),
        }
    }
}

impl std::error::Error for CardParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoleCardsParseError {
    WrongLength { input: String },
    Card(CardParseError),
    CardSet(CardSetError),
}

impl fmt::Display for HoleCardsParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HoleCardsParseError::WrongLength { input } => {
                write!(f, "hole-card literal must describe exactly two cards: {input}")
            }
            HoleCardsParseError::Card(error) => write!(f, "{error}"),
            HoleCardsParseError::CardSet(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for HoleCardsParseError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardSetError {
    DuplicateCard(Card),
    WrongBoardLength(usize),
}

impl fmt::Display for CardSetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CardSetError::DuplicateCard(card) => write!(f, "duplicate card {card}"),
            CardSetError::WrongBoardLength(length) => {
                write!(f, "board must contain 3, 4, or 5 cards, found {length}")
            }
        }
    }
}

impl std::error::Error for CardSetError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PokerError {
    Card(CardParseError),
    CardSet(CardSetError),
    HoleCards(HoleCardsParseError),
    InvalidCardIndex(u8),
    InvalidBoardLiteral(String),
    InvalidBoardLength(usize),
    BoardStreetMismatch { expected: Street, found: Street },
    HoleCardsHitBoard { card: Card },
}

impl fmt::Display for PokerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PokerError::Card(error) => write!(f, "{error}"),
            PokerError::CardSet(error) => write!(f, "{error}"),
            PokerError::HoleCards(error) => write!(f, "{error}"),
            PokerError::InvalidCardIndex(index) => write!(f, "invalid card index {index}"),
            PokerError::InvalidBoardLiteral(value) => write!(f, "invalid board literal {value}"),
            PokerError::InvalidBoardLength(length) => write!(f, "invalid board length {length}"),
            PokerError::BoardStreetMismatch { expected, found } => {
                write!(f, "expected {expected} board, found {found}")
            }
            PokerError::HoleCardsHitBoard { card } => write!(f, "hole cards overlap board card {card}"),
        }
    }
}

impl std::error::Error for PokerError {}

impl From<CardParseError> for PokerError {
    fn from(value: CardParseError) -> Self {
        PokerError::Card(value)
    }
}

impl From<CardSetError> for PokerError {
    fn from(value: CardSetError) -> Self {
        PokerError::CardSet(value)
    }
}

impl From<HoleCardsParseError> for PokerError {
    fn from(value: HoleCardsParseError) -> Self {
        PokerError::HoleCards(value)
    }
}

pub fn validate_hole_cards_against_board(hole_cards: HoleCards, board: &Board) -> Result<(), PokerError> {
    hole_cards.validate().map_err(PokerError::CardSet)?;
    for card in hole_cards.cards() {
        if board.contains(card) {
            return Err(PokerError::HoleCardsHitBoard { card });
        }
    }
    Ok(())
}

pub fn parse_card(value: &str) -> Result<Card, PokerError> {
    Card::from_str(value).map_err(PokerError::Card)
}

pub fn parse_hole_cards(value: &str) -> Result<HoleCards, PokerError> {
    HoleCards::from_str(value).map_err(PokerError::HoleCards)
}

pub fn parse_board(value: &str) -> Result<Board, PokerError> {
    Board::from_str(value)
}

pub fn parse_street(value: &str) -> Result<Street, PokerError> {
    Street::from_str(value)
}

pub fn cards_are_unique(cards: &[Card]) -> bool {
    cards_are_disjoint(cards, &[])
}

pub fn cards_are_disjoint(left: &[Card], right: &[Card]) -> bool {
    for (index, card) in left.iter().enumerate() {
        if left[index + 1..].contains(card) || right.contains(card) {
            return false;
        }
    }
    for (index, card) in right.iter().enumerate() {
        if right[index + 1..].contains(card) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cards_hole_cards_and_board() {
        let ace_spades = Card::from_str("As").unwrap();
        assert_eq!(ace_spades.to_string(), "As");
        assert_eq!(ace_spades.index(), 51);
        assert_eq!(Card::from_index(51).unwrap(), ace_spades);

        let hole_cards = HoleCards::from_str("AsKd").unwrap();
        assert_eq!(hole_cards.to_string(), "KdAs");

        let board = Board::from_str("AhKhQcJd2s").unwrap();
        assert_eq!(board.street().unwrap(), Street::River);
        assert_eq!(board.to_string(), "2sJdQcKhAh");
        assert!(!hole_cards.overlaps_board(&board));
    }

    #[test]
    fn validates_street_and_overlap_rules() {
        let board = Board::from_compact("AsKhQcJd2s").unwrap();
        assert!(board.validate_for_street(Street::River).is_ok());
        let hole_cards = HoleCards::from_str("AsKd").unwrap();
        assert!(validate_hole_cards_against_board(hole_cards, &board).is_err());
    }

    #[test]
    fn parsing_helpers_are_public_and_canonical() {
        assert_eq!(parse_card("As").unwrap().to_string(), "As");
        assert_eq!(parse_hole_cards("AsKd").unwrap().to_string(), "KdAs");
        assert_eq!(parse_board("AsKhQcJd2s").unwrap().street().unwrap(), Street::River);
        assert_eq!(parse_street("turn").unwrap(), Street::Turn);
        assert!(cards_are_disjoint(&parse_board("AsKhQcJd2s").unwrap().cards().to_vec(), &parse_hole_cards("2c3d").unwrap().cards()));
    }

    #[test]
    fn deck_covers_every_card_exactly_once() {
        let deck = Card::deck();
        assert_eq!(deck.len(), 52);
        let mut deduped = deck.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(deduped.len(), 52);
        assert_eq!(deck.first().unwrap().to_string(), "2c");
        assert_eq!(deck.last().unwrap().to_string(), "As");
    }
}
