use std::{collections::BTreeMap, fmt};

use config_core::{ConfigError, PlayerRole, SolveConfig};
use equity_core::compare_hands;
use poker_core::{Board, HoleCards};
use range_core::{RangeError, WeightedRange};
use tree_core::{
    build_river_tree, RiverTerminalOutcome, RiverTree, RiverTreeError, RiverTreeNodeKind,
};

use crate::{
    game::{ActionEdge, ChanceOutcome, ExtensiveGame, Infoset, Node, NodeKind, Player},
    oracle::ExploitabilityReport,
    profile::StrategyProfile,
    solve::solve_game,
};

#[derive(Debug, Clone)]
pub struct PokerSolveResult {
    pub game: ExtensiveGame,
    pub current_profile: StrategyProfile,
    pub average_profile: StrategyProfile,
    pub exploitability: ExploitabilityReport,
    pub iterations: usize,
    pub root_value_oop: f64,
    pub config_hash: String,
    pub tree_identity: String,
    pub strategy_by_infoset: BTreeMap<String, Vec<f64>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PokerSolveError {
    Config(String),
    Range(String),
    Tree(String),
    EmptyChanceSupport,
}

impl fmt::Display for PokerSolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(value) => write!(f, "config error: {value}"),
            Self::Range(value) => write!(f, "range error: {value}"),
            Self::Tree(value) => write!(f, "tree error: {value}"),
            Self::EmptyChanceSupport => write!(
                f,
                "no disjoint private-hand combinations remain after card removal"
            ),
        }
    }
}

impl std::error::Error for PokerSolveError {}

impl From<ConfigError> for PokerSolveError {
    fn from(value: ConfigError) -> Self {
        Self::Config(value.to_string())
    }
}

impl From<RangeError> for PokerSolveError {
    fn from(value: RangeError) -> Self {
        Self::Range(value.to_string())
    }
}

impl From<RiverTreeError> for PokerSolveError {
    fn from(value: RiverTreeError) -> Self {
        Self::Tree(value.to_string())
    }
}

pub fn build_river_game(config: &SolveConfig) -> Result<ExtensiveGame, PokerSolveError> {
    config.validate()?;
    let config_hash = config.canonical_hash()?;
    let tree = build_river_tree(config)?;
    let deals = enumerate_deals(&config.ranges.oop_range, &config.ranges.ip_range)?;
    build_river_game_from_parts(config, &config_hash, &tree, &deals)
}

pub fn solve_river_spot(config: &SolveConfig) -> Result<PokerSolveResult, PokerSolveError> {
    config.validate()?;
    let config_hash = config.canonical_hash()?;
    let tree = build_river_tree(config)?;
    let deals = enumerate_deals(&config.ranges.oop_range, &config.ranges.ip_range)?;
    let game = build_river_game_from_parts(config, &config_hash, &tree, &deals)?;
    let summary = solve_game(&game, config.solver_settings.iterations);
    let strategy_by_infoset = game
        .infosets()
        .iter()
        .map(|infoset| {
            (
                infoset.key.clone(),
                summary.average_profile.probabilities(infoset.id).to_vec(),
            )
        })
        .collect();

    Ok(PokerSolveResult {
        root_value_oop: summary.exploitability.profile_value_p0,
        config_hash,
        tree_identity: tree.tree_identity,
        game,
        current_profile: summary.current_profile,
        average_profile: summary.average_profile,
        exploitability: summary.exploitability,
        iterations: summary.iterations,
        strategy_by_infoset,
    })
}

#[derive(Debug, Clone)]
struct DealOutcome {
    oop: HoleCards,
    ip: HoleCards,
    probability: f64,
}

fn build_river_game_from_parts(
    config: &SolveConfig,
    config_hash: &str,
    tree: &RiverTree,
    deals: &[DealOutcome],
) -> Result<ExtensiveGame, PokerSolveError> {
    let mut registry = InfosetRegistry::default();
    let mut nodes = Vec::new();
    let root = build_chance_root(
        &mut nodes,
        &mut registry,
        tree,
        &config.root_state.board,
        deals,
    );
    let game = ExtensiveGame::new(
        format!("river_poker:{}:{}", config_hash, tree.tree_identity),
        root,
        nodes,
        registry.into_infosets(),
    );
    game.validate().map_err(PokerSolveError::Tree)?;
    Ok(game)
}

fn enumerate_deals(
    oop_range: &WeightedRange,
    ip_range: &WeightedRange,
) -> Result<Vec<DealOutcome>, PokerSolveError> {
    let mut deals = Vec::new();
    let mut total = 0.0;
    for oop in oop_range.combos() {
        for ip in ip_range.combos() {
            if !oop.hole_cards.conflicts_with(ip.hole_cards) {
                let probability = oop.weight * ip.weight;
                total += probability;
                deals.push(DealOutcome {
                    oop: oop.hole_cards,
                    ip: ip.hole_cards,
                    probability,
                });
            }
        }
    }
    if total <= 0.0 {
        return Err(PokerSolveError::EmptyChanceSupport);
    }
    for deal in &mut deals {
        deal.probability /= total;
    }
    Ok(deals)
}

fn build_chance_root(
    nodes: &mut Vec<Node>,
    registry: &mut InfosetRegistry,
    tree: &RiverTree,
    board: &Board,
    deals: &[DealOutcome],
) -> usize {
    let mut outcomes = Vec::new();
    for deal in deals {
        let child = build_public_subtree(
            nodes,
            registry,
            tree,
            board,
            tree.root_node_id,
            deal.oop,
            deal.ip,
        );
        outcomes.push(ChanceOutcome {
            label: format!("deal:{}|{}", deal.oop, deal.ip),
            probability: deal.probability,
            child,
        });
    }
    let node_id = nodes.len();
    nodes.push(Node {
        history: "chance/deal".to_string(),
        kind: NodeKind::Chance { outcomes },
    });
    node_id
}

fn build_public_subtree(
    nodes: &mut Vec<Node>,
    registry: &mut InfosetRegistry,
    tree: &RiverTree,
    board: &Board,
    public_node_id: usize,
    oop: HoleCards,
    ip: HoleCards,
) -> usize {
    let public_node = &tree.nodes[public_node_id];
    match &public_node.kind {
        RiverTreeNodeKind::Decision { player, actions } => {
            let children = actions
                .iter()
                .map(|action| {
                    let child = build_public_subtree(
                        nodes,
                        registry,
                        tree,
                        board,
                        action.next_node_id,
                        oop,
                        ip,
                    );
                    ActionEdge {
                        label: action.label.clone(),
                        child,
                    }
                })
                .collect::<Vec<_>>();
            let node_id = nodes.len();
            let private_hand = match player {
                PlayerRole::Oop => oop,
                PlayerRole::Ip => ip,
            };
            let infoset_key = format!(
                "river:{}:{}:{}",
                actor_key(*player),
                private_hand,
                public_node.history_key
            );
            let action_labels = children
                .iter()
                .map(|edge| edge.label.clone())
                .collect::<Vec<_>>();
            let infoset_id = registry.register(
                to_extensive_player(*player),
                infoset_key,
                action_labels,
                node_id,
            );
            nodes.push(Node {
                history: format!("{}|{}|{}", public_node.history_key, oop, ip),
                kind: NodeKind::Decision {
                    player: to_extensive_player(*player),
                    infoset: infoset_id,
                    actions: children,
                },
            });
            node_id
        }
        RiverTreeNodeKind::Terminal { outcome } => {
            let utility_p0 = terminal_utility(*outcome, board, oop, ip);
            let node_id = nodes.len();
            nodes.push(Node {
                history: format!("terminal:{}|{}|{}", public_node.history_key, oop, ip),
                kind: NodeKind::Terminal { utility_p0 },
            });
            node_id
        }
    }
}

fn terminal_utility(
    outcome: RiverTerminalOutcome,
    board: &Board,
    oop: HoleCards,
    ip: HoleCards,
) -> f64 {
    match outcome {
        RiverTerminalOutcome::Fold {
            winner,
            utility_magnitude,
        } => match winner {
            PlayerRole::Oop => utility_magnitude as f64,
            PlayerRole::Ip => -(utility_magnitude as f64),
        },
        RiverTerminalOutcome::Showdown { utility_magnitude } => match compare_hands(board, oop, ip)
        {
            std::cmp::Ordering::Greater => utility_magnitude as f64,
            std::cmp::Ordering::Less => -(utility_magnitude as f64),
            std::cmp::Ordering::Equal => 0.0,
        },
    }
}

fn to_extensive_player(player: PlayerRole) -> Player {
    match player {
        PlayerRole::Oop => Player::P0,
        PlayerRole::Ip => Player::P1,
    }
}

fn actor_key(player: PlayerRole) -> &'static str {
    match player {
        PlayerRole::Oop => "oop",
        PlayerRole::Ip => "ip",
    }
}

#[derive(Default)]
struct InfosetRegistry {
    infosets: Vec<Infoset>,
    index_by_key: BTreeMap<String, usize>,
}

impl InfosetRegistry {
    fn register(
        &mut self,
        player: Player,
        key: String,
        action_labels: Vec<String>,
        node_id: usize,
    ) -> usize {
        if let Some(existing) = self.index_by_key.get(&key).copied() {
            self.infosets[existing].node_ids.push(node_id);
            return existing;
        }
        let infoset_id = self.infosets.len();
        self.index_by_key.insert(key.clone(), infoset_id);
        self.infosets.push(Infoset {
            id: infoset_id,
            player,
            key,
            action_labels,
            node_ids: vec![node_id],
        });
        infoset_id
    }

    fn into_infosets(self) -> Vec<Infoset> {
        self.infosets
    }
}
