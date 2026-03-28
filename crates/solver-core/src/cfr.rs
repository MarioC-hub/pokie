use crate::{
    game::{ExtensiveGame, NodeKind, Player},
    profile::StrategyProfile,
};

#[derive(Debug, Clone)]
pub struct IterationReport {
    pub iteration_index: usize,
    pub profile_used: StrategyProfile,
}

#[derive(Debug, Clone)]
pub struct CfrState {
    regrets: Vec<Vec<f64>>,
    average_numerators: Vec<Vec<f64>>,
    average_denominators: Vec<f64>,
    iterations_completed: usize,
}

impl CfrState {
    pub fn new(game: &ExtensiveGame) -> Self {
        let regrets = game
            .infosets()
            .iter()
            .map(|infoset| vec![0.0; infoset.action_labels.len()])
            .collect();
        let average_numerators = game
            .infosets()
            .iter()
            .map(|infoset| vec![0.0; infoset.action_labels.len()])
            .collect();
        let average_denominators = vec![0.0; game.infosets().len()];
        Self {
            regrets,
            average_numerators,
            average_denominators,
            iterations_completed: 0,
        }
    }

    pub fn iterations_completed(&self) -> usize {
        self.iterations_completed
    }

    pub fn regrets(&self) -> &[Vec<f64>] {
        &self.regrets
    }

    pub fn current_profile(&self, game: &ExtensiveGame) -> StrategyProfile {
        let infoset_strategies = game
            .infosets()
            .iter()
            .enumerate()
            .map(|(infoset_id, infoset)| regret_matching(&self.regrets[infoset_id], infoset.action_labels.len()))
            .collect();
        StrategyProfile::new(infoset_strategies)
    }

    pub fn average_profile(&self, game: &ExtensiveGame) -> StrategyProfile {
        let infoset_strategies = game
            .infosets()
            .iter()
            .enumerate()
            .map(|(infoset_id, infoset)| {
                let denominator = self.average_denominators[infoset_id];
                if denominator <= 0.0 {
                    let probability = 1.0 / infoset.action_labels.len() as f64;
                    vec![probability; infoset.action_labels.len()]
                } else {
                    normalize_probabilities(
                        self.average_numerators[infoset_id]
                        .iter()
                        .map(|value| value / denominator)
                        .collect(),
                    )
                }
            })
            .collect();
        StrategyProfile::new(infoset_strategies)
    }

    pub fn regret_row_by_key<'a>(&'a self, game: &'a ExtensiveGame, key: &str) -> Option<&'a [f64]> {
        let infoset_id = game.infoset_id(key)?;
        Some(&self.regrets[infoset_id])
    }

    pub fn run_iteration(&mut self, game: &ExtensiveGame) -> IterationReport {
        let profile_used = self.current_profile(game);
        self.accumulate_average_profile(game, &profile_used);

        let mut deltas_p0 = zero_table_like(game);
        let mut deltas_p1 = zero_table_like(game);

        cfr_traverse(
            game,
            game.root(),
            &profile_used,
            1.0,
            1.0,
            1.0,
            Player::P0,
            &mut deltas_p0,
        );
        cfr_traverse(
            game,
            game.root(),
            &profile_used,
            1.0,
            1.0,
            1.0,
            Player::P1,
            &mut deltas_p1,
        );

        for infoset in game.player_infosets(Player::P0) {
            for action_index in 0..self.regrets[infoset].len() {
                self.regrets[infoset][action_index] += deltas_p0[infoset][action_index];
            }
        }
        for infoset in game.player_infosets(Player::P1) {
            for action_index in 0..self.regrets[infoset].len() {
                self.regrets[infoset][action_index] += deltas_p1[infoset][action_index];
            }
        }

        self.iterations_completed += 1;
        IterationReport {
            iteration_index: self.iterations_completed,
            profile_used,
        }
    }

    pub fn run_iterations(&mut self, game: &ExtensiveGame, iterations: usize) {
        for _ in 0..iterations {
            self.run_iteration(game);
        }
    }

    fn accumulate_average_profile(&mut self, game: &ExtensiveGame, profile: &StrategyProfile) {
        accumulate_average_profile_node(
            self,
            game,
            game.root(),
            profile,
            1.0,
        );
    }
}

fn zero_table_like(game: &ExtensiveGame) -> Vec<Vec<f64>> {
    game.infosets()
        .iter()
        .map(|infoset| vec![0.0; infoset.action_labels.len()])
        .collect()
}

fn regret_matching(regrets: &[f64], action_count: usize) -> Vec<f64> {
    let positive_sum: f64 = regrets.iter().copied().filter(|value| *value > 0.0).sum();
    if positive_sum > 0.0 {
        normalize_probabilities(
            regrets
            .iter()
            .map(|value| if *value > 0.0 { *value / positive_sum } else { 0.0 })
            .collect(),
        )
    } else {
        let probability = 1.0 / action_count as f64;
        vec![probability; action_count]
    }
}

fn normalize_probabilities(mut values: Vec<f64>) -> Vec<f64> {
    let sum: f64 = values.iter().sum();
    if sum <= 0.0 {
        return values;
    }
    for value in &mut values {
        *value /= sum;
    }
    values
}

fn accumulate_average_profile_node(
    state: &mut CfrState,
    game: &ExtensiveGame,
    node_id: usize,
    profile: &StrategyProfile,
    reach_probability: f64,
) {
    match &game.node(node_id).kind {
        NodeKind::Terminal { .. } => {}
        NodeKind::Chance { outcomes } => {
            for outcome in outcomes {
                accumulate_average_profile_node(
                    state,
                    game,
                    outcome.child,
                    profile,
                    reach_probability * outcome.probability,
                );
            }
        }
        NodeKind::Decision {
            player: _,
            infoset,
            actions,
        } => {
            let strategy = profile.probabilities(*infoset);
            state.average_denominators[*infoset] += reach_probability;
            for (action_index, action) in actions.iter().enumerate() {
                state.average_numerators[*infoset][action_index] += reach_probability * strategy[action_index];
                accumulate_average_profile_node(
                    state,
                    game,
                    action.child,
                    profile,
                    reach_probability * strategy[action_index],
                );
            }
        }
    }
}

fn cfr_traverse(
    game: &ExtensiveGame,
    node_id: usize,
    profile: &StrategyProfile,
    reach_p0: f64,
    reach_p1: f64,
    chance_reach: f64,
    updating_player: Player,
    regret_deltas: &mut [Vec<f64>],
) -> f64 {
    match &game.node(node_id).kind {
        NodeKind::Terminal { utility_p0 } => match updating_player {
            Player::P0 => *utility_p0,
            Player::P1 => -*utility_p0,
        },
        NodeKind::Chance { outcomes } => outcomes
            .iter()
            .map(|outcome| {
                outcome.probability
                    * cfr_traverse(
                        game,
                        outcome.child,
                        profile,
                        reach_p0,
                        reach_p1,
                        chance_reach * outcome.probability,
                        updating_player,
                        regret_deltas,
                    )
            })
            .sum(),
        NodeKind::Decision {
            player,
            infoset,
            actions,
        } => {
            let strategy = profile.probabilities(*infoset);
            if *player == updating_player {
                let mut action_values = vec![0.0; actions.len()];
                let mut node_value = 0.0;
                for (action_index, action) in actions.iter().enumerate() {
                    let next_reach_p0 = if *player == Player::P0 {
                        reach_p0 * strategy[action_index]
                    } else {
                        reach_p0
                    };
                    let next_reach_p1 = if *player == Player::P1 {
                        reach_p1 * strategy[action_index]
                    } else {
                        reach_p1
                    };
                    let action_value = cfr_traverse(
                        game,
                        action.child,
                        profile,
                        next_reach_p0,
                        next_reach_p1,
                        chance_reach,
                        updating_player,
                        regret_deltas,
                    );
                    action_values[action_index] = action_value;
                    node_value += strategy[action_index] * action_value;
                }
                let counterfactual_reach = match updating_player {
                    Player::P0 => reach_p1 * chance_reach,
                    Player::P1 => reach_p0 * chance_reach,
                };
                for (action_index, action_value) in action_values.into_iter().enumerate() {
                    regret_deltas[*infoset][action_index] +=
                        counterfactual_reach * (action_value - node_value);
                }
                node_value
            } else {
                actions
                    .iter()
                    .enumerate()
                    .map(|(action_index, action)| {
                        let next_reach_p0 = if *player == Player::P0 {
                            reach_p0 * strategy[action_index]
                        } else {
                            reach_p0
                        };
                        let next_reach_p1 = if *player == Player::P1 {
                            reach_p1 * strategy[action_index]
                        } else {
                            reach_p1
                        };
                        strategy[action_index]
                            * cfr_traverse(
                                game,
                                action.child,
                                profile,
                                next_reach_p0,
                                next_reach_p1,
                                chance_reach,
                                updating_player,
                                regret_deltas,
                            )
                    })
                    .sum()
            }
        }
    }
}
