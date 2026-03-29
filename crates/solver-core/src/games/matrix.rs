use crate::game::{ActionEdge, ExtensiveGame, Infoset, Node, NodeKind, Player};

pub fn simultaneous_matrix_game(
    name: &str,
    row_actions: &[&str],
    column_actions: &[&str],
    payoff_matrix_p0: &[Vec<f64>],
) -> ExtensiveGame {
    assert_eq!(
        payoff_matrix_p0.len(),
        row_actions.len(),
        "row action count mismatch"
    );
    for row in payoff_matrix_p0 {
        assert_eq!(
            row.len(),
            column_actions.len(),
            "column action count mismatch"
        );
    }

    let mut nodes = Vec::new();
    let mut infosets = vec![
        Infoset {
            id: 0,
            player: Player::P0,
            key: format!("{name}:P0/root"),
            action_labels: row_actions
                .iter()
                .map(|label| (*label).to_string())
                .collect(),
            node_ids: vec![0],
        },
        Infoset {
            id: 1,
            player: Player::P1,
            key: format!("{name}:P1/hidden"),
            action_labels: column_actions
                .iter()
                .map(|label| (*label).to_string())
                .collect(),
            node_ids: Vec::new(),
        },
    ];

    nodes.push(Node {
        history: "root".to_string(),
        kind: NodeKind::Decision {
            player: Player::P0,
            infoset: 0,
            actions: Vec::new(),
        },
    });

    let mut root_actions = Vec::new();
    for (row_index, row_label) in row_actions.iter().enumerate() {
        let p1_node_id = nodes.len();
        infosets[1].node_ids.push(p1_node_id);
        nodes.push(Node {
            history: format!("P0:{row_label}"),
            kind: NodeKind::Decision {
                player: Player::P1,
                infoset: 1,
                actions: Vec::new(),
            },
        });

        let mut p1_actions = Vec::new();
        for (column_index, column_label) in column_actions.iter().enumerate() {
            let terminal_node_id = nodes.len();
            nodes.push(Node {
                history: format!("P0:{row_label} -> P1:{column_label}"),
                kind: NodeKind::Terminal {
                    utility_p0: payoff_matrix_p0[row_index][column_index],
                },
            });
            p1_actions.push(ActionEdge {
                label: (*column_label).to_string(),
                child: terminal_node_id,
            });
        }
        nodes[p1_node_id].kind = NodeKind::Decision {
            player: Player::P1,
            infoset: 1,
            actions: p1_actions,
        };
        root_actions.push(ActionEdge {
            label: (*row_label).to_string(),
            child: p1_node_id,
        });
    }

    nodes[0].kind = NodeKind::Decision {
        player: Player::P0,
        infoset: 0,
        actions: root_actions,
    };

    let game = ExtensiveGame::new(name, 0, nodes, infosets);
    game.validate().expect("matrix game should be valid");
    game
}
