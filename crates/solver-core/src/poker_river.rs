use std::{collections::BTreeMap, fmt};

use config_core::SolveConfig;
use equity_core::compare_hands;
use poker_core::HoleCards;
use range_core::WeightedRange;
use tree_core::build_river_tree;

use crate::{
    cfr::CfrState,
    game::{ActionEdge, ChanceOutcome, ExtensiveGame, Infoset, Node, NodeKind, Player},
    oracle::{exact_exploitability, ExploitabilityReport},
    profile::StrategyProfile,
};

#[derive(Debug, Clone)]
pub struct RiverSolveResult {
    pub game: ExtensiveGame,
    pub average_profile: StrategyProfile,
    pub exploitability: ExploitabilityReport,
}

pub fn build_river_game(config: &SolveConfig) -> Result<ExtensiveGame, RiverSolveError> {
    config.validate().map_err(RiverSolveError::Config)?;
    let public_tree = build_river_tree(config).map_err(RiverSolveError::Tree)?;
    let weighted_matchups = weighted_matchups(
        &config
            .ranges
            .oop_range
            .remove_board_blocked_combos(&config.root_state.board)
            .map_err(RiverSolveError::Range)?,
        &config
            .ranges
            .ip_range
            .remove_board_blocked_combos(&config.root_state.board)
            .map_err(RiverSolveError::Range)?,
    )?;

    let mut builder = RiverGameBuilder::new(config, public_tree.identity.clone());
    let root = builder.build_root(&weighted_matchups)?;
    let game = ExtensiveGame::new(
        format!("river_nlhe:{}", public_tree.identity),
        root,
        builder.nodes,
        builder.infosets,
    );
    game.validate().map_err(RiverSolveError::Game)?;
    Ok(game)
}

pub fn solve_river_cfr(config: &SolveConfig) -> Result<RiverSolveResult, RiverSolveError> {
    let game = build_river_game(config)?;
    let mut state = CfrState::new(&game);
    state.run_iterations(&game, config.solver_settings.iterations);
    let average_profile = state.average_profile(&game);
    average_profile.validate(&game).map_err(RiverSolveError::Profile)?;
    let exploitability = exact_exploitability(&game, &average_profile);
    Ok(RiverSolveResult {
        game,
        average_profile,
        exploitability,
    })
}

#[derive(Debug, Clone, Copy)]
struct WeightedMatchup {
    oop_hole_cards: HoleCards,
    ip_hole_cards: HoleCards,
    probability: f64,
}

struct RiverGameBuilder<'a> {
    config: &'a SolveConfig,
    tree_identity: String,
    nodes: Vec<Node>,
    infosets: Vec<Infoset>,
    infoset_ids: BTreeMap<String, usize>,
}

impl<'a> RiverGameBuilder<'a> {
    fn new(config: &'a SolveConfig, tree_identity: String) -> Self {
        Self {
            config,
            tree_identity,
            nodes: Vec::new(),
            infosets: Vec::new(),
            infoset_ids: BTreeMap::new(),
        }
    }

    fn build_root(&mut self, matchups: &[WeightedMatchup]) -> Result<usize, RiverSolveError> {
        let mut outcomes = Vec::with_capacity(matchups.len());
        for matchup in matchups {
            let child = self.build_oop_root(matchup.oop_hole_cards, matchup.ip_hole_cards)?;
            outcomes.push(ChanceOutcome {
                label: format!("{}|{}", matchup.oop_hole_cards, matchup.ip_hole_cards),
                probability: matchup.probability,
                child,
            });
        }
        Ok(self.push_node(Node {
            history: format!("deal:{}", self.tree_identity),
            kind: NodeKind::Chance { outcomes },
        }))
    }

    fn build_oop_root(
        &mut self,
        oop_hole_cards: HoleCards,
        ip_hole_cards: HoleCards,
    ) -> Result<usize, RiverSolveError> {
        let infoset = self.infoset_id(
            Player::P0,
            format!("river:P0:root:{oop_hole_cards}"),
            &["check", "bet"],
        );
        let check_terminal = self.check_showdown_terminal(oop_hole_cards, ip_hole_cards)?;
        let bet_response = self.build_ip_vs_bet(oop_hole_cards, ip_hole_cards)?;
        let node_id = self.push_node(Node {
            history: format!("{}|{}:root", oop_hole_cards, ip_hole_cards),
            kind: NodeKind::Decision {
                player: Player::P0,
                infoset,
                actions: vec![
                    ActionEdge {
                        label: "check".to_string(),
                        child: check_terminal,
                    },
                    ActionEdge {
                        label: "bet".to_string(),
                        child: bet_response,
                    },
                ],
            },
        });
        self.infosets[infoset].node_ids.push(node_id);
        Ok(node_id)
    }

    fn build_ip_vs_bet(
        &mut self,
        oop_hole_cards: HoleCards,
        ip_hole_cards: HoleCards,
    ) -> Result<usize, RiverSolveError> {
        let infoset = self.infoset_id(
            Player::P1,
            format!("river:P1:vs_bet:{ip_hole_cards}"),
            &["fold", "call"],
        );
        let fold_terminal = self.bet_fold_terminal();
        let call_terminal = self.bet_call_showdown_terminal(oop_hole_cards, ip_hole_cards)?;
        let node_id = self.push_node(Node {
            history: format!("{}|{}:vs_bet", oop_hole_cards, ip_hole_cards),
            kind: NodeKind::Decision {
                player: Player::P1,
                infoset,
                actions: vec![
                    ActionEdge {
                        label: "fold".to_string(),
                        child: fold_terminal,
                    },
                    ActionEdge {
                        label: "call".to_string(),
                        child: call_terminal,
                    },
                ],
            },
        });
        self.infosets[infoset].node_ids.push(node_id);
        Ok(node_id)
    }

    fn check_showdown_terminal(
        &mut self,
        oop_hole_cards: HoleCards,
        ip_hole_cards: HoleCards,
    ) -> Result<usize, RiverSolveError> {
        let utility_p0 = showdown_utility(
            self.config.root_state.pot_size as f64 / 2.0,
            compare_hands(&self.config.root_state.board, oop_hole_cards, ip_hole_cards)
                .map_err(RiverSolveError::Equity)?,
        );
        Ok(self.push_node(Node {
            history: format!("{}|{}:check_check", oop_hole_cards, ip_hole_cards),
            kind: NodeKind::Terminal { utility_p0 },
        }))
    }

    fn bet_fold_terminal(&mut self) -> usize {
        self.push_node(Node {
            history: "bet_fold".to_string(),
            kind: NodeKind::Terminal {
                utility_p0: self.config.root_state.pot_size as f64 / 2.0,
            },
        })
    }

    fn bet_call_showdown_terminal(
        &mut self,
        oop_hole_cards: HoleCards,
        ip_hole_cards: HoleCards,
    ) -> Result<usize, RiverSolveError> {
        let utility_p0 = showdown_utility(
            self.config.root_state.pot_size as f64 / 2.0
                + self.config.tree_template.bet_size_chips as f64,
            compare_hands(&self.config.root_state.board, oop_hole_cards, ip_hole_cards)
                .map_err(RiverSolveError::Equity)?,
        );
        Ok(self.push_node(Node {
            history: format!("{}|{}:bet_call", oop_hole_cards, ip_hole_cards),
            kind: NodeKind::Terminal { utility_p0 },
        }))
    }

    fn push_node(&mut self, node: Node) -> usize {
        let node_id = self.nodes.len();
        self.nodes.push(node);
        node_id
    }

    fn infoset_id(&mut self, player: Player, key: String, action_labels: &[&str]) -> usize {
        if let Some(existing) = self.infoset_ids.get(&key).copied() {
            return existing;
        }
        let infoset_id = self.infosets.len();
        self.infosets.push(Infoset {
            id: infoset_id,
            player,
            key: key.clone(),
            action_labels: action_labels.iter().map(|label| (*label).to_string()).collect(),
            node_ids: Vec::new(),
        });
        self.infoset_ids.insert(key, infoset_id);
        infoset_id
    }
}

fn weighted_matchups(
    oop_range: &WeightedRange,
    ip_range: &WeightedRange,
) -> Result<Vec<WeightedMatchup>, RiverSolveError> {
    let mut matchups = Vec::new();
    let mut total = 0.0;
    for oop_entry in oop_range.entries() {
        for ip_entry in ip_range.entries() {
            if oop_entry
                .hole_cards
                .intersects_hole_cards(ip_entry.hole_cards)
            {
                continue;
            }
            let probability = oop_entry.weight * ip_entry.weight;
            if probability > 0.0 {
                matchups.push(WeightedMatchup {
                    oop_hole_cards: oop_entry.hole_cards,
                    ip_hole_cards: ip_entry.hole_cards,
                    probability,
                });
                total += probability;
            }
        }
    }
    if total <= 0.0 {
        return Err(RiverSolveError::NoValidMatchups);
    }
    for matchup in &mut matchups {
        matchup.probability /= total;
    }
    Ok(matchups)
}

fn showdown_utility(magnitude: f64, ordering: std::cmp::Ordering) -> f64 {
    match ordering {
        std::cmp::Ordering::Greater => magnitude,
        std::cmp::Ordering::Less => -magnitude,
        std::cmp::Ordering::Equal => 0.0,
    }
}

#[derive(Debug)]
pub enum RiverSolveError {
    Config(config_core::ConfigError),
    Tree(tree_core::TreeError),
    Range(range_core::RangeError),
    Equity(equity_core::EquityError),
    Game(String),
    Profile(String),
    NoValidMatchups,
}

impl fmt::Display for RiverSolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiverSolveError::Config(error) => write!(f, "{error}"),
            RiverSolveError::Tree(error) => write!(f, "{error}"),
            RiverSolveError::Range(error) => write!(f, "{error}"),
            RiverSolveError::Equity(error) => write!(f, "{error}"),
            RiverSolveError::Game(error) => write!(f, "{error}"),
            RiverSolveError::Profile(error) => write!(f, "{error}"),
            RiverSolveError::NoValidMatchups => write!(f, "no valid range matchups remain after blocker filtering"),
        }
    }
}

impl std::error::Error for RiverSolveError {}
