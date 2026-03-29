#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib.sh"

load_cargo_env
require_command rustc
require_command cargo
require_rust_target x86_64-unknown-linux-musl
configure_musl_linker

cd "$ROOT_DIR"

cargo test --workspace --target x86_64-unknown-linux-musl "$@"
