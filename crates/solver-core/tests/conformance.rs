use std::{collections::BTreeMap, fs, path::PathBuf};

use solver_core::{
    cfr::CfrState,
    game::Player,
    games::{kuhn, matrix},
    oracle::{exact_exploitability, expected_value_p0, pure_best_response_value_p0},
    profile::StrategyProfile,
};

#[test]
fn fixture_manifest_lists_known_conformance_assets() {
    let manifest = load_manifest();
    let fixture_ids: Vec<_> = manifest.iter().map(|entry| entry.0.as_str()).collect();
    assert_eq!(
        fixture_ids,
        vec![
            "kuhn_reference_equilibrium_v1",
            "skewed_matrix_cfr_iteration_1_v1",
            "river_polarized_equilibrium_v1",
        ]
    );

    for (_, path) in manifest {
        let path = fixture_dir().join(path);
        assert!(path.exists(), "fixture {:?} should exist", path);
    }
}

#[test]
fn kuhn_game_shape_matches_exact_contract() {
    let game = kuhn::game();
    assert_eq!(game.chance_count(), 1);
    assert_eq!(game.decision_count(), 24);
    assert_eq!(game.terminal_count(), 30);
    assert_eq!(game.infosets().len(), 12);
    assert_eq!(game.player_infosets(Player::P0).len(), 6);
    assert_eq!(game.player_infosets(Player::P1).len(), 6);
}

#[test]
fn kuhn_terminal_payoffs_are_exact_for_representative_histories() {
    let game = kuhn::game();
    let history_to_utility = game
        .nodes()
        .iter()
        .filter_map(|node| match node.kind {
            solver_core::game::NodeKind::Terminal { utility_p0 } => Some((node.history.as_str(), utility_p0)),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert!(history_to_utility.contains(&("bet-fold", 1.0)));
    assert!(history_to_utility.contains(&("check-bet-fold", -1.0)));
    assert!(history_to_utility.contains(&("check-check", 1.0)));
    assert!(history_to_utility.contains(&("check-check", -1.0)));
    assert!(history_to_utility.contains(&("bet-call", 2.0)));
    assert!(history_to_utility.contains(&("bet-call", -2.0)));
}

#[test]
fn kuhn_reference_equilibrium_has_zero_exploitability_and_exact_value() {
    let game = kuhn::game();
    let fixture = load_kuhn_equilibrium_fixture();
    let profile = StrategyProfile::from_named_probabilities(&game, &fixture.profile).unwrap();

    let report = exact_exploitability(&game, &profile);
    assert_close(report.profile_value_p0, fixture.expected_value_p0, 1e-12);
    assert!(report.nash_conv <= fixture.max_nash_conv, "exploitability too high: {}", report.nash_conv);
    assert!(report.p0_improvement <= fixture.max_nash_conv);
    assert!(report.p1_improvement <= fixture.max_nash_conv);

    let hard_coded_reference = kuhn::reference_equilibrium(&game);
    for infoset in game.infosets() {
        let expected = profile.probabilities(infoset.id);
        let actual = hard_coded_reference.probabilities(infoset.id);
        assert_slice_close(actual, expected, 1e-12);
    }
}

#[test]
fn skewed_matrix_first_iteration_regrets_match_fixture_exactly() {
    let game = matrix::simultaneous_matrix_game(
        "skewed_matrix",
        &["A", "B"],
        &["X", "Y"],
        &[vec![2.0, -1.0], vec![-3.0, 1.0]],
    );
    let fixture = load_matrix_iteration_fixture();

    let mut state = CfrState::new(&game);
    let report = state.run_iteration(&game);

    assert_eq!(report.iteration_index, 1);
    for (infoset_key, expected_row) in fixture.regrets {
        let actual_row = state.regret_row_by_key(&game, &infoset_key).unwrap();
        assert_slice_close(actual_row, &expected_row, 1e-12);
    }

    let average_profile = state.average_profile(&game);
    for (infoset_key, expected_row) in fixture.average_profile {
        let actual_row = average_profile.probability_by_key(&game, &infoset_key).unwrap();
        assert_slice_close(actual_row, &expected_row, 1e-12);
    }

    let next_current_profile = state.current_profile(&game);
    assert_slice_close(
        next_current_profile
            .probability_by_key(&game, "skewed_matrix:P0/root")
            .unwrap(),
        &[1.0, 0.0],
        1e-12,
    );
    assert_slice_close(
        next_current_profile
            .probability_by_key(&game, "skewed_matrix:P1/hidden")
            .unwrap(),
        &[1.0, 0.0],
        1e-12,
    );
}

#[test]
fn exact_best_response_detects_strictly_exploitable_uniform_kuhn_profile() {
    let game = kuhn::game();
    let uniform = StrategyProfile::uniform(&game);
    let value = expected_value_p0(&game, &uniform);
    let br_p0 = pure_best_response_value_p0(&game, &uniform, Player::P0);
    let br_p1 = pure_best_response_value_p0(&game, &uniform, Player::P1);

    assert_close(value, 0.125, 1e-12);
    assert!(br_p0 > value);
    assert!(br_p1 < value);

    let report = exact_exploitability(&game, &uniform);
    assert!(report.nash_conv > 0.0);
    assert_close(report.best_response_value_p0, br_p0, 1e-12);
    assert_close(report.worst_case_value_p0_against_p1_best_response, br_p1, 1e-12);
}

struct KuhnEquilibriumFixture {
    expected_value_p0: f64,
    max_nash_conv: f64,
    profile: BTreeMap<String, Vec<f64>>,
}

struct MatrixIterationFixture {
    regrets: BTreeMap<String, Vec<f64>>,
    average_profile: BTreeMap<String, Vec<f64>>,
}

fn load_manifest() -> Vec<(String, String)> {
    let text = fs::read_to_string(fixture_dir().join("manifest.txt")).expect("manifest should read");
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let mut parts = line.split_whitespace();
            let id = parts.next().expect("manifest id").to_string();
            let path = parts.next().expect("manifest path").to_string();
            assert!(parts.next().is_none(), "unexpected extra manifest columns in {line}");
            (id, path)
        })
        .collect()
}

fn load_kuhn_equilibrium_fixture() -> KuhnEquilibriumFixture {
    let text = fs::read_to_string(fixture_dir().join("kuhn_reference_equilibrium.fixture"))
        .expect("kuhn fixture should read");
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

    KuhnEquilibriumFixture {
        expected_value_p0: expected_value_p0.expect("expected_value_p0 missing"),
        max_nash_conv: max_nash_conv.expect("max_nash_conv missing"),
        profile,
    }
}

fn load_matrix_iteration_fixture() -> MatrixIterationFixture {
    let text = fs::read_to_string(fixture_dir().join("skewed_matrix_cfr_iteration_1.fixture"))
        .expect("matrix fixture should read");
    let mut current_section = None::<&str>;
    let mut regrets = BTreeMap::new();
    let mut average_profile = BTreeMap::new();

    for line in meaningful_lines(&text) {
        if let Some(section_name) = line.strip_prefix('[').and_then(|line| line.strip_suffix(']')) {
            current_section = Some(section_name);
            continue;
        }
        let (key, value) = split_key_value(line);
        match current_section {
            Some("regrets") => {
                regrets.insert(key.to_string(), parse_f64_list(value));
            }
            Some("average_profile") => {
                average_profile.insert(key.to_string(), parse_f64_list(value));
            }
            other => panic!("unexpected fixture section: {:?}", other),
        }
    }

    MatrixIterationFixture {
        regrets,
        average_profile,
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
    value.split(',').map(|part| parse_f64(part.trim())).collect()
}

fn parse_f64(value: &str) -> f64 {
    value.parse::<f64>()
        .unwrap_or_else(|_| panic!("expected f64, got {value}"))
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/conformance")
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tolerance,
        "expected {expected} ± {tolerance}, got {actual} (delta {delta})"
    );
}

fn assert_slice_close(actual: &[f64], expected: &[f64], tolerance: f64) {
    assert_eq!(actual.len(), expected.len(), "slice length mismatch");
    for (actual_value, expected_value) in actual.iter().zip(expected.iter()) {
        assert_close(*actual_value, *expected_value, tolerance);
    }
}
