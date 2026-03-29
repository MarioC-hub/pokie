#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib.sh"

load_cargo_env
require_command rustc
require_command cargo
require_rust_component rustfmt

TARGET_NAME="${1:-x86_64-pc-windows-gnu}"

cd "$ROOT_DIR"

cargo fmt --all --manifest-path desktop/src-tauri/Cargo.toml --check

if [[ "$TARGET_NAME" == "native" ]]; then
  cargo check --manifest-path desktop/src-tauri/Cargo.toml
else
  require_rust_target "$TARGET_NAME"
  cargo check --manifest-path desktop/src-tauri/Cargo.toml --target "$TARGET_NAME"
fi
