#!/usr/bin/env bash
set -euo pipefail

if [[ -f "$HOME/.cargo/env" ]]; then
  # shellcheck disable=SC1090
  . "$HOME/.cargo/env"
fi

if ! command -v rustc >/dev/null 2>&1 || ! command -v cargo >/dev/null 2>&1; then
  echo "rustc and cargo must be installed before running the conformance suite." >&2
  exit 1
fi

HOST_TRIPLE="$(rustc -vV | sed -n 's/^host: //p')"
SYSROOT="$(rustc --print sysroot)"
RUST_LLD="$SYSROOT/lib/rustlib/$HOST_TRIPLE/bin/rust-lld"

if [[ ! -x "$RUST_LLD" ]]; then
  echo "rust-lld not found at $RUST_LLD" >&2
  exit 1
fi

export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="$RUST_LLD"

cargo test --workspace --target x86_64-unknown-linux-musl "$@"
