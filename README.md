# pokie

## Current implemented slice

Pokie currently ships an exact, intentionally narrow backend-and-desktop slice:
- heads-up NLHE cash semantics
- river-rooted spots only
- exact weighted combo ranges
- single-bet / no-raise river tree template
- exact showdown evaluation and exact exploitability diagnostics
- a thin Tauri + React desktop shell over `app-api`

## Solver conformance suite

Run the deterministic conformance suite with:

```bash
./scripts/run_solver_conformance.sh -- --nocapture
```

The current suite covers:
- exact Kuhn Poker game-shape and terminal-payoff checks
- exact Kuhn equilibrium value and exploitability checks
- exact best-response oracle checks against exploitable profiles
- exact first-iteration CFR regression on a hidden-action matrix game
- exact river poker tree-shape and terminal-utility checks for the first poker slice
- exact river poker equilibrium and exploitability checks on a bounded polarized fixture
- exact first-iteration CFR regression for the river poker slice

## Repository checks

Install the Rust components and targets used by the local runners:

```bash
rustup component add rustfmt clippy
rustup target add x86_64-unknown-linux-musl x86_64-pc-windows-gnu
```

Run the Rust formatter, lints, workspace tests, and the named conformance runner:

```bash
./scripts/run_rust_checks.sh
```

Run the frontend formatter, lints, and production build:

```bash
./scripts/run_frontend_checks.sh
```

Run the Tauri shell formatting and Windows-target compile check:

```bash
./scripts/run_desktop_shell_checks.sh
```

Run the full local check suite used by this repository:

```bash
./scripts/run_repo_checks.sh
```

## Desktop shell

Install frontend dependencies from `desktop/`:

```bash
cd desktop
npm ci
```

Build the React frontend:

```bash
npm run build
```

Run the desktop shell on a Windows workstation:

```bash
npm run tauri:dev
```

Run the Windows-host desktop smoke check used in this repository:

```bash
./scripts/run_desktop_e2e_smoke.sh
```

The current desktop surface is intentionally small:
- `sample_river_request`
- `validate_config`
- `solve_river_spot`

It renders:
- canonical config output
- root EV
- exploitability / NashConv
- root infoset strategies

### Validation used in this repository

The repository CI mirrors the local runners with:

```bash
./scripts/run_rust_checks.sh
./scripts/run_frontend_checks.sh
./scripts/run_desktop_shell_checks.sh
```

For the current exact river desktop slice, the Windows-host smoke validation remains:

```bash
./scripts/run_desktop_e2e_smoke.sh
```
