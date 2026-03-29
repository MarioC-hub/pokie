# pokie

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

The first implemented poker slice is intentionally narrow:
- heads-up NLHE cash semantics
- river-rooted spots only
- exact weighted combo ranges
- single-bet / no-raise river tree template
- exact showdown evaluation and exact exploitability diagnostics
