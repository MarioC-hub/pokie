use std::fmt;

use config_core::{ConfigError, PlayerRole, SolveConfig};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiverTree {
    pub root_node_id: usize,
    pub nodes: Vec<RiverTreeNode>,
    pub tree_identity: String,
}

impl RiverTree {
    pub fn validate(&self) -> Result<(), RiverTreeError> {
        if self.nodes.is_empty() {
            return Err(RiverTreeError::new("tree must contain at least one node"));
        }
        if self.root_node_id >= self.nodes.len() {
            return Err(RiverTreeError::new("root node id is out of bounds"));
        }
        for (index, node) in self.nodes.iter().enumerate() {
            if node.node_id != index {
                return Err(RiverTreeError::new(format!(
                    "node index {index} does not match node id {}",
                    node.node_id
                )));
            }
            match &node.kind {
                RiverTreeNodeKind::Decision { player, actions } => {
                    if *player != node.player_to_act {
                        return Err(RiverTreeError::new(format!(
                            "node {} player_to_act mismatch",
                            node.history_key
                        )));
                    }
                    if actions.is_empty() {
                        return Err(RiverTreeError::new(format!(
                            "node {} has no actions",
                            node.history_key
                        )));
                    }
                    for action in actions {
                        if action.next_node_id >= self.nodes.len() {
                            return Err(RiverTreeError::new(format!(
                                "node {} references missing child {}",
                                node.history_key, action.next_node_id
                            )));
                        }
                    }
                }
                RiverTreeNodeKind::Terminal { .. } => {}
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiverTreeNode {
    pub node_id: usize,
    pub history_key: String,
    pub player_to_act: PlayerRole,
    pub pot_size: u32,
    pub kind: RiverTreeNodeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiverTreeNodeKind {
    Decision {
        player: PlayerRole,
        actions: Vec<RiverTreeAction>,
    },
    Terminal {
        outcome: RiverTerminalOutcome,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiverTreeAction {
    pub label: String,
    pub chips: u32,
    pub next_node_id: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiverTerminalOutcome {
    Fold {
        winner: PlayerRole,
        utility_magnitude: u32,
    },
    Showdown {
        utility_magnitude: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiverTreeError {
    message: String,
}

impl RiverTreeError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for RiverTreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for RiverTreeError {}

pub fn build_river_tree(config: &SolveConfig) -> Result<RiverTree, RiverTreeError> {
    config.validate().map_err(config_error)?;
    let config_hash = config.canonical_hash().map_err(config_error)?;

    let mut nodes = Vec::new();
    let root_node_id = push_decision_node(
        &mut nodes,
        "root",
        config.root_state.player_to_act,
        config.root_state.pot_size,
    );

    let first_bet_size = config.tree_template.first_bet_size;
    let after_check_bet_size = config.tree_template.after_check_bet_size;
    let initial_pot = config.root_state.pot_size;

    let mut root_actions = Vec::new();

    let check_node_id = if after_check_bet_size > 0 {
        let check_node_id = push_decision_node(
            &mut nodes,
            "x",
            config.root_state.player_to_act.opponent(),
            initial_pot,
        );
        let check_check_terminal = push_terminal_node(
            &mut nodes,
            "x-x",
            config.root_state.player_to_act,
            initial_pot,
            RiverTerminalOutcome::Showdown {
                utility_magnitude: initial_pot / 2,
            },
        );
        let check_bet_node = push_decision_node(
            &mut nodes,
            "x-b",
            config.root_state.player_to_act,
            initial_pot + after_check_bet_size,
        );
        let check_bet_fold_terminal = push_terminal_node(
            &mut nodes,
            "x-b-f",
            config.root_state.player_to_act,
            initial_pot + after_check_bet_size,
            RiverTerminalOutcome::Fold {
                winner: config.root_state.player_to_act.opponent(),
                utility_magnitude: initial_pot / 2,
            },
        );
        let check_bet_call_terminal = push_terminal_node(
            &mut nodes,
            "x-b-c",
            config.root_state.player_to_act,
            initial_pot + 2 * after_check_bet_size,
            RiverTerminalOutcome::Showdown {
                utility_magnitude: (initial_pot / 2) + after_check_bet_size,
            },
        );
        set_actions(
            &mut nodes,
            check_bet_node,
            vec![
                RiverTreeAction {
                    label: "fold".to_string(),
                    chips: 0,
                    next_node_id: check_bet_fold_terminal,
                },
                RiverTreeAction {
                    label: "call".to_string(),
                    chips: after_check_bet_size,
                    next_node_id: check_bet_call_terminal,
                },
            ],
        )?;
        set_actions(
            &mut nodes,
            check_node_id,
            vec![
                RiverTreeAction {
                    label: "check".to_string(),
                    chips: 0,
                    next_node_id: check_check_terminal,
                },
                RiverTreeAction {
                    label: format!("bet_{after_check_bet_size}"),
                    chips: after_check_bet_size,
                    next_node_id: check_bet_node,
                },
            ],
        )?;
        Some(check_node_id)
    } else {
        let check_terminal = push_terminal_node(
            &mut nodes,
            "x",
            config.root_state.player_to_act.opponent(),
            initial_pot,
            RiverTerminalOutcome::Showdown {
                utility_magnitude: initial_pot / 2,
            },
        );
        Some(check_terminal)
    };

    if let Some(check_node_id) = check_node_id {
        root_actions.push(RiverTreeAction {
            label: "check".to_string(),
            chips: 0,
            next_node_id: check_node_id,
        });
    }

    if first_bet_size > 0 {
        let bet_node_id = push_decision_node(
            &mut nodes,
            "b",
            config.root_state.player_to_act.opponent(),
            initial_pot + first_bet_size,
        );
        let bet_fold_terminal = push_terminal_node(
            &mut nodes,
            "b-f",
            config.root_state.player_to_act,
            initial_pot + first_bet_size,
            RiverTerminalOutcome::Fold {
                winner: config.root_state.player_to_act,
                utility_magnitude: initial_pot / 2,
            },
        );
        let bet_call_terminal = push_terminal_node(
            &mut nodes,
            "b-c",
            config.root_state.player_to_act,
            initial_pot + 2 * first_bet_size,
            RiverTerminalOutcome::Showdown {
                utility_magnitude: (initial_pot / 2) + first_bet_size,
            },
        );
        set_actions(
            &mut nodes,
            bet_node_id,
            vec![
                RiverTreeAction {
                    label: "fold".to_string(),
                    chips: 0,
                    next_node_id: bet_fold_terminal,
                },
                RiverTreeAction {
                    label: "call".to_string(),
                    chips: first_bet_size,
                    next_node_id: bet_call_terminal,
                },
            ],
        )?;
        root_actions.push(RiverTreeAction {
            label: format!("bet_{first_bet_size}"),
            chips: first_bet_size,
            next_node_id: bet_node_id,
        });
    }

    set_actions(&mut nodes, root_node_id, root_actions)?;

    let tree = RiverTree {
        root_node_id,
        nodes,
        tree_identity: format!(
            "{}:{}:{}:{}",
            config.tree_template.template_id,
            config.tree_template.template_version,
            config_hash,
            initial_pot
        ),
    };
    tree.validate()?;
    Ok(tree)
}

fn push_decision_node(
    nodes: &mut Vec<RiverTreeNode>,
    history_key: &str,
    player_to_act: PlayerRole,
    pot_size: u32,
) -> usize {
    let node_id = nodes.len();
    nodes.push(RiverTreeNode {
        node_id,
        history_key: history_key.to_string(),
        player_to_act,
        pot_size,
        kind: RiverTreeNodeKind::Decision {
            player: player_to_act,
            actions: Vec::new(),
        },
    });
    node_id
}

fn push_terminal_node(
    nodes: &mut Vec<RiverTreeNode>,
    history_key: &str,
    player_to_act: PlayerRole,
    pot_size: u32,
    outcome: RiverTerminalOutcome,
) -> usize {
    let node_id = nodes.len();
    nodes.push(RiverTreeNode {
        node_id,
        history_key: history_key.to_string(),
        player_to_act,
        pot_size,
        kind: RiverTreeNodeKind::Terminal { outcome },
    });
    node_id
}

fn set_actions(
    nodes: &mut [RiverTreeNode],
    node_id: usize,
    actions: Vec<RiverTreeAction>,
) -> Result<(), RiverTreeError> {
    match &mut nodes[node_id].kind {
        RiverTreeNodeKind::Decision { actions: existing, .. } => {
            *existing = actions;
            Ok(())
        }
        RiverTreeNodeKind::Terminal { .. } => Err(RiverTreeError::new(format!(
            "node {} is terminal and cannot receive actions",
            node_id
        ))),
    }
}

fn config_error(error: ConfigError) -> RiverTreeError {
    RiverTreeError::new(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use config_core::{
        AlgorithmFamily, BlindProfile, GameConfig, GameMode, NodeLock, PlayerRole, RakeProfile,
        RangeConfig, RiverTreeTemplate, RootState, SolveConfig, SolverSettings, Variant,
        RIVER_SINGLE_BET_TEMPLATE_ID,
    };
    use poker_core::{Board, Street};
    use range_core::WeightedRange;

    use super::*;

    fn config(first_bet_size: u32, after_check_bet_size: u32) -> SolveConfig {
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
                rake_profile: RakeProfile { rake_bps: 0, cap: 0 },
            },
            ranges: RangeConfig {
                oop_range: WeightedRange::from_hands(vec![
                    ("7c7h".parse().unwrap(), 1.0),
                    ("AcJc".parse().unwrap(), 1.0),
                ])
                .unwrap(),
                ip_range: WeightedRange::from_hands(vec![("KcQh".parse().unwrap(), 1.0)]).unwrap(),
            },
            tree_template: RiverTreeTemplate {
                template_id: RIVER_SINGLE_BET_TEMPLATE_ID.to_string(),
                template_version: 1,
                first_bet_size,
                after_check_bet_size,
                max_raises_per_street: 0,
                allow_all_in: false,
            },
            node_locks: Vec::<NodeLock>::new(),
            solver_settings: SolverSettings {
                algorithm: AlgorithmFamily::Cfr,
                iterations: 1,
                checkpoint_cadence: 0,
                thread_count: 1,
                deterministic_seed: 0,
            },
        }
    }

    #[test]
    fn builds_expected_root_shape() {
        let tree = build_river_tree(&config(10, 0)).unwrap();
        assert_eq!(tree.root_node_id, 0);
        assert_eq!(tree.nodes.len(), 5);
        match &tree.nodes[0].kind {
            RiverTreeNodeKind::Decision { actions, .. } => {
                assert_eq!(actions.len(), 2);
                assert_eq!(actions[0].label, "check");
                assert_eq!(actions[1].label, "bet_10");
            }
            other => panic!("unexpected root kind: {other:?}"),
        }
    }

    #[test]
    fn terminal_utilities_match_exact_contract_for_check_and_bet_lines() {
        let tree = build_river_tree(&config(10, 6)).unwrap();
        assert_eq!(tree.nodes.len(), 9);

        let terminals = tree
            .nodes
            .iter()
            .filter_map(|node| match node.kind {
                RiverTreeNodeKind::Terminal { outcome } => Some((node.history_key.as_str(), outcome)),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert!(terminals.contains(&(
            "x-x",
            RiverTerminalOutcome::Showdown {
                utility_magnitude: 5,
            }
        )));
        assert!(terminals.contains(&(
            "x-b-f",
            RiverTerminalOutcome::Fold {
                winner: PlayerRole::Ip,
                utility_magnitude: 5,
            }
        )));
        assert!(terminals.contains(&(
            "x-b-c",
            RiverTerminalOutcome::Showdown {
                utility_magnitude: 11,
            }
        )));
        assert!(terminals.contains(&(
            "b-f",
            RiverTerminalOutcome::Fold {
                winner: PlayerRole::Oop,
                utility_magnitude: 5,
            }
        )));
        assert!(terminals.contains(&(
            "b-c",
            RiverTerminalOutcome::Showdown {
                utility_magnitude: 15,
            }
        )));
    }
}
