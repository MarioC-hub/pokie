use std::{collections::BTreeMap, fs, path::PathBuf, str::FromStr};

use config_core::{
    AlgorithmFamily, BlindProfile, GameConfig, GameMode, NodeLock, PlayerRole, RakeProfile,
    RangeConfig, RiverTreeTemplate, RootState, SolveConfig, SolverSettings, Variant,
    RIVER_SINGLE_BET_TEMPLATE_ID,
};
use poker_core::{Board, HoleCards, Street};
use range_core::WeightedRange;
use solver_core::{
    cfr::CfrState,
    oracle::{exact_exploitability, expected_value_p0},
    poker::{build_river_game, solve_river_spot},
    profile::StrategyProfile,
};
use tree_core::build_river_tree;

fn polarized_config(iterations: usize) -> SolveConfig {
    SolveConfig {
        schema_version: 1,
        game: GameConfig {
            variant: Variant::Nlhe,
            mode: GameMode::Cash,
            active_players: 2,
        },
        root_state: RootState {
            street: Street::River,
            board: Board::from_str("Ks7d4c2h2d").unwrap(),
            oop_stack: 100,
            ip_stack: 100,
            pot_size: 10,
            player_to_act: PlayerRole::Oop,
            blind_profile: BlindProfile {
                small_blind: 1,
                big_blind: 2,
            },
            rake_profile: RakeProfile {
                rake_bps: 0,
                cap: 0,
            },
        },
        ranges: RangeConfig {
            oop_range: WeightedRange::from_hands(vec![
                (HoleCards::from_str("7c7h").unwrap(), 1.0),
                (HoleCards::from_str("AcJc").unwrap(), 1.0),
            ])
            .unwrap(),
            ip_range: WeightedRange::from_hands(vec![(HoleCards::from_str("KcQh").unwrap(), 1.0)])
                .unwrap(),
        },
        tree_template: RiverTreeTemplate {
            template_id: RIVER_SINGLE_BET_TEMPLATE_ID.to_string(),
            template_version: 1,
            first_bet_size: 10,
            after_check_bet_size: 0,
            max_raises_per_street: 0,
            allow_all_in: false,
        },
        node_locks: Vec::<NodeLock>::new(),
        solver_settings: SolverSettings {
            algorithm: AlgorithmFamily::Cfr,
            iterations,
            checkpoint_cadence: 0,
            thread_count: 1,
            deterministic_seed: 0,
        },
    }
}

struct PokerEquilibriumFixture {
    expected_value_p0: f64,
    max_nash_conv: f64,
    profile: BTreeMap<String, Vec<f64>>,
}

fn infoset_key(role: &str, hole_cards: &str, history: &str) -> String {
    let hand = HoleCards::from_str(hole_cards).unwrap();
    format!("river:{role}:{hand}:{history}")
}

#[test]
fn polarized_river_game_shape_is_exact() {
    let game = build_river_game(&polarized_config(1)).unwrap();
    assert_eq!(game.chance_count(), 1);
    assert_eq!(game.decision_count(), 4);
    assert_eq!(game.terminal_count(), 6);
    assert_eq!(game.infosets().len(), 3);
}

#[test]
fn polarized_river_exact_equilibrium_profile_is_zero_exploitability() {
    let config = polarized_config(1);
    let game = build_river_game(&config).unwrap();
    let fixture = load_poker_equilibrium_fixture();

    let profile = StrategyProfile::from_named_probabilities(&game, &fixture.profile).unwrap();
    let report = exact_exploitability(&game, &profile);

    assert_close(
        expected_value_p0(&game, &profile),
        fixture.expected_value_p0,
        1e-12,
    );
    assert_close(report.profile_value_p0, fixture.expected_value_p0, 1e-12);
    assert!(
        report.nash_conv <= fixture.max_nash_conv,
        "nash_conv too high: {}",
        report.nash_conv
    );
    assert!(
        report.p0_improvement <= fixture.max_nash_conv,
        "p0 improvement too high: {}",
        report.p0_improvement
    );
    assert!(
        report.p1_improvement <= fixture.max_nash_conv,
        "p1 improvement too high: {}",
        report.p1_improvement
    );
}

#[test]
fn polarized_river_first_iteration_regrets_match_exact_fixture() {
    let config = polarized_config(1);
    let game = build_river_game(&config).unwrap();

    let mut state = CfrState::new(&game);
    let report = state.run_iteration(&game);
    assert_eq!(report.iteration_index, 1);

    assert_slice_close(
        state
            .regret_row_by_key(&game, &infoset_key("oop", "7c7h", "root"))
            .unwrap(),
        &[-1.25, 1.25],
        1e-12,
    );
    assert_slice_close(
        state
            .regret_row_by_key(&game, &infoset_key("oop", "AcJc", "root"))
            .unwrap(),
        &[0.0, 0.0],
        1e-12,
    );
    assert_slice_close(
        state
            .regret_row_by_key(&game, &infoset_key("ip", "KcQh", "b"))
            .unwrap(),
        &[-1.25, 1.25],
        1e-12,
    );
}

#[test]
fn solve_river_spot_one_iteration_is_exact_and_self_consistent() {
    let config = polarized_config(1);
    let tree = build_river_tree(&config).unwrap();
    let result = solve_river_spot(&config).unwrap();

    assert_eq!(result.iterations, 1);
    assert_eq!(result.config_hash, config.canonical_hash().unwrap());
    assert_eq!(result.tree_identity, tree.tree_identity);
    assert_close(
        result.exploitability.profile_value_p0,
        expected_value_p0(&result.game, &result.average_profile),
        1e-12,
    );

    assert_slice_close(
        result
            .average_profile
            .probability_by_key(&result.game, &infoset_key("oop", "7c7h", "root"))
            .unwrap(),
        &[0.5, 0.5],
        1e-12,
    );
    assert_slice_close(
        result
            .average_profile
            .probability_by_key(&result.game, &infoset_key("oop", "AcJc", "root"))
            .unwrap(),
        &[0.5, 0.5],
        1e-12,
    );
    assert_slice_close(
        result
            .average_profile
            .probability_by_key(&result.game, &infoset_key("ip", "KcQh", "b"))
            .unwrap(),
        &[0.5, 0.5],
        1e-12,
    );

    assert_slice_close(
        result
            .current_profile
            .probability_by_key(&result.game, &infoset_key("oop", "7c7h", "root"))
            .unwrap(),
        &[0.0, 1.0],
        1e-12,
    );
    assert_slice_close(
        result
            .current_profile
            .probability_by_key(&result.game, &infoset_key("oop", "AcJc", "root"))
            .unwrap(),
        &[0.5, 0.5],
        1e-12,
    );
    assert_slice_close(
        result
            .current_profile
            .probability_by_key(&result.game, &infoset_key("ip", "KcQh", "b"))
            .unwrap(),
        &[0.0, 1.0],
        1e-12,
    );

    assert_slice_close(
        result
            .strategy_by_infoset
            .get(&infoset_key("oop", "7c7h", "root"))
            .unwrap(),
        &[0.5, 0.5],
        1e-12,
    );
    assert_slice_close(
        result
            .strategy_by_infoset
            .get(&infoset_key("oop", "AcJc", "root"))
            .unwrap(),
        &[0.5, 0.5],
        1e-12,
    );
    assert_slice_close(
        result
            .strategy_by_infoset
            .get(&infoset_key("ip", "KcQh", "b"))
            .unwrap(),
        &[0.5, 0.5],
        1e-12,
    );
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tolerance,
        "expected {expected} ± {tolerance}, got {actual} (delta {delta})"
    );
}

fn assert_slice_close(actual: &[f64], expected: &[f64], tolerance: f64) {
    assert_eq!(actual.len(), expected.len(), "slice lengths differ");
    for (actual, expected) in actual.iter().zip(expected.iter()) {
        assert_close(*actual, *expected, tolerance);
    }
}

fn load_poker_equilibrium_fixture() -> PokerEquilibriumFixture {
    let text = fs::read_to_string(fixture_dir().join("river_polarized_equilibrium.fixture"))
        .expect("poker equilibrium fixture should read");
    let mut expected_value_p0 = None;
    let mut max_nash_conv = None;
    let mut profile = BTreeMap::new();

    for line in meaningful_lines(&text) {
        let (key, value) = split_key_value(line);
        match key {
            "expected_value_p0" => expected_value_p0 = Some(parse_f64(value)),
            "max_nash_conv" => max_nash_conv = Some(parse_f64(value)),
            _ => {
                profile.insert(key.to_string(), parse_f64_list(value));
            }
        }
    }

    PokerEquilibriumFixture {
        expected_value_p0: expected_value_p0.expect("expected_value_p0 missing"),
        max_nash_conv: max_nash_conv.expect("max_nash_conv missing"),
        profile,
    }
}

fn meaningful_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

fn split_key_value(line: &str) -> (&str, &str) {
    line.split_once('=')
        .unwrap_or_else(|| panic!("expected key=value line, got {line}"))
}

fn parse_f64_list(value: &str) -> Vec<f64> {
    value
        .split(',')
        .map(|part| parse_f64(part.trim()))
        .collect()
}

fn parse_f64(value: &str) -> f64 {
    value
        .parse::<f64>()
        .unwrap_or_else(|_| panic!("expected f64, got {value}"))
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/conformance")
}
