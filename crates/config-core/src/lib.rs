#![forbid(unsafe_code)]

//! Canonical solve configuration for the current Pokie poker vertical slice.
//!
//! Current implemented production slice:
//! - heads-up NLHE cash semantics
//! - exact river-rooted postflop spots only
//! - exact board and exact weighted combo ranges
//! - a single-bet, no-raise river tree template
//! - node locks and rake are intentionally rejected for now

use std::fmt;

pub use poker_core::{Board, Card, PlayerRole, Street};
pub use range_core::{RangeError, WeightedCombo, WeightedRange};

pub const CURRENT_SCHEMA_VERSION: u32 = 1;
pub const RIVER_SINGLE_BET_TEMPLATE_ID: &str = "river_single_bet_v1";
pub const RIVER_SINGLE_BET_TEMPLATE_VERSION: u32 = 1;

pub type GameSpec = GameConfig;
pub type RangeBundle = RangeConfig;
pub type RiverSingleBetTemplate = RiverTreeTemplate;
pub type ConfigResult<T> = Result<T, ConfigError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Nlhe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Cash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameConfig {
    pub variant: Variant,
    pub mode: GameMode,
    pub active_players: u8,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            variant: Variant::Nlhe,
            mode: GameMode::Cash,
            active_players: 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlindProfile {
    pub small_blind: u32,
    pub big_blind: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RakeProfile {
    pub rake_bps: u32,
    pub cap: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootState {
    pub street: Street,
    pub board: Board,
    pub oop_stack: u32,
    pub ip_stack: u32,
    pub pot_size: u32,
    pub player_to_act: PlayerRole,
    pub blind_profile: BlindProfile,
    pub rake_profile: RakeProfile,
}

impl RootState {
    pub fn effective_stack(&self) -> u32 {
        self.oop_stack.min(self.ip_stack)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RangeConfig {
    pub oop_range: WeightedRange,
    pub ip_range: WeightedRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiverTreeTemplate {
    pub template_id: String,
    pub template_version: u32,
    pub first_bet_size: u32,
    pub after_check_bet_size: u32,
    pub max_raises_per_street: u8,
    pub allow_all_in: bool,
}

impl RiverTreeTemplate {
    pub fn single_bet(first_bet_size: u32, after_check_bet_size: u32) -> Self {
        Self {
            template_id: RIVER_SINGLE_BET_TEMPLATE_ID.to_string(),
            template_version: RIVER_SINGLE_BET_TEMPLATE_VERSION,
            first_bet_size,
            after_check_bet_size,
            max_raises_per_street: 0,
            allow_all_in: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlgorithmFamily {
    Cfr,
}

pub type SolverAlgorithm = AlgorithmFamily;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolverSettings {
    pub algorithm: AlgorithmFamily,
    pub iterations: usize,
    pub checkpoint_cadence: usize,
    pub thread_count: usize,
    pub deterministic_seed: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeLock {
    pub node_key: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolveConfig {
    pub schema_version: u32,
    pub game: GameConfig,
    pub root_state: RootState,
    pub ranges: RangeConfig,
    pub tree_template: RiverTreeTemplate,
    pub node_locks: Vec<NodeLock>,
    pub solver_settings: SolverSettings,
}

impl SolveConfig {
    #[allow(clippy::too_many_arguments)]
    pub fn river_single_bet(
        board: Board,
        oop_range: WeightedRange,
        ip_range: WeightedRange,
        player_to_act: PlayerRole,
        pot_size: u32,
        first_bet_size: u32,
        after_check_bet_size: u32,
        iterations: usize,
    ) -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            game: GameConfig::default(),
            root_state: RootState {
                street: Street::River,
                board,
                oop_stack: 100,
                ip_stack: 100,
                pot_size,
                player_to_act,
                blind_profile: BlindProfile {
                    small_blind: 1,
                    big_blind: 2,
                },
                rake_profile: RakeProfile {
                    rake_bps: 0,
                    cap: 0,
                },
            },
            ranges: RangeConfig {
                oop_range,
                ip_range,
            },
            tree_template: RiverTreeTemplate::single_bet(first_bet_size, after_check_bet_size),
            node_locks: Vec::new(),
            solver_settings: SolverSettings {
                algorithm: AlgorithmFamily::Cfr,
                iterations,
                checkpoint_cadence: 0,
                thread_count: 1,
                deterministic_seed: 0,
            },
        }
    }

    pub fn validate(&self) -> ConfigResult<()> {
        if self.schema_version != CURRENT_SCHEMA_VERSION {
            return Err(ConfigError::UnsupportedSchemaVersion(self.schema_version));
        }
        if self.game.variant != Variant::Nlhe {
            return Err(ConfigError::UnsupportedVariant(self.game.variant));
        }
        if self.game.mode != GameMode::Cash {
            return Err(ConfigError::UnsupportedMode(self.game.mode));
        }
        if self.game.active_players != 2 {
            return Err(ConfigError::UnsupportedActivePlayers(
                self.game.active_players,
            ));
        }
        if self.root_state.street != Street::River {
            return Err(ConfigError::UnsupportedStreet(self.root_state.street));
        }
        self.root_state
            .board
            .validate_for_street(Street::River)
            .map_err(ConfigError::InvalidBoard)?;
        if self.root_state.pot_size == 0 {
            return Err(ConfigError::InvalidPotSize(self.root_state.pot_size));
        }
        if self.root_state.oop_stack == 0 {
            return Err(ConfigError::InvalidStack {
                role: PlayerRole::Oop,
                stack: self.root_state.oop_stack,
            });
        }
        if self.root_state.ip_stack == 0 {
            return Err(ConfigError::InvalidStack {
                role: PlayerRole::Ip,
                stack: self.root_state.ip_stack,
            });
        }
        if self.root_state.blind_profile.small_blind == 0
            || self.root_state.blind_profile.big_blind == 0
            || self.root_state.blind_profile.small_blind >= self.root_state.blind_profile.big_blind
        {
            return Err(ConfigError::InvalidBlindProfile {
                small_blind: self.root_state.blind_profile.small_blind,
                big_blind: self.root_state.blind_profile.big_blind,
            });
        }
        if self.root_state.rake_profile.rake_bps != 0 || self.root_state.rake_profile.cap != 0 {
            return Err(ConfigError::UnsupportedRakeProfile(
                self.root_state.rake_profile,
            ));
        }
        if self.tree_template.template_id != RIVER_SINGLE_BET_TEMPLATE_ID {
            return Err(ConfigError::UnsupportedTemplateId(
                self.tree_template.template_id.clone(),
            ));
        }
        if self.tree_template.template_version != RIVER_SINGLE_BET_TEMPLATE_VERSION {
            return Err(ConfigError::UnsupportedTemplateVersion(
                self.tree_template.template_version,
            ));
        }
        if self.tree_template.max_raises_per_street != 0 {
            return Err(ConfigError::UnsupportedRaises(
                self.tree_template.max_raises_per_street,
            ));
        }
        if self.tree_template.allow_all_in {
            return Err(ConfigError::UnsupportedAllIn);
        }
        if self.tree_template.first_bet_size == 0 && self.tree_template.after_check_bet_size == 0 {
            return Err(ConfigError::NoActionsAvailable);
        }
        let effective_stack = self.root_state.effective_stack();
        for (label, size) in [
            ("first_bet_size", self.tree_template.first_bet_size),
            (
                "after_check_bet_size",
                self.tree_template.after_check_bet_size,
            ),
        ] {
            if size > effective_stack {
                return Err(ConfigError::InvalidBetSize {
                    label,
                    size,
                    effective_stack,
                });
            }
        }
        self.ranges
            .oop_range
            .validate_against_board(&self.root_state.board)
            .map_err(|source| ConfigError::InvalidRange {
                role: PlayerRole::Oop,
                source,
            })?;
        self.ranges
            .ip_range
            .validate_against_board(&self.root_state.board)
            .map_err(|source| ConfigError::InvalidRange {
                role: PlayerRole::Ip,
                source,
            })?;
        self.ranges
            .oop_range
            .compatible_pairings(&self.ranges.ip_range, &self.root_state.board)
            .map_err(|source| ConfigError::InvalidRange {
                role: PlayerRole::Oop,
                source,
            })?;
        if !self.node_locks.is_empty() {
            return Err(ConfigError::UnsupportedNodeLocks);
        }
        if self.solver_settings.algorithm != AlgorithmFamily::Cfr {
            return Err(ConfigError::InvalidSolverSettings(
                "only CFR is implemented for the current slice".to_string(),
            ));
        }
        if self.solver_settings.iterations == 0 {
            return Err(ConfigError::InvalidSolverSettings(
                "iterations must be positive".to_string(),
            ));
        }
        if self.solver_settings.thread_count == 0 {
            return Err(ConfigError::InvalidSolverSettings(
                "thread_count must be positive".to_string(),
            ));
        }
        Ok(())
    }

    pub fn count_compatible_range_pairs(&self) -> ConfigResult<usize> {
        Ok(self
            .ranges
            .oop_range
            .compatible_pairings(&self.ranges.ip_range, &self.root_state.board)
            .map_err(|source| ConfigError::InvalidRange {
                role: PlayerRole::Oop,
                source,
            })?
            .len())
    }

    pub fn canonical_string(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("schema_version={}", self.schema_version));
        lines.push("variant=nlhe".to_string());
        lines.push("mode=cash".to_string());
        lines.push("active_players=2".to_string());
        lines.push(format!("street={}", self.root_state.street));
        lines.push(format!("board={}", self.root_state.board));
        lines.push(format!("player_to_act={}", self.root_state.player_to_act));
        lines.push(format!("oop_stack={}", self.root_state.oop_stack));
        lines.push(format!("ip_stack={}", self.root_state.ip_stack));
        lines.push(format!("pot_size={}", self.root_state.pot_size));
        lines.push(format!(
            "small_blind={}",
            self.root_state.blind_profile.small_blind
        ));
        lines.push(format!(
            "big_blind={}",
            self.root_state.blind_profile.big_blind
        ));
        lines.push(format!(
            "rake_bps={}",
            self.root_state.rake_profile.rake_bps
        ));
        lines.push(format!("rake_cap={}", self.root_state.rake_profile.cap));
        lines.push(format!(
            "oop_range={}",
            self.ranges.oop_range.canonical_string()
        ));
        lines.push(format!(
            "ip_range={}",
            self.ranges.ip_range.canonical_string()
        ));
        lines.push(format!("template_id={}", self.tree_template.template_id));
        lines.push(format!(
            "template_version={}",
            self.tree_template.template_version
        ));
        lines.push(format!(
            "first_bet_size={}",
            self.tree_template.first_bet_size
        ));
        lines.push(format!(
            "after_check_bet_size={}",
            self.tree_template.after_check_bet_size
        ));
        lines.push(format!(
            "max_raises_per_street={}",
            self.tree_template.max_raises_per_street
        ));
        lines.push(format!("allow_all_in={}", self.tree_template.allow_all_in));
        lines.push(format!("algorithm={:?}", self.solver_settings.algorithm));
        lines.push(format!("iterations={}", self.solver_settings.iterations));
        lines.push(format!(
            "checkpoint_cadence={}",
            self.solver_settings.checkpoint_cadence
        ));
        lines.push(format!(
            "thread_count={}",
            self.solver_settings.thread_count
        ));
        lines.push(format!(
            "deterministic_seed={}",
            self.solver_settings.deterministic_seed
        ));
        lines.join("\n")
    }

    pub fn canonical_hash(&self) -> ConfigResult<String> {
        self.validate()?;
        Ok(hex_string(&canonical_hash_bytes(
            self.canonical_string().as_bytes(),
        )))
    }

    pub fn stable_hash(&self) -> ConfigResult<String> {
        self.canonical_hash()
    }

    pub fn hash(&self) -> ConfigResult<String> {
        self.canonical_hash()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    UnsupportedSchemaVersion(u32),
    UnsupportedVariant(Variant),
    UnsupportedMode(GameMode),
    UnsupportedActivePlayers(u8),
    UnsupportedStreet(Street),
    InvalidBoard(poker_core::PokerError),
    InvalidPotSize(u32),
    InvalidStack {
        role: PlayerRole,
        stack: u32,
    },
    InvalidBlindProfile {
        small_blind: u32,
        big_blind: u32,
    },
    UnsupportedRakeProfile(RakeProfile),
    InvalidRange {
        role: PlayerRole,
        source: RangeError,
    },
    UnsupportedTemplateId(String),
    UnsupportedTemplateVersion(u32),
    NoActionsAvailable,
    InvalidBetSize {
        label: &'static str,
        size: u32,
        effective_stack: u32,
    },
    UnsupportedRaises(u8),
    UnsupportedAllIn,
    UnsupportedNodeLocks,
    InvalidSolverSettings(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion(version) => {
                write!(f, "unsupported schema version {version}")
            }
            Self::UnsupportedVariant(variant) => write!(f, "unsupported variant {variant:?}"),
            Self::UnsupportedMode(mode) => write!(f, "unsupported game mode {mode:?}"),
            Self::UnsupportedActivePlayers(players) => {
                write!(f, "unsupported active player count {players}")
            }
            Self::UnsupportedStreet(street) => write!(f, "unsupported root street {street}"),
            Self::InvalidBoard(source) => write!(f, "invalid board: {source}"),
            Self::InvalidPotSize(size) => write!(f, "invalid pot size {size}"),
            Self::InvalidStack { role, stack } => write!(f, "invalid {role} stack {stack}"),
            Self::InvalidBlindProfile {
                small_blind,
                big_blind,
            } => write!(f, "invalid blind profile {small_blind}/{big_blind}"),
            Self::UnsupportedRakeProfile(profile) => {
                write!(
                    f,
                    "rake is not implemented yet (rake_bps={}, cap={})",
                    profile.rake_bps, profile.cap
                )
            }
            Self::InvalidRange { role, source } => write!(f, "invalid {role} range: {source}"),
            Self::UnsupportedTemplateId(id) => write!(f, "unsupported tree template id {id}"),
            Self::UnsupportedTemplateVersion(version) => {
                write!(f, "unsupported tree template version {version}")
            }
            Self::NoActionsAvailable => write!(f, "tree template offers no legal actions"),
            Self::InvalidBetSize {
                label,
                size,
                effective_stack,
            } => write!(
                f,
                "{label} {size} exceeds effective stack {effective_stack}"
            ),
            Self::UnsupportedRaises(raises) => write!(
                f,
                "raises are unsupported in the current slice (got {raises})"
            ),
            Self::UnsupportedAllIn => {
                write!(f, "all-in templates are unsupported in the current slice")
            }
            Self::UnsupportedNodeLocks => {
                write!(f, "node locks are unsupported in the current slice")
            }
            Self::InvalidSolverSettings(message) => write!(f, "invalid solver settings: {message}"),
        }
    }
}

impl std::error::Error for ConfigError {}

pub fn canonical_hash_bytes(bytes: &[u8]) -> [u8; 16] {
    let mut hash = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58du128;
    for byte in bytes {
        hash ^= u128::from(*byte);
        hash = hash.wrapping_mul(0x0000_0000_0100_0000_0000_0000_0000_013bu128);
    }
    hash.to_be_bytes()
}

fn hex_string(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("writing to string cannot fail");
    }
    output
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use poker_core::{Board, HoleCards};
    use range_core::WeightedRange;

    use super::{
        AlgorithmFamily, BlindProfile, ConfigError, GameConfig, GameMode, PlayerRole, RakeProfile,
        RangeConfig, RiverTreeTemplate, RootState, SolveConfig, SolverSettings, Street, Variant,
        CURRENT_SCHEMA_VERSION, RIVER_SINGLE_BET_TEMPLATE_ID,
    };

    fn base_config() -> SolveConfig {
        SolveConfig {
            schema_version: CURRENT_SCHEMA_VERSION,
            game: GameConfig {
                variant: Variant::Nlhe,
                mode: GameMode::Cash,
                active_players: 2,
            },
            root_state: RootState {
                street: Street::River,
                board: Board::from_str("KhTd8s5c2h").unwrap(),
                oop_stack: 100,
                ip_stack: 100,
                pot_size: 20,
                player_to_act: PlayerRole::Oop,
                blind_profile: BlindProfile {
                    small_blind: 1,
                    big_blind: 2,
                },
                rake_profile: RakeProfile {
                    rake_bps: 0,
                    cap: 0,
                },
            },
            ranges: RangeConfig {
                oop_range: WeightedRange::from_hands(vec![
                    (HoleCards::from_str("AsAh").unwrap(), 2.0),
                    (HoleCards::from_str("3c4c").unwrap(), 1.0),
                ])
                .unwrap(),
                ip_range: WeightedRange::from_hands(vec![(
                    HoleCards::from_str("KdQd").unwrap(),
                    1.0,
                )])
                .unwrap(),
            },
            tree_template: RiverTreeTemplate {
                template_id: RIVER_SINGLE_BET_TEMPLATE_ID.to_string(),
                template_version: 1,
                first_bet_size: 10,
                after_check_bet_size: 0,
                max_raises_per_street: 0,
                allow_all_in: false,
            },
            node_locks: Vec::new(),
            solver_settings: SolverSettings {
                algorithm: AlgorithmFamily::Cfr,
                iterations: 10,
                checkpoint_cadence: 0,
                thread_count: 1,
                deterministic_seed: 0,
            },
        }
    }

    #[test]
    fn canonical_hash_is_stable_across_range_ordering() {
        let left = base_config();
        let mut right = base_config();
        right.ranges.oop_range = WeightedRange::from_hands(vec![
            (HoleCards::from_str("3c4c").unwrap(), 1.0),
            (HoleCards::from_str("AsAh").unwrap(), 2.0),
        ])
        .unwrap();

        assert_eq!(
            left.canonical_hash().unwrap(),
            right.canonical_hash().unwrap()
        );
        left.validate().unwrap();
    }

    #[test]
    fn rejects_non_river_roots_exactly() {
        let mut config = base_config();
        config.root_state.street = Street::Turn;
        assert!(matches!(
            config.validate(),
            Err(ConfigError::UnsupportedStreet(Street::Turn))
        ));
    }

    #[test]
    fn rejects_nonzero_rake_until_implemented() {
        let mut config = base_config();
        config.root_state.rake_profile = RakeProfile {
            rake_bps: 500,
            cap: 3,
        };
        assert!(matches!(
            config.validate(),
            Err(ConfigError::UnsupportedRakeProfile(RakeProfile {
                rake_bps: 500,
                cap: 3
            }))
        ));
    }

    #[test]
    fn counts_compatible_private_hand_pairs_exactly() {
        let config = base_config();
        assert_eq!(config.count_compatible_range_pairs().unwrap(), 2);
    }
}
