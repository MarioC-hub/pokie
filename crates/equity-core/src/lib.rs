#![forbid(unsafe_code)]

//! Exact seven-card river showdown evaluation for NLHE.

use std::{cmp::Ordering, fmt};

use poker_core::{
    validate_hole_cards_against_board, Board, Card, CardSetError, HoleCards, PokerError,
    PrivateHand, Street,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandCategory {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandRank {
    category: HandCategory,
    kickers: [u8; 5],
}

pub type HandValue = HandRank;
pub type HandStrength = HandRank;
pub type ShowdownStrength = HandRank;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShowdownOutcome {
    OopWins,
    IpWins,
    Tie,
}

impl HandRank {
    pub const fn new(category: HandCategory, kickers: [u8; 5]) -> Self {
        Self { category, kickers }
    }

    pub const fn category(self) -> HandCategory {
        self.category
    }

    pub const fn kickers(self) -> [u8; 5] {
        self.kickers
    }
}

impl Ord for HandRank {
    fn cmp(&self, other: &Self) -> Ordering {
        self.category
            .cmp(&other.category)
            .then_with(|| self.kickers.cmp(&other.kickers))
    }
}

impl PartialOrd for HandRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquityError {
    DuplicateCard(Card),
    CardSet(CardSetError),
    Poker(PokerError),
}

impl fmt::Display for EquityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EquityError::DuplicateCard(card) => write!(f, "duplicate card {card}"),
            EquityError::CardSet(err) => write!(f, "{err}"),
            EquityError::Poker(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for EquityError {}

impl From<CardSetError> for EquityError {
    fn from(value: CardSetError) -> Self {
        Self::CardSet(value)
    }
}

impl From<PokerError> for EquityError {
    fn from(value: PokerError) -> Self {
        Self::Poker(value)
    }
}

pub fn evaluate_seven(board: &Board, hand: &PrivateHand) -> Result<HandRank, EquityError> {
    board
        .validate_for_street(Street::River)
        .map_err(EquityError::from)?;
    validate_hole_cards_against_board(*hand, board)?;
    hand.validate().map_err(EquityError::from)?;
    let cards = combined_cards(board, hand);
    let mut best = None;
    for omit_a in 0..cards.len() {
        for omit_b in (omit_a + 1)..cards.len() {
            let mut subset = [cards[0]; 5];
            let mut cursor = 0;
            for (index, card) in cards.iter().copied().enumerate() {
                if index == omit_a || index == omit_b {
                    continue;
                }
                subset[cursor] = card;
                cursor += 1;
            }
            let value = evaluate_five(subset);
            if best.is_none_or(|current| value > current) {
                best = Some(value);
            }
        }
    }
    Ok(best.expect("seven cards always yield at least one five-card subset"))
}

pub fn compare_hands_checked(
    board: &Board,
    oop: &PrivateHand,
    ip: &PrivateHand,
) -> Result<Ordering, EquityError> {
    board
        .validate_for_street(Street::River)
        .map_err(EquityError::from)?;
    validate_hole_cards_against_board(*oop, board)?;
    validate_hole_cards_against_board(*ip, board)?;
    oop.validate().map_err(EquityError::from)?;
    ip.validate().map_err(EquityError::from)?;
    validate_unique_showdown_cards(board, oop, ip)?;
    let oop_value = evaluate_seven(board, oop)?;
    let ip_value = evaluate_seven(board, ip)?;
    Ok(oop_value.cmp(&ip_value))
}

pub fn compare_hands(board: &Board, oop: HoleCards, ip: HoleCards) -> Ordering {
    compare_hands_checked(board, &oop, &ip).expect("invalid showdown inputs")
}

pub fn compare_hands_result(
    board: &Board,
    oop: HoleCards,
    ip: HoleCards,
) -> Result<Ordering, EquityError> {
    compare_hands_checked(board, &oop, &ip)
}

pub fn evaluate_strength(board: &Board, hand: &PrivateHand) -> Result<HandStrength, EquityError> {
    evaluate_seven(board, hand)
}

pub fn evaluate_five_card_hand(cards: [Card; 5]) -> Result<HandRank, String> {
    Ok(evaluate_five(cards))
}

pub fn evaluate_seven_card_hand(board: &Board, hand: &HoleCards) -> Result<HandRank, EquityError> {
    evaluate_seven(board, hand)
}

pub fn evaluate_river_hand(board: &Board, hand: &HoleCards) -> Result<HandRank, EquityError> {
    evaluate_seven(board, hand)
}

pub fn compare_hole_cards_on_board(
    first: &HoleCards,
    second: &HoleCards,
    board: &Board,
) -> Result<Ordering, EquityError> {
    compare_hands_checked(board, first, second)
}

pub fn showdown(
    board: &Board,
    oop: HoleCards,
    ip: HoleCards,
) -> Result<ShowdownOutcome, EquityError> {
    match compare_hands_checked(board, &oop, &ip)? {
        Ordering::Less => Ok(ShowdownOutcome::IpWins),
        Ordering::Equal => Ok(ShowdownOutcome::Tie),
        Ordering::Greater => Ok(ShowdownOutcome::OopWins),
    }
}

fn combined_cards(board: &Board, hand: &PrivateHand) -> [Card; 7] {
    let board_cards = board.cards();
    let hand_cards = hand.cards();
    [
        hand_cards[0],
        hand_cards[1],
        board_cards[0],
        board_cards[1],
        board_cards[2],
        board_cards[3],
        board_cards[4],
    ]
}

fn validate_unique_showdown_cards(
    board: &Board,
    first: &PrivateHand,
    second: &PrivateHand,
) -> Result<(), EquityError> {
    for card in first.iter() {
        if board.contains(card) {
            return Err(EquityError::DuplicateCard(card));
        }
    }
    for card in second.iter() {
        if board.contains(card) {
            return Err(EquityError::DuplicateCard(card));
        }
    }
    for card in first.iter() {
        if second.contains(card) {
            return Err(EquityError::DuplicateCard(card));
        }
    }
    Ok(())
}

fn evaluate_five(cards: [Card; 5]) -> HandRank {
    let mut rank_counts = [0u8; 15];
    let mut suit_counts = [0u8; 4];
    for card in cards {
        rank_counts[card.rank() as usize] += 1;
        suit_counts[card.suit().index()] += 1;
    }
    let flush = suit_counts.contains(&5);
    let straight_high = straight_high(&rank_counts);
    let groups = rank_groups(&rank_counts);
    let rank_vector = ranks_desc(&rank_counts);

    if flush {
        if let Some(high) = straight_high {
            return HandRank::new(HandCategory::StraightFlush, [high, 0, 0, 0, 0]);
        }
    }

    if groups[0].0 == 4 {
        return HandRank::new(
            HandCategory::FourOfAKind,
            [groups[0].1, groups[1].1, 0, 0, 0],
        );
    }
    if groups[0].0 == 3 && groups[1].0 == 2 {
        return HandRank::new(HandCategory::FullHouse, [groups[0].1, groups[1].1, 0, 0, 0]);
    }
    if flush {
        return HandRank::new(HandCategory::Flush, rank_vector);
    }
    if let Some(high) = straight_high {
        return HandRank::new(HandCategory::Straight, [high, 0, 0, 0, 0]);
    }
    if groups[0].0 == 3 {
        let kickers = kicker_ranks(&groups, 2);
        return HandRank::new(
            HandCategory::ThreeOfAKind,
            [groups[0].1, kickers[0], kickers[1], 0, 0],
        );
    }
    if groups[0].0 == 2 && groups[1].0 == 2 {
        let high_pair = groups[0].1.max(groups[1].1);
        let low_pair = groups[0].1.min(groups[1].1);
        let kicker = groups.iter().find(|group| group.0 == 1).unwrap().1;
        return HandRank::new(HandCategory::TwoPair, [high_pair, low_pair, kicker, 0, 0]);
    }
    if groups[0].0 == 2 {
        let kickers = kicker_ranks(&groups, 3);
        return HandRank::new(
            HandCategory::OnePair,
            [groups[0].1, kickers[0], kickers[1], kickers[2], 0],
        );
    }

    HandRank::new(HandCategory::HighCard, rank_vector)
}

fn kicker_ranks(groups: &[(u8, u8)], count: usize) -> Vec<u8> {
    let mut kickers = groups
        .iter()
        .filter_map(|group| (group.0 == 1).then_some(group.1))
        .collect::<Vec<_>>();
    kickers.sort_unstable_by(|left, right| right.cmp(left));
    kickers.truncate(count);
    kickers
}

fn rank_groups(rank_counts: &[u8; 15]) -> Vec<(u8, u8)> {
    let mut groups = (2u8..=14)
        .rev()
        .filter_map(|rank| {
            let count = rank_counts[rank as usize];
            (count > 0).then_some((count, rank))
        })
        .collect::<Vec<_>>();
    groups.sort_by(|left, right| right.cmp(left));
    groups
}

fn ranks_desc(rank_counts: &[u8; 15]) -> [u8; 5] {
    let mut result = [0u8; 5];
    let mut cursor = 0;
    for rank in (2u8..=14).rev() {
        for _ in 0..rank_counts[rank as usize] {
            result[cursor] = rank;
            cursor += 1;
        }
    }
    result
}

fn straight_high(rank_counts: &[u8; 15]) -> Option<u8> {
    let mut present = [false; 15];
    for rank in 2..=14 {
        present[rank] = rank_counts[rank] > 0;
    }
    if present[14] {
        present[1] = true;
    }

    for high in (5u8..=14).rev() {
        if (0..5).all(|offset| present[(high - offset) as usize]) {
            return Some(high);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use poker_core::{Board, PrivateHand};

    use super::{compare_hands_checked, evaluate_seven, HandCategory};

    #[test]
    fn evaluator_orders_representative_hand_classes() {
        let straight_flush_board = Board::from_str("AhKhQhJh2c").unwrap();
        let full_house_board = Board::from_str("AhKhQhJdJc").unwrap();
        let straight_flush = PrivateHand::from_str("Th9d").unwrap();
        let full_house = PrivateHand::from_str("AsAc").unwrap();

        let straight_flush_rank = evaluate_seven(&straight_flush_board, &straight_flush).unwrap();
        let full_house_rank = evaluate_seven(&full_house_board, &full_house).unwrap();

        assert_eq!(straight_flush_rank.category(), HandCategory::StraightFlush);
        assert_eq!(full_house_rank.category(), HandCategory::FullHouse);
        assert!(straight_flush_rank > full_house_rank);
    }

    #[test]
    fn compare_hands_detects_ties_and_winners() {
        let board = Board::from_str("KhTd8s5c2h").unwrap();
        let value = PrivateHand::from_str("AsAh").unwrap();
        let bluff_catcher = PrivateHand::from_str("KdQd").unwrap();
        let air = PrivateHand::from_str("3c4c").unwrap();

        assert!(compare_hands_checked(&board, &value, &bluff_catcher)
            .unwrap()
            .is_gt());
        assert!(compare_hands_checked(&board, &air, &bluff_catcher)
            .unwrap()
            .is_lt());

        let tie_board = Board::from_str("AhKdQsJcTc").unwrap();
        let tie_a = PrivateHand::from_str("2c3d").unwrap();
        let tie_b = PrivateHand::from_str("4h5s").unwrap();
        assert!(compare_hands_checked(&tie_board, &tie_a, &tie_b)
            .unwrap()
            .is_eq());
    }
}
