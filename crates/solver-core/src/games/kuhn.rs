use std::collections::BTreeMap;

use crate::{
    game::{ActionEdge, ChanceOutcome, ExtensiveGame, Infoset, Node, NodeKind, Player},
    profile::StrategyProfile,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Card {
    J,
    Q,
    K,
}

impl Card {
    fn all() -> [Card; 3] {
        [Card::J, Card::Q, Card::K]
    }

    fn label(self) -> &'static str {
        match self {
            Card::J => "J",
            Card::Q => "Q",
            Card::K => "K",
        }
    }

    fn rank(self) -> u8 {
        match self {
            Card::J => 0,
            Card::Q => 1,
            Card::K => 2,
        }
    }
}

pub fn game() -> ExtensiveGame {
    let mut builder = KuhnBuilder::new();
    let root = builder.build_root();
    let game = ExtensiveGame::new("kuhn_poker", root, builder.nodes, builder.infosets);
    game.validate().expect("kuhn game should be valid");
    game
}

pub fn reference_equilibrium(game: &ExtensiveGame) -> StrategyProfile {
    let mut probabilities = BTreeMap::new();

    probabilities.insert("kuhn:P0:init:J".to_string(), vec![2.0 / 3.0, 1.0 / 3.0]);
    probabilities.insert("kuhn:P0:init:Q".to_string(), vec![1.0, 0.0]);
    probabilities.insert("kuhn:P0:init:K".to_string(), vec![0.0, 1.0]);

    probabilities.insert("kuhn:P1:after_check:J".to_string(), vec![2.0 / 3.0, 1.0 / 3.0]);
    probabilities.insert("kuhn:P1:after_check:Q".to_string(), vec![1.0, 0.0]);
    probabilities.insert("kuhn:P1:after_check:K".to_string(), vec![0.0, 1.0]);

    probabilities.insert("kuhn:P1:vs_bet:J".to_string(), vec![1.0, 0.0]);
    probabilities.insert("kuhn:P1:vs_bet:Q".to_string(), vec![2.0 / 3.0, 1.0 / 3.0]);
    probabilities.insert("kuhn:P1:vs_bet:K".to_string(), vec![0.0, 1.0]);

    probabilities.insert("kuhn:P0:after_check_bet:J".to_string(), vec![1.0, 0.0]);
    probabilities.insert("kuhn:P0:after_check_bet:Q".to_string(), vec![1.0 / 3.0, 2.0 / 3.0]);
    probabilities.insert("kuhn:P0:after_check_bet:K".to_string(), vec![0.0, 1.0]);

    StrategyProfile::from_named_probabilities(game, &probabilities)
        .expect("reference equilibrium must fit the Kuhn game")
}

struct KuhnBuilder {
    nodes: Vec<Node>,
    infosets: Vec<Infoset>,
}

impl KuhnBuilder {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            infosets: Vec::new(),
        }
    }

    fn build_root(&mut self) -> usize {
        let mut outcomes = Vec::new();
        let cards = Card::all();
        for &card_p0 in &cards {
            for &card_p1 in &cards {
                if card_p0 == card_p1 {
                    continue;
                }
                let child = self.build_p0_initial(card_p0, card_p1);
                outcomes.push(ChanceOutcome {
                    label: format!("deal:{}{}", card_p0.label(), card_p1.label()),
                    probability: 1.0 / 6.0,
                    child,
                });
            }
        }
        self.push_node(Node {
            history: "deal".to_string(),
            kind: NodeKind::Chance { outcomes },
        })
    }

    fn build_p0_initial(&mut self, card_p0: Card, card_p1: Card) -> usize {
        let infoset = self.infoset_id(Player::P0, format!("kuhn:P0:init:{}", card_p0.label()), &["check", "bet"]);
        let check_child = self.build_p1_after_check(card_p0, card_p1);
        let bet_child = self.build_p1_vs_bet(card_p0, card_p1);
        let node_id = self.push_node(Node {
            history: format!("deal:{}{} -> P0", card_p0.label(), card_p1.label()),
            kind: NodeKind::Decision {
                player: Player::P0,
                infoset,
                actions: vec![
                    ActionEdge {
                        label: "check".to_string(),
                        child: check_child,
                    },
                    ActionEdge {
                        label: "bet".to_string(),
                        child: bet_child,
                    },
                ],
            },
        });
        self.infosets[infoset].node_ids.push(node_id);
        node_id
    }

    fn build_p1_after_check(&mut self, card_p0: Card, card_p1: Card) -> usize {
        let infoset = self.infoset_id(
            Player::P1,
            format!("kuhn:P1:after_check:{}", card_p1.label()),
            &["check", "bet"],
        );
        let showdown = self.showdown_terminal(card_p0, card_p1, 1.0, "check-check");
        let bet_child = self.build_p0_after_check_bet(card_p0, card_p1);
        let node_id = self.push_node(Node {
            history: format!("deal:{}{} -> check -> P1", card_p0.label(), card_p1.label()),
            kind: NodeKind::Decision {
                player: Player::P1,
                infoset,
                actions: vec![
                    ActionEdge {
                        label: "check".to_string(),
                        child: showdown,
                    },
                    ActionEdge {
                        label: "bet".to_string(),
                        child: bet_child,
                    },
                ],
            },
        });
        self.infosets[infoset].node_ids.push(node_id);
        node_id
    }

    fn build_p1_vs_bet(&mut self, card_p0: Card, card_p1: Card) -> usize {
        let infoset = self.infoset_id(
            Player::P1,
            format!("kuhn:P1:vs_bet:{}", card_p1.label()),
            &["fold", "call"],
        );
        let fold_terminal = self.terminal(1.0, "bet-fold");
        let call_terminal = self.showdown_terminal(card_p0, card_p1, 2.0, "bet-call");
        let node_id = self.push_node(Node {
            history: format!("deal:{}{} -> bet -> P1", card_p0.label(), card_p1.label()),
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
        node_id
    }

    fn build_p0_after_check_bet(&mut self, card_p0: Card, card_p1: Card) -> usize {
        let infoset = self.infoset_id(
            Player::P0,
            format!("kuhn:P0:after_check_bet:{}", card_p0.label()),
            &["fold", "call"],
        );
        let fold_terminal = self.terminal(-1.0, "check-bet-fold");
        let call_terminal = self.showdown_terminal(card_p0, card_p1, 2.0, "check-bet-call");
        let node_id = self.push_node(Node {
            history: format!("deal:{}{} -> check-bet -> P0", card_p0.label(), card_p1.label()),
            kind: NodeKind::Decision {
                player: Player::P0,
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
        node_id
    }

    fn showdown_terminal(&mut self, card_p0: Card, card_p1: Card, stakes: f64, suffix: &str) -> usize {
        let utility_p0 = if card_p0.rank() > card_p1.rank() { stakes } else { -stakes };
        self.terminal(utility_p0, suffix)
    }

    fn terminal(&mut self, utility_p0: f64, suffix: &str) -> usize {
        self.push_node(Node {
            history: suffix.to_string(),
            kind: NodeKind::Terminal { utility_p0 },
        })
    }

    fn push_node(&mut self, node: Node) -> usize {
        let node_id = self.nodes.len();
        self.nodes.push(node);
        node_id
    }

    fn infoset_id(&mut self, player: Player, key: String, action_labels: &[&str]) -> usize {
        if let Some(existing) = self.infosets.iter().find(|infoset| infoset.key == key) {
            return existing.id;
        }
        let infoset_id = self.infosets.len();
        self.infosets.push(Infoset {
            id: infoset_id,
            player,
            key,
            action_labels: action_labels.iter().map(|label| (*label).to_string()).collect(),
            node_ids: Vec::new(),
        });
        infoset_id
    }
}
