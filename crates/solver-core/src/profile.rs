use std::collections::BTreeMap;

use crate::game::{ExtensiveGame, InfosetId};

#[derive(Debug, Clone)]
pub struct StrategyProfile {
    infoset_strategies: Vec<Vec<f64>>,
}

impl StrategyProfile {
    pub fn new(infoset_strategies: Vec<Vec<f64>>) -> Self {
        Self { infoset_strategies }
    }

    pub fn uniform(game: &ExtensiveGame) -> Self {
        let infoset_strategies = game
            .infosets()
            .iter()
            .map(|infoset| {
                let probability = 1.0 / infoset.action_labels.len() as f64;
                vec![probability; infoset.action_labels.len()]
            })
            .collect();
        Self { infoset_strategies }
    }

    pub fn from_named_probabilities(
        game: &ExtensiveGame,
        probabilities: &BTreeMap<String, Vec<f64>>,
    ) -> Result<Self, String> {
        let mut infoset_strategies = Vec::with_capacity(game.infosets().len());
        for infoset in game.infosets() {
            let values = probabilities.get(&infoset.key).ok_or_else(|| {
                format!(
                    "missing strategy probabilities for infoset {} in game {}",
                    infoset.key,
                    game.name()
                )
            })?;
            if values.len() != infoset.action_labels.len() {
                return Err(format!(
                    "infoset {} expected {} probabilities, found {}",
                    infoset.key,
                    infoset.action_labels.len(),
                    values.len()
                ));
            }
            infoset_strategies.push(values.clone());
        }
        let profile = Self { infoset_strategies };
        profile.validate(game)?;
        Ok(profile)
    }

    pub fn validate(&self, game: &ExtensiveGame) -> Result<(), String> {
        if self.infoset_strategies.len() != game.infosets().len() {
            return Err(format!(
                "profile has {} infoset rows, expected {}",
                self.infoset_strategies.len(),
                game.infosets().len()
            ));
        }
        for (infoset, probabilities) in game.infosets().iter().zip(self.infoset_strategies.iter()) {
            if probabilities.len() != infoset.action_labels.len() {
                return Err(format!(
                    "infoset {} expected {} actions, found {}",
                    infoset.key,
                    infoset.action_labels.len(),
                    probabilities.len()
                ));
            }
            let mut sum = 0.0;
            for probability in probabilities {
                if *probability < -1e-15 {
                    return Err(format!(
                        "infoset {} has negative action probability {probability}",
                        infoset.key
                    ));
                }
                sum += probability;
            }
            if (sum - 1.0).abs() > 1e-12 {
                return Err(format!(
                    "infoset {} probabilities sum to {sum}, expected 1.0",
                    infoset.key
                ));
            }
        }
        Ok(())
    }

    pub fn probabilities(&self, infoset: InfosetId) -> &[f64] {
        &self.infoset_strategies[infoset]
    }

    pub fn probability_by_key<'a>(
        &'a self,
        game: &'a ExtensiveGame,
        key: &str,
    ) -> Option<&'a [f64]> {
        let infoset_id = game.infoset_id(key)?;
        Some(self.probabilities(infoset_id))
    }

    pub fn infoset_strategies(&self) -> &[Vec<f64>] {
        &self.infoset_strategies
    }
}
