use crate::{validate_hole_cards_against_board, Board, Card, CardSetError, PrivateHand, Rank, Suit};

#[test]
fn parses_cards_ranks_and_suits_deterministically() {
    let card = Card::parse("As").unwrap();
    assert_eq!(card.rank(), Rank::Ace);
    assert_eq!(card.suit(), Suit::Spades);
    assert_eq!(card.to_string(), "As");
    assert_eq!(Rank::parse('T').unwrap(), Rank::Ten);
    assert_eq!(Suit::parse('h').unwrap(), Suit::Hearts);
}

#[test]
fn deck_has_52_unique_cards() {
    let deck = Card::deck();
    assert_eq!(deck.len(), 52);
    let mut unique = deck.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), 52);
    assert_eq!(deck.first().unwrap().to_string(), "2c");
    assert_eq!(deck.last().unwrap().to_string(), "As");
}

#[test]
fn private_hand_canonicalizes_exact_combos() {
    let hand = PrivateHand::parse_exact("KsAs").unwrap();
    assert_eq!(hand.to_string(), "AsKs");
    assert_eq!(hand.cards()[0].to_string(), "Ks");
    assert_eq!(hand.cards()[1].to_string(), "As");
}

#[test]
fn board_parsing_and_overlap_validation_work() {
    let board = Board::parse_exact("2c3d4h5sAs").unwrap();
    assert_eq!(board.to_string(), "2c3d4h5sAs");
    let hand = PrivateHand::parse_exact("KhQh").unwrap();
    validate_hole_cards_against_board(hand, board).unwrap();
    let overlapping = PrivateHand::parse_exact("AsKd").unwrap();
    let err = validate_hole_cards_against_board(overlapping, board).unwrap_err();
    assert!(matches!(err, CardSetError::Overlap { card } if card.to_string() == "As"));
}

#[test]
fn duplicate_cards_and_invalid_lengths_are_rejected() {
    assert!(matches!(PrivateHand::parse_exact("AsAs"), Err(CardSetError::DuplicateCard { .. })));
    assert!(matches!(Board::parse_exact("AsKsQsJs"), Err(CardSetError::InvalidCardCount { .. })));
    assert!(matches!(Card::parse("1x"), Err(_)));
}
