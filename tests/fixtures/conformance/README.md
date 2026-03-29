# Conformance Fixtures

These fixtures are intentionally exact and small.

They protect four classes of failures:
- analytical equilibrium drift in Kuhn Poker
- regret-update sign or weighting bugs in tabular CFR
- analytical river-poker equilibrium drift in the first exact poker slice
- accidental fixture churn through an explicit manifest

The suite avoids pattern-matching heuristics and instead uses:
- exact chance enumeration
- exact terminal utilities
- exact pure-strategy best responses on the toy games in scope
- exact exploitability checks on bounded river poker fixtures
- fixed iteration-count regression for deterministic CFR state
