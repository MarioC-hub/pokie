#![forbid(unsafe_code)]

pub use config_core::SolveConfig;
pub use solver_core::poker::{PokerSolveError, PokerSolveResult};

pub fn solve_river_spot(config: &SolveConfig) -> Result<PokerSolveResult, PokerSolveError> {
    solver_core::poker::solve_river_spot(config)
}
