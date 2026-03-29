#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
POWERSHELL="/mnt/c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe"
WINDOWS_EXE="$(wslpath -w "$ROOT_DIR/desktop/src-tauri/target/x86_64-pc-windows-gnu/debug/pokie-desktop.exe")"
SMOKE_ENV_FILE="$ROOT_DIR/desktop/.env.local"
REPORT_FILE="$ROOT_DIR/desktop/.pokie-e2e-report.json"
WINDOWS_REPORT_FILE="$(wslpath -w "$REPORT_FILE")"
TRACE_FILE="$ROOT_DIR/desktop/.pokie-e2e-trace.log"
WINDOWS_TRACE_FILE="$(wslpath -w "$TRACE_FILE")"
VITE_LOG="${TMPDIR:-/tmp}/pokie-desktop-e2e-vite.log"
APP_PID=""
VITE_PID=""

export PATH="$HOME/.local/bin:$PATH"

cleanup() {
  if [[ -n "${APP_PID}" ]]; then
    "$POWERSHELL" -NoProfile -Command "Get-Process -Id ${APP_PID} -ErrorAction SilentlyContinue | Stop-Process -Force" >/dev/null 2>&1 || true
  fi
  "$POWERSHELL" -NoProfile -Command 'Get-Process pokie-desktop -ErrorAction SilentlyContinue | Stop-Process -Force' >/dev/null 2>&1 || true
  if [[ -n "${VITE_PID}" ]]; then
    kill "${VITE_PID}" >/dev/null 2>&1 || true
    wait "${VITE_PID}" >/dev/null 2>&1 || true
  fi
  pkill -f 'vite --host 127.0.0.1 --port 1420' >/dev/null 2>&1 || true
  "$POWERSHELL" -NoProfile -Command 'Get-NetTCPConnection -LocalPort 1420 -State Listen -ErrorAction SilentlyContinue | Select-Object -ExpandProperty OwningProcess -Unique | ForEach-Object { Stop-Process -Id $_ -Force -ErrorAction SilentlyContinue }' >/dev/null 2>&1 || true
  rm -f "$SMOKE_ENV_FILE"
  rm -f "$REPORT_FILE"
  rm -f "$TRACE_FILE"
}
trap cleanup EXIT

echo "==> Closing any prior Windows desktop instance"
"$POWERSHELL" -NoProfile -Command '
  Get-Process pokie-desktop -ErrorAction SilentlyContinue | Stop-Process -Force
  $ws = New-Object -ComObject WScript.Shell
  if ($ws.AppActivate("pokie-desktop.exe - Entry Point Not Found")) {
    Start-Sleep -Milliseconds 200
    $ws.SendKeys("{ENTER}")
  }
' >/dev/null 2>&1 || true
pkill -f 'vite --host 127.0.0.1 --port 1420' >/dev/null 2>&1 || true
"$POWERSHELL" -NoProfile -Command 'Get-NetTCPConnection -LocalPort 1420 -State Listen -ErrorAction SilentlyContinue | Select-Object -ExpandProperty OwningProcess -Unique | ForEach-Object { Stop-Process -Id $_ -Force -ErrorAction SilentlyContinue }' >/dev/null 2>&1 || true

echo "==> Building Windows desktop shell"
cargo zigbuild --manifest-path "$ROOT_DIR/desktop/src-tauri/Cargo.toml" --target x86_64-pc-windows-gnu >/dev/null

echo "==> Starting Vite dev server with desktop smoke mode"
printf 'VITE_POKIE_E2E=1\n' > "$SMOKE_ENV_FILE"
rm -f "$REPORT_FILE"
rm -f "$TRACE_FILE"
(
  cd "$ROOT_DIR/desktop"
  npm run dev >"$VITE_LOG" 2>&1
) &
VITE_PID=$!

for _ in $(seq 1 30); do
  if "$POWERSHELL" -NoProfile -Command 'try { (Invoke-WebRequest -UseBasicParsing http://127.0.0.1:1420/).StatusCode | Out-Null; exit 0 } catch { exit 1 }' >/dev/null 2>&1; then
    break
  fi
  sleep 1
done
if ! "$POWERSHELL" -NoProfile -Command 'try { (Invoke-WebRequest -UseBasicParsing http://127.0.0.1:1420/).StatusCode | Out-Null; exit 0 } catch { exit 1 }' >/dev/null 2>&1; then
  echo "Vite dev server failed to start" >&2
  tail -n 100 "$VITE_LOG" >&2 || true
  exit 1
fi

echo "==> Launching Tauri shell on Windows"
APP_PID="$(
  "$POWERSHELL" -NoProfile -Command '
    $env:POKIE_E2E_REPORT_PATH = "'"$WINDOWS_REPORT_FILE"'"
    $env:POKIE_E2E_TRACE_PATH = "'"$WINDOWS_TRACE_FILE"'"
    $process = Start-Process -FilePath "'"$WINDOWS_EXE"'" -PassThru
    Start-Sleep -Seconds 2
    Write-Output $process.Id
  ' | tr -d '\r'
)"

if [[ -z "$APP_PID" ]]; then
  echo "failed to launch Windows desktop shell" >&2
  exit 1
fi

echo "==> Waiting for desktop smoke pass"
for _ in $(seq 1 60); do
  if ! "$POWERSHELL" -NoProfile -Command "Get-Process -Id ${APP_PID} -ErrorAction SilentlyContinue | Out-Null; if (-not \$?) { exit 1 }" >/dev/null 2>&1; then
    echo "desktop process exited unexpectedly" >&2
    exit 1
  fi

  if [[ -f "$REPORT_FILE" ]]; then
    status="$(python3 - <<'PY' "$REPORT_FILE"
import json, sys
with open(sys.argv[1], 'r', encoding='utf-8') as fh:
    data = json.load(fh)
print(data.get('status', ''))
PY
)"
    if [[ "$status" == "pass" ]]; then
      echo "desktop smoke passed"
      exit 0
    fi
    if [[ "$status" == "fail" ]]; then
      echo "desktop smoke failed" >&2
      cat "$REPORT_FILE" >&2
      exit 1
    fi
  fi
  sleep 1
done

echo "timed out waiting for desktop smoke pass" >&2
echo "Vite log:" >&2
tail -n 100 "$VITE_LOG" >&2 || true
if [[ -f "$REPORT_FILE" ]]; then
  echo "Smoke report:" >&2
  cat "$REPORT_FILE" >&2
fi
if [[ -f "$TRACE_FILE" ]]; then
  echo "Smoke trace:" >&2
  cat "$TRACE_FILE" >&2
fi
exit 1
