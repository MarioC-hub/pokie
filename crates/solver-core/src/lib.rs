#![forbid(unsafe_code)]

//! Exact conformance-first solver core for Pokie.
//!
//! Current capabilities:
//! - explicit extensive-form game trees
//! - deterministic full-tree tabular CFR
//! - exact pure-strategy best-response / exploitability checks for bounded games
//! - toy-game validation fixtures
//! - first end-to-end river poker vertical slice over exact combo ranges

pub mod cfr;
pub mod game;
pub mod games;
pub mod oracle;
pub mod poker;
pub mod profile;
pub mod solve;
