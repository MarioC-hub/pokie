#!/usr/bin/env bash

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

load_cargo_env() {
  if [[ -d "$HOME/.local/bin" ]]; then
    export PATH="$HOME/.local/bin:$PATH"
  fi
  if [[ -f "$HOME/.cargo/env" ]]; then
    # shellcheck disable=SC1090
    . "$HOME/.cargo/env"
  fi
}

require_command() {
  local command_name="$1"

  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "$command_name must be installed before running this check." >&2
    exit 1
  fi
}

require_rust_component() {
  local component_name="$1"

  require_command rustup
  if ! rustup component list --installed | grep -q "^${component_name}-"; then
    echo "missing Rust component '$component_name'. Install it with: rustup component add ${component_name}" >&2
    exit 1
  fi
}

require_rust_target() {
  local target_name="$1"

  require_command rustup
  if ! rustup target list --installed | grep -qx "$target_name"; then
    echo "missing Rust target '$target_name'. Install it with: rustup target add ${target_name}" >&2
    exit 1
  fi
}

configure_musl_linker() {
  require_command rustc

  local host_triple
  local sysroot
  local rust_lld

  host_triple="$(rustc -vV | sed -n 's/^host: //p')"
  sysroot="$(rustc --print sysroot)"
  rust_lld="$sysroot/lib/rustlib/$host_triple/bin/rust-lld"

  if [[ ! -x "$rust_lld" ]]; then
    echo "rust-lld not found at $rust_lld" >&2
    exit 1
  fi

  export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="$rust_lld"
}
