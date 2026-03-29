#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib.sh"

require_command npm

cd "$ROOT_DIR/desktop"

npm run format:check
npm run lint
npm run build
