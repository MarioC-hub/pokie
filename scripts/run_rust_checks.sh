#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib.sh"

load_cargo_env
require_command rustc
require_command cargo
require_rust_component rustfmt
require_rust_component clippy
require_rust_target x86_64-unknown-linux-musl

cd "$ROOT_DIR"

cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
"$ROOT_DIR/scripts/run_workspace_tests.sh"
"$ROOT_DIR/scripts/run_solver_conformance.sh" -- --nocapture
