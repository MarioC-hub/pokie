#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib.sh"

DESKTOP_TARGET="${1:-x86_64-pc-windows-gnu}"

"$ROOT_DIR/scripts/run_rust_checks.sh"
"$ROOT_DIR/scripts/run_frontend_checks.sh"
"$ROOT_DIR/scripts/run_desktop_shell_checks.sh" "$DESKTOP_TARGET"

if [[ "${POKIE_RUN_DESKTOP_SMOKE:-0}" == "1" ]]; then
  "$ROOT_DIR/scripts/run_desktop_e2e_smoke.sh"
fi
