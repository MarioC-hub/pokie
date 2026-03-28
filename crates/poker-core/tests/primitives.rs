use std::str::FromStr;

use poker_core::{
    parse_board, parse_card, parse_hole_cards, validate_hole_cards_against_board, Board, Card, CardParseError,
    CardSetError, HoleCardsParseError, PokerError, Rank, Street, Suit,
};

#[test]
fn card_parse_display_and_encoding_roundtrip_for_all_52_cards() {
    for index in 0..52 {
        let card = Card::from_index(index).unwrap();
        let text = card.to_string();
        assert_eq!(parse_card(&text).unwrap(), card);
        assert_eq!(Card::from_str(&text).unwrap(), card);
        assert_eq!(Card::from_index(card.index()).unwrap(), card);
    }
}

#[test]
fn hole_cards_parse_and_display_roundtrip() {
    let hand = parse_hole_cards("AsKd").unwrap();
    assert_eq!(hand.to_string(), "KdAs");
    assert_eq!(poker_core::PrivateHand::parse_exact("AsKd").unwrap(), hand);

    let cards = hand.cards();
    assert_eq!(cards[0], Card::from_str("Kd").unwrap());
    assert_eq!(cards[1], Card::from_str("As").unwrap());
}

#[test]
fn board_parse_and_validation_rejects_duplicates() {
    let board = parse_board("AsKsQsJsTs").unwrap();
    assert_eq!(board.cards().len(), 5);
    assert_eq!(board.street().unwrap(), Street::River);
    assert_eq!(board.to_string(), "TsJsQsKsAs");
    assert_eq!(Board::parse_exact("AsKsQsJsTs").unwrap(), board);

    let duplicate = parse_board("AsAsQsJsTs").unwrap_err();
    assert!(matches!(duplicate, PokerError::CardSet(CardSetError::DuplicateCard(_))));
}

#[test]
fn hole_cards_reject_duplicate_private_cards() {
    let err = parse_hole_cards("AsAs").unwrap_err();
    assert!(matches!(
        err,
        PokerError::HoleCards(HoleCardsParseError::CardSet(CardSetError::DuplicateCard(_)))
    ));
}

#[test]
fn board_and_hole_cards_have_the_expected_counts_and_overlap_rules() {
    let board = parse_board("AhKdQcJs9h").unwrap();
    let hole = parse_hole_cards("2c3d").unwrap();

    assert_eq!(board.cards().len(), 5);
    assert_eq!(hole.cards().len(), 2);
    assert!(validate_hole_cards_against_board(hole, &board).is_ok());

    let overlapping = parse_hole_cards("9h2c").unwrap();
    let err = validate_hole_cards_against_board(overlapping, &board).unwrap_err();
    assert!(matches!(err, PokerError::HoleCardsHitBoard { card } if card == Card::from_str("9h").unwrap()));
}

#[test]
fn parsers_reject_invalid_literals_exactly() {
    let card_err = parse_card("1x").unwrap_err();
    assert!(matches!(card_err, PokerError::Card(CardParseError::InvalidRank('1'))));

    let board_err = parse_board("As Ks").unwrap_err();
    assert!(matches!(board_err, PokerError::CardSet(CardSetError::WrongBoardLength(2))));
}

#[test]
fn board_and_hole_helpers_are_deterministic_about_sorting() {
    let board = parse_board("Ks As Qd Jc Tc").unwrap();
    assert_eq!(board.to_string(), "TcJcQdKsAs");

    let hole = parse_hole_cards("Ks As").unwrap();
    assert_eq!(hole.to_string(), "KsAs");
}

#[test]
fn support_types_cover_expected_ordering_and_deck_size() {
    assert_eq!(Rank::Two.value(), 2);
    assert_eq!(Rank::Ace.value(), 14);
    assert_eq!(Suit::Clubs.index(), 0);
    assert_eq!(Suit::Spades.index(), 3);

    let deck = (0u8..52).map(|index| Card::from_index(index).unwrap()).collect::<Vec<_>>();
    assert_eq!(deck.len(), 52);
    assert_eq!(deck.first().unwrap().to_string(), "2c");
    assert_eq!(deck.last().unwrap().to_string(), "As");
}
