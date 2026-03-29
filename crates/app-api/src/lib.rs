#![forbid(unsafe_code)]

use std::str::FromStr;

use config_core::{
    AlgorithmFamily, BlindProfile, Board, GameConfig, GameMode, PlayerRole, RakeProfile,
    RangeConfig, RiverTreeTemplate, RootState, SolveConfig, SolverSettings, Street, Variant,
    WeightedRange, CURRENT_SCHEMA_VERSION, RIVER_SINGLE_BET_TEMPLATE_ID,
};
use serde::{Deserialize, Serialize};
use solver_core::{
    game::NodeKind,
    poker::{PokerSolveError, PokerSolveResult},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RiverSolveRequestDto {
    pub board: String,
    pub oop_range: String,
    pub ip_range: String,
    pub pot_size: String,
    pub oop_stack: String,
    pub ip_stack: String,
    pub player_to_act: String,
    pub small_blind: String,
    pub big_blind: String,
    pub first_bet_size: String,
    pub after_check_bet_size: String,
    pub iterations: String,
    pub deterministic_seed: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ValidateConfigResponseDto {
    pub config_hash: String,
    pub compatible_deal_count: usize,
    pub normalized: NormalizedRiverConfigDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedRiverConfigDto {
    pub schema_version: u32,
    pub variant: String,
    pub mode: String,
    pub active_players: u8,
    pub street: String,
    pub board: String,
    pub oop_range: String,
    pub ip_range: String,
    pub pot_size: u32,
    pub oop_stack: u32,
    pub ip_stack: u32,
    pub player_to_act: String,
    pub small_blind: u32,
    pub big_blind: u32,
    pub rake_bps: u32,
    pub rake_cap: u32,
    pub template_id: String,
    pub template_version: u32,
    pub first_bet_size: u32,
    pub after_check_bet_size: u32,
    pub max_raises_per_street: u8,
    pub allow_all_in: bool,
    pub algorithm: String,
    pub iterations: usize,
    pub checkpoint_cadence: usize,
    pub thread_count: usize,
    pub deterministic_seed: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RiverSolveResponseDto {
    pub config_hash: String,
    pub compatible_deal_count: usize,
    pub normalized: NormalizedRiverConfigDto,
    pub tree_identity: String,
    pub iterations: usize,
    pub root_value_oop: f64,
    pub nash_conv: f64,
    pub p0_improvement: f64,
    pub p1_improvement: f64,
    pub root_infosets: Vec<InfosetStrategyDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InfosetStrategyDto {
    pub key: String,
    pub actions: Vec<ActionProbabilityDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ActionProbabilityDto {
    pub label: String,
    pub probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppErrorDto {
    pub code: String,
    pub message: String,
}

impl RiverSolveRequestDto {
    pub fn to_solve_config(&self) -> Result<SolveConfig, AppErrorDto> {
        let board = Board::from_str(self.board.trim())
            .map_err(|error| app_error("invalid_board", format!("invalid board: {error}")))?;
        let oop_range = WeightedRange::from_str(self.oop_range.trim()).map_err(|error| {
            app_error("invalid_oop_range", format!("invalid OOP range: {error}"))
        })?;
        let ip_range = WeightedRange::from_str(self.ip_range.trim())
            .map_err(|error| app_error("invalid_ip_range", format!("invalid IP range: {error}")))?;
        let player_to_act = parse_player_role(&self.player_to_act)?;
        let pot_size = parse_u32_field("pot_size", &self.pot_size)?;
        let oop_stack = parse_u32_field("oop_stack", &self.oop_stack)?;
        let ip_stack = parse_u32_field("ip_stack", &self.ip_stack)?;
        let small_blind = parse_u32_field("small_blind", &self.small_blind)?;
        let big_blind = parse_u32_field("big_blind", &self.big_blind)?;
        let first_bet_size = parse_u32_field("first_bet_size", &self.first_bet_size)?;
        let after_check_bet_size =
            parse_u32_field("after_check_bet_size", &self.after_check_bet_size)?;
        let iterations = parse_usize_field("iterations", &self.iterations)?;
        let deterministic_seed = parse_u64_field("deterministic_seed", &self.deterministic_seed)?;

        Ok(SolveConfig {
            schema_version: CURRENT_SCHEMA_VERSION,
            game: GameConfig {
                variant: Variant::Nlhe,
                mode: GameMode::Cash,
                active_players: 2,
            },
            root_state: RootState {
                street: Street::River,
                board,
                oop_stack,
                ip_stack,
                pot_size,
                player_to_act,
                blind_profile: BlindProfile {
                    small_blind,
                    big_blind,
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
            tree_template: RiverTreeTemplate {
                template_id: RIVER_SINGLE_BET_TEMPLATE_ID.to_string(),
                template_version: 1,
                first_bet_size,
                after_check_bet_size,
                max_raises_per_street: 0,
                allow_all_in: false,
            },
            node_locks: Vec::new(),
            solver_settings: SolverSettings {
                algorithm: AlgorithmFamily::Cfr,
                iterations,
                checkpoint_cadence: 0,
                thread_count: 1,
                deterministic_seed,
            },
        })
    }
}

pub fn sample_river_request() -> RiverSolveRequestDto {
    RiverSolveRequestDto {
        board: "Ks7d4c2h2d".to_string(),
        oop_range: "7c7h:1.0,AcJc:1.0".to_string(),
        ip_range: "KcQh:1.0".to_string(),
        pot_size: "10".to_string(),
        oop_stack: "100".to_string(),
        ip_stack: "100".to_string(),
        player_to_act: "oop".to_string(),
        small_blind: "1".to_string(),
        big_blind: "2".to_string(),
        first_bet_size: "10".to_string(),
        after_check_bet_size: "0".to_string(),
        iterations: "2000".to_string(),
        deterministic_seed: "0".to_string(),
    }
}

pub fn validate_config(
    request: &RiverSolveRequestDto,
) -> Result<ValidateConfigResponseDto, AppErrorDto> {
    let config = request.to_solve_config()?;
    config
        .validate()
        .map_err(|error| app_error("invalid_config", error.to_string()))?;

    let config_hash = config
        .canonical_hash()
        .map_err(|error| app_error("invalid_config", error.to_string()))?;
    let compatible_deal_count = config
        .count_compatible_range_pairs()
        .map_err(|error| app_error("invalid_config", error.to_string()))?;

    Ok(ValidateConfigResponseDto {
        config_hash,
        compatible_deal_count,
        normalized: NormalizedRiverConfigDto::from_config(&config),
    })
}

pub fn solve_river_spot(
    request: &RiverSolveRequestDto,
) -> Result<RiverSolveResponseDto, AppErrorDto> {
    let config = request.to_solve_config()?;
    let result = solver_core::poker::solve_river_spot(&config).map_err(map_solve_error)?;
    Ok(RiverSolveResponseDto::from_result(&result, &config))
}

impl NormalizedRiverConfigDto {
    fn from_config(config: &SolveConfig) -> Self {
        Self {
            schema_version: config.schema_version,
            variant: "nlhe".to_string(),
            mode: "cash".to_string(),
            active_players: config.game.active_players,
            street: config.root_state.street.to_string(),
            board: config.root_state.board.to_string(),
            oop_range: config.ranges.oop_range.canonical_string(),
            ip_range: config.ranges.ip_range.canonical_string(),
            pot_size: config.root_state.pot_size,
            oop_stack: config.root_state.oop_stack,
            ip_stack: config.root_state.ip_stack,
            player_to_act: config.root_state.player_to_act.to_string(),
            small_blind: config.root_state.blind_profile.small_blind,
            big_blind: config.root_state.blind_profile.big_blind,
            rake_bps: config.root_state.rake_profile.rake_bps,
            rake_cap: config.root_state.rake_profile.cap,
            template_id: config.tree_template.template_id.clone(),
            template_version: config.tree_template.template_version,
            first_bet_size: config.tree_template.first_bet_size,
            after_check_bet_size: config.tree_template.after_check_bet_size,
            max_raises_per_street: config.tree_template.max_raises_per_street,
            allow_all_in: config.tree_template.allow_all_in,
            algorithm: "cfr".to_string(),
            iterations: config.solver_settings.iterations,
            checkpoint_cadence: config.solver_settings.checkpoint_cadence,
            thread_count: config.solver_settings.thread_count,
            deterministic_seed: config.solver_settings.deterministic_seed.to_string(),
        }
    }
}

impl RiverSolveResponseDto {
    fn from_result(result: &PokerSolveResult, config: &SolveConfig) -> Self {
        let compatible_deal_count = match &result.game.node(result.game.root()).kind {
            NodeKind::Chance { outcomes } => outcomes.len(),
            _ => 0,
        };

        let root_infoset_ids = match &result.game.node(result.game.root()).kind {
            NodeKind::Chance { outcomes } => outcomes
                .iter()
                .filter_map(|outcome| match &result.game.node(outcome.child).kind {
                    NodeKind::Decision { infoset, .. } => Some(*infoset),
                    _ => None,
                })
                .collect::<std::collections::BTreeSet<_>>(),
            _ => std::collections::BTreeSet::new(),
        };

        let root_infosets = root_infoset_ids
            .into_iter()
            .map(|infoset_id| {
                let infoset = result.game.infoset(infoset_id);
                InfosetStrategyDto {
                    key: infoset.key.clone(),
                    actions: infoset
                        .action_labels
                        .iter()
                        .cloned()
                        .zip(
                            result
                                .average_profile
                                .probabilities(infoset.id)
                                .iter()
                                .copied(),
                        )
                        .map(|(label, probability)| ActionProbabilityDto { label, probability })
                        .collect(),
                }
            })
            .collect();

        Self {
            config_hash: result.config_hash.clone(),
            compatible_deal_count,
            normalized: NormalizedRiverConfigDto::from_config(config),
            tree_identity: result.tree_identity.clone(),
            iterations: result.iterations,
            root_value_oop: result.root_value_oop,
            nash_conv: result.exploitability.nash_conv,
            p0_improvement: result.exploitability.p0_improvement,
            p1_improvement: result.exploitability.p1_improvement,
            root_infosets,
        }
    }
}

fn parse_player_role(value: &str) -> Result<PlayerRole, AppErrorDto> {
    match value.trim().to_ascii_lowercase().as_str() {
        "oop" => Ok(PlayerRole::Oop),
        "ip" => Ok(PlayerRole::Ip),
        other => Err(app_error(
            "invalid_player_to_act",
            format!("player_to_act must be 'oop' or 'ip', got {other:?}"),
        )),
    }
}

fn parse_u32_field(field: &str, value: &str) -> Result<u32, AppErrorDto> {
    value.trim().parse::<u32>().map_err(|_| {
        app_error(
            format!("invalid_{field}"),
            format!("{field} must be a non-negative integer string"),
        )
    })
}

fn parse_usize_field(field: &str, value: &str) -> Result<usize, AppErrorDto> {
    value.trim().parse::<usize>().map_err(|_| {
        app_error(
            format!("invalid_{field}"),
            format!("{field} must be a non-negative integer string"),
        )
    })
}

fn parse_u64_field(field: &str, value: &str) -> Result<u64, AppErrorDto> {
    value.trim().parse::<u64>().map_err(|_| {
        app_error(
            format!("invalid_{field}"),
            format!("{field} must be a non-negative integer string"),
        )
    })
}

fn map_solve_error(error: PokerSolveError) -> AppErrorDto {
    match error {
        PokerSolveError::Config(message) => app_error("invalid_config", message),
        PokerSolveError::Range(message) => app_error("invalid_range", message),
        PokerSolveError::Tree(message) => app_error("tree_generation_failed", message),
        PokerSolveError::EmptyChanceSupport => app_error(
            "empty_chance_support",
            "no disjoint private-hand combinations remain after card removal".to_string(),
        ),
    }
}

fn app_error(code: impl Into<String>, message: impl Into<String>) -> AppErrorDto {
    AppErrorDto {
        code: code.into(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{sample_river_request, solve_river_spot, validate_config};

    #[test]
    fn sample_request_validates_into_canonical_contract() {
        let response =
            validate_config(&sample_river_request()).expect("sample request should validate");
        assert_eq!(response.normalized.variant, "nlhe");
        assert_eq!(response.normalized.mode, "cash");
        assert_eq!(response.normalized.street, "river");
        assert_eq!(response.compatible_deal_count, 2);
        assert_eq!(response.normalized.board, "2d2h4c7dKs");
        assert_eq!(response.normalized.oop_range, "7c7h:0.5,JcAc:0.5");
        assert_eq!(response.normalized.ip_range, "QhKc:1");
        assert_eq!(response.normalized.deterministic_seed, "0");
    }

    #[test]
    fn one_iteration_solve_exposes_exact_root_infosets() {
        let mut request = sample_river_request();
        request.iterations = "1".to_string();
        let response = solve_river_spot(&request).expect("solve should succeed");

        assert_eq!(response.iterations, 1);
        assert_eq!(response.root_infosets.len(), 2);
        for infoset in &response.root_infosets {
            assert!(infoset.key.ends_with(":root"));
            assert_eq!(infoset.actions.len(), 2);
            assert_eq!(infoset.actions[0].label, "check");
            assert_eq!(infoset.actions[1].label, "bet_10");
            assert!((infoset.actions[0].probability - 0.5).abs() <= 1e-12);
            assert!((infoset.actions[1].probability - 0.5).abs() <= 1e-12);
        }
    }

    #[test]
    fn fractional_numeric_fields_are_rejected_before_solver_execution() {
        let mut request = sample_river_request();
        request.first_bet_size = "10.5".to_string();
        let error = validate_config(&request).expect_err("fractional sizes should fail validation");
        assert_eq!(error.code, "invalid_first_bet_size");
    }
}
