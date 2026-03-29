use crate::{
    game::{ExtensiveGame, NodeKind, Player},
    profile::StrategyProfile,
};

#[derive(Debug, Clone)]
pub struct ExploitabilityReport {
    pub profile_value_p0: f64,
    pub best_response_value_p0: f64,
    pub worst_case_value_p0_against_p1_best_response: f64,
    pub p0_improvement: f64,
    pub p1_improvement: f64,
    pub nash_conv: f64,
}

pub fn expected_value_p0(game: &ExtensiveGame, profile: &StrategyProfile) -> f64 {
    expected_value_node_p0(game, game.root(), profile)
}

pub fn exact_exploitability(game: &ExtensiveGame, profile: &StrategyProfile) -> ExploitabilityReport {
    let profile_value_p0 = expected_value_p0(game, profile);
    let best_response_value_p0 = pure_best_response_value_p0(game, profile, Player::P0);
    let worst_case_value_p0_against_p1_best_response = pure_best_response_value_p0(game, profile, Player::P1);
    let p0_improvement = best_response_value_p0 - profile_value_p0;
    let p1_improvement = profile_value_p0 - worst_case_value_p0_against_p1_best_response;
    let nash_conv = p0_improvement + p1_improvement;

    ExploitabilityReport {
        profile_value_p0,
        best_response_value_p0,
        worst_case_value_p0_against_p1_best_response,
        p0_improvement,
        p1_improvement,
        nash_conv,
    }
}

pub fn pure_best_response_value_p0(
    game: &ExtensiveGame,
    profile: &StrategyProfile,
    responding_player: Player,
) -> f64 {
    let infosets = game.player_infosets(responding_player);
    let mut choices = vec![0usize; game.infosets().len()];

    enumerate_best_response(
        game,
        profile,
        responding_player,
        &infosets,
        0,
        &mut choices,
        None,
    )
    .expect("at least one pure strategy should exist")
}

fn enumerate_best_response(
    game: &ExtensiveGame,
    profile: &StrategyProfile,
    responding_player: Player,
    infosets: &[usize],
    depth: usize,
    choices: &mut [usize],
    current_best: Option<f64>,
) -> Option<f64> {
    if depth == infosets.len() {
        let value = expected_value_with_pure_response_p0(game, game.root(), profile, responding_player, choices);
        return Some(match (current_best, responding_player) {
            (None, _) => value,
            (Some(best), Player::P0) => best.max(value),
            (Some(best), Player::P1) => best.min(value),
        });
    }

    let infoset_id = infosets[depth];
    let action_count = game.infoset(infoset_id).action_labels.len();
    let mut best = current_best;
    for action_index in 0..action_count {
        choices[infoset_id] = action_index;
        best = enumerate_best_response(
            game,
            profile,
            responding_player,
            infosets,
            depth + 1,
            choices,
            best,
        );
    }
    best
}

fn expected_value_node_p0(game: &ExtensiveGame, node_id: usize, profile: &StrategyProfile) -> f64 {
    match &game.node(node_id).kind {
        NodeKind::Terminal { utility_p0 } => *utility_p0,
        NodeKind::Chance { outcomes } => outcomes
            .iter()
            .map(|outcome| outcome.probability * expected_value_node_p0(game, outcome.child, profile))
            .sum(),
        NodeKind::Decision {
            player: _,
            infoset,
            actions,
        } => profile
            .probabilities(*infoset)
            .iter()
            .zip(actions.iter())
            .map(|(probability, action)| probability * expected_value_node_p0(game, action.child, profile))
            .sum(),
    }
}

fn expected_value_with_pure_response_p0(
    game: &ExtensiveGame,
    node_id: usize,
    profile: &StrategyProfile,
    responding_player: Player,
    pure_choices: &[usize],
) -> f64 {
    match &game.node(node_id).kind {
        NodeKind::Terminal { utility_p0 } => *utility_p0,
        NodeKind::Chance { outcomes } => outcomes
            .iter()
            .map(|outcome| {
                outcome.probability
                    * expected_value_with_pure_response_p0(
                        game,
                        outcome.child,
                        profile,
                        responding_player,
                        pure_choices,
                    )
            })
            .sum(),
        NodeKind::Decision {
            player,
            infoset,
            actions,
        } => {
            if *player == responding_player {
                let action_index = pure_choices[*infoset];
                expected_value_with_pure_response_p0(
                    game,
                    actions[action_index].child,
                    profile,
                    responding_player,
                    pure_choices,
                )
            } else {
                profile
                    .probabilities(*infoset)
                    .iter()
                    .zip(actions.iter())
                    .map(|(probability, action)| {
                        probability
                            * expected_value_with_pure_response_p0(
                                game,
                                action.child,
                                profile,
                                responding_player,
                                pure_choices,
                            )
                    })
                    .sum()
            }
        }
    }
}
