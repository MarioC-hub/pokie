#![forbid(unsafe_code)]

//! Versioned river-only public tree generation for the first Pokie slice.

mod river;

pub use river::{
    build_river_tree, RiverTerminalOutcome, RiverTree, RiverTreeAction, RiverTreeError,
    RiverTreeNode, RiverTreeNodeKind,
};
