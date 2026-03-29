use core::cmp::Ordering;

use equity_core::{
    compare_hole_cards_on_board, evaluate_five_card_hand, evaluate_seven_card_hand, HandCategory,
};
use poker_core::{parse_board, parse_card, parse_hole_cards, Card, Rank::*, Suit::*};

fn c(rank: poker_core::Rank, suit: poker_core::Suit) -> Card {
    Card::new(rank, suit)
}

#[test]
fn evaluate_seven_card_hand_detects_a_royal_straight_flush() {
    let board = parse_board("AhKhQhJh2c").unwrap();
    let hole = parse_hole_cards("Th9h").unwrap();

    let value = evaluate_seven_card_hand(&board, &hole).unwrap();
    assert_eq!(value.category(), HandCategory::StraightFlush);
    assert_eq!(value.kickers()[0], 14);
}

#[test]
fn compare_hole_cards_on_board_orders_a_clear_winner_exactly() {
    let board = parse_board("AsKdQcJh9h").unwrap();
    let winner = parse_hole_cards("Ts2c").unwrap();
    let loser = parse_hole_cards("3c4d").unwrap();

    assert_eq!(
        compare_hole_cards_on_board(&winner, &loser, &board).unwrap(),
        Ordering::Greater
    );
}

#[test]
fn compare_hole_cards_on_board_returns_exact_ties_when_board_plays() {
    let board = parse_board("AsKdQcJhTs").unwrap();
    let first = parse_hole_cards("2c3d").unwrap();
    let second = parse_hole_cards("4c5d").unwrap();

    assert_eq!(
        compare_hole_cards_on_board(&first, &second, &board).unwrap(),
        Ordering::Equal
    );
}

#[test]
fn evaluate_five_card_hand_orders_categories_and_kickers_exactly() {
    let full_house = evaluate_five_card_hand([
        parse_card("Ah").unwrap(),
        parse_card("Ad").unwrap(),
        parse_card("Ac").unwrap(),
        parse_card("Kd").unwrap(),
        parse_card("Ks").unwrap(),
    ]);
    let flush = evaluate_five_card_hand([
        parse_card("Ah").unwrap(),
        parse_card("Kh").unwrap(),
        parse_card("Th").unwrap(),
        parse_card("4h").unwrap(),
        parse_card("2h").unwrap(),
    ]);

    assert!(full_house > flush);

    let ace_pair = evaluate_five_card_hand([c(Ace, Spades), c(Ace, Clubs), c(King, Diamonds), c(Queen, Hearts), c(Jack, Hearts)]);
    let weaker_ace_pair = evaluate_five_card_hand([c(Ace, Spades), c(Ace, Clubs), c(Queen, Diamonds), c(Jack, Hearts), c(Ten, Diamonds)]);
    assert!(ace_pair > weaker_ace_pair);
}

#[test]
fn overlapping_hole_cards_are_rejected_exactly() {
    let board = parse_board("AsKdQcJh9h").unwrap();
    let hole = parse_hole_cards("As2c").unwrap();

    let err = evaluate_seven_card_hand(&board, &hole).unwrap_err();
    assert!(matches!(err, equity_core::EquityError::Poker(poker_core::PokerError::HoleCardsHitBoard { card }) if card == parse_card("As").unwrap()));
}
