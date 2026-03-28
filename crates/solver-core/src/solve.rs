use crate::{
    cfr::CfrState,
    game::ExtensiveGame,
    oracle::{exact_exploitability, ExploitabilityReport},
    profile::StrategyProfile,
};

#[derive(Debug, Clone)]
pub struct SolveSummary {
    pub iterations: usize,
    pub current_profile: StrategyProfile,
    pub average_profile: StrategyProfile,
    pub exploitability: ExploitabilityReport,
}

pub fn solve_game(game: &ExtensiveGame, iterations: usize) -> SolveSummary {
    let mut state = CfrState::new(game);
    state.run_iterations(game, iterations);
    let current_profile = state.current_profile(game);
    let average_profile = state.average_profile(game);
    let exploitability = exact_exploitability(game, &average_profile);

    SolveSummary {
        iterations,
        current_profile,
        average_profile,
        exploitability,
    }
}
