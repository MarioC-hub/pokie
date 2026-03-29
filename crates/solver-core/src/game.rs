use std::fmt;

pub type NodeId = usize;
pub type InfosetId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    P0,
    P1,
}

impl Player {
    pub const ALL: [Player; 2] = [Player::P0, Player::P1];

    pub fn index(self) -> usize {
        match self {
            Player::P0 => 0,
            Player::P1 => 1,
        }
    }

    pub fn opponent(self) -> Self {
        match self {
            Player::P0 => Player::P1,
            Player::P1 => Player::P0,
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Player::P0 => write!(f, "P0"),
            Player::P1 => write!(f, "P1"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActionEdge {
    pub label: String,
    pub child: NodeId,
}

#[derive(Debug, Clone)]
pub struct ChanceOutcome {
    pub label: String,
    pub probability: f64,
    pub child: NodeId,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Terminal {
        utility_p0: f64,
    },
    Chance {
        outcomes: Vec<ChanceOutcome>,
    },
    Decision {
        player: Player,
        infoset: InfosetId,
        actions: Vec<ActionEdge>,
    },
}

#[derive(Debug, Clone)]
pub struct Node {
    pub history: String,
    pub kind: NodeKind,
}

#[derive(Debug, Clone)]
pub struct Infoset {
    pub id: InfosetId,
    pub player: Player,
    pub key: String,
    pub action_labels: Vec<String>,
    pub node_ids: Vec<NodeId>,
}

#[derive(Debug, Clone)]
pub struct ExtensiveGame {
    name: String,
    root: NodeId,
    nodes: Vec<Node>,
    infosets: Vec<Infoset>,
}

impl ExtensiveGame {
    pub fn new(
        name: impl Into<String>,
        root: NodeId,
        nodes: Vec<Node>,
        infosets: Vec<Infoset>,
    ) -> Self {
        Self {
            name: name.into(),
            root,
            nodes,
            infosets,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn root(&self) -> NodeId {
        self.root
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn infosets(&self) -> &[Infoset] {
        &self.infosets
    }

    pub fn node(&self, node_id: NodeId) -> &Node {
        &self.nodes[node_id]
    }

    pub fn infoset(&self, infoset_id: InfosetId) -> &Infoset {
        &self.infosets[infoset_id]
    }

    pub fn infoset_id(&self, key: &str) -> Option<InfosetId> {
        self.infosets
            .iter()
            .find(|infoset| infoset.key == key)
            .map(|infoset| infoset.id)
    }

    pub fn player_infosets(&self, player: Player) -> Vec<InfosetId> {
        self.infosets
            .iter()
            .filter(|infoset| infoset.player == player)
            .map(|infoset| infoset.id)
            .collect()
    }

    pub fn terminal_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|node| matches!(node.kind, NodeKind::Terminal { .. }))
            .count()
    }

    pub fn decision_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|node| matches!(node.kind, NodeKind::Decision { .. }))
            .count()
    }

    pub fn chance_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|node| matches!(node.kind, NodeKind::Chance { .. }))
            .count()
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.nodes.is_empty() {
            return Err("game must contain at least one node".to_string());
        }
        if self.root >= self.nodes.len() {
            return Err(format!("root node {} is out of bounds", self.root));
        }

        for (node_id, node) in self.nodes.iter().enumerate() {
            match &node.kind {
                NodeKind::Terminal { .. } => {}
                NodeKind::Chance { outcomes } => {
                    if outcomes.is_empty() {
                        return Err(format!("chance node {node_id} has no outcomes"));
                    }
                    let mut sum = 0.0;
                    for outcome in outcomes {
                        if outcome.probability < 0.0 {
                            return Err(format!(
                                "chance node {node_id} has negative probability for outcome {}",
                                outcome.label
                            ));
                        }
                        if outcome.child >= self.nodes.len() {
                            return Err(format!(
                                "chance node {node_id} references missing child {}",
                                outcome.child
                            ));
                        }
                        sum += outcome.probability;
                    }
                    if (sum - 1.0).abs() > 1e-12 {
                        return Err(format!(
                            "chance node {node_id} probabilities sum to {sum}, expected 1.0"
                        ));
                    }
                }
                NodeKind::Decision {
                    player: _,
                    infoset,
                    actions,
                } => {
                    if *infoset >= self.infosets.len() {
                        return Err(format!(
                            "decision node {node_id} references missing infoset {infoset}"
                        ));
                    }
                    if actions.is_empty() {
                        return Err(format!("decision node {node_id} has no actions"));
                    }
                    for action in actions {
                        if action.child >= self.nodes.len() {
                            return Err(format!(
                                "decision node {node_id} references missing child {}",
                                action.child
                            ));
                        }
                    }
                }
            }
        }

        for (infoset_id, infoset) in self.infosets.iter().enumerate() {
            if infoset.id != infoset_id {
                return Err(format!(
                    "infoset at index {infoset_id} has mismatched id {}",
                    infoset.id
                ));
            }
            if infoset.node_ids.is_empty() {
                return Err(format!("infoset {} has no member nodes", infoset.key));
            }
            if infoset.action_labels.is_empty() {
                return Err(format!("infoset {} has no action labels", infoset.key));
            }
            for &node_id in &infoset.node_ids {
                let node = self.nodes.get(node_id).ok_or_else(|| {
                    format!("infoset {} references missing node {node_id}", infoset.key)
                })?;
                let NodeKind::Decision {
                    player,
                    infoset: node_infoset,
                    actions,
                } = &node.kind
                else {
                    return Err(format!(
                        "infoset {} references non-decision node {node_id}",
                        infoset.key
                    ));
                };
                if *player != infoset.player {
                    return Err(format!(
                        "infoset {} mixes players: expected {}, node {node_id} belongs to {}",
                        infoset.key, infoset.player, player
                    ));
                }
                if *node_infoset != infoset.id {
                    return Err(format!(
                        "infoset {} node {node_id} points at infoset {} instead of {}",
                        infoset.key, node_infoset, infoset.id
                    ));
                }
                if actions.len() != infoset.action_labels.len() {
                    return Err(format!(
                        "infoset {} action count mismatch at node {node_id}: {} vs {}",
                        infoset.key,
                        actions.len(),
                        infoset.action_labels.len()
                    ));
                }
                for (action, label) in actions.iter().zip(infoset.action_labels.iter()) {
                    if &action.label != label {
                        return Err(format!(
                            "infoset {} action label mismatch at node {node_id}: {} vs {}",
                            infoset.key, action.label, label
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}
