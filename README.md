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

## Desktop shell

Install frontend dependencies from `desktop/`:

```bash
cd desktop
npm install
```

Build the React frontend:

```bash
npm run build
```

Run the desktop shell on a Windows workstation:

```bash
npm run tauri:dev
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

### Cross-check used in this repository

The Tauri shell was compile-checked in this repo with:

```bash
cargo check --manifest-path desktop/src-tauri/Cargo.toml --target x86_64-pc-windows-gnu
```
