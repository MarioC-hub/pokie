# AGENTS.md

This file defines the operating contract for contributors and coding agents working in this repository.

## Repository Objective

Pokie is building a local-first GTO workstation for typical casino poker study.

Committed v1 scope:
- heads-up no-limit Texas Hold'em
- cash-game semantics only
- postflop solving from the start of the flop, turn, or river
- common single-raised-pot and 3-bet-pot study spots

If a game format or workflow is not listed as supported in `architecture.md`, treat it as out of scope.

## Source of Truth

- `architecture.md` is the source of truth for product scope, architecture, crate boundaries, solver strategy, and data contracts.
- `PLAN.md` is the source of truth for task status, dependencies, blockers, and acceptance criteria.
- Code is the source of truth for implementation only after the docs above are aligned with it.

If code, architecture, and plan diverge, fix the docs and implementation in the same change set.

## Mandatory Read Order

Before making any substantial change:

1. read the repository objective and committed v1 scope in `architecture.md`
2. read the relevant sections and task IDs in `PLAN.md`
3. inspect the affected code paths
4. decide whether the work changes architecture, execution state, or both
5. only then implement

Do not code around an unresolved product decision. Resolve it in `architecture.md` and `PLAN.md` first.

## Scope Rules

- Do not silently widen v1 beyond the documented scope.
- Do not treat future-phase notes as permission to implement them now.
- If a task touches preflop solving, multiway postflop, tournament ICM, push-fold charts, PLO, exploitative population models, or partial re-solve, treat it as out of scope unless the docs are updated first.
- Prefer narrowing scope explicitly over building a generic abstraction for an undocumented future format.

## Documentation Update Triggers

Update `architecture.md` when changing:
- supported game types or v1 boundaries
- solver algorithm family or convergence policy
- crate or subsystem boundaries
- canonical `SolveConfig` fields or validation rules
- artifact schemas, compatibility classes, or migration rules
- frontend/backend contracts

Update `PLAN.md` when changing:
- task status
- dependencies or blockers
- milestone ordering
- acceptance criteria
- deferred-work decisions
- the real next step for execution

Update `AGENTS.md` when changing:
- contributor workflow
- decision rules
- required read order
- testing expectations
- the definition of done for plan items

When scope or architecture changes, update `architecture.md` and `PLAN.md` in the same change set.

## Implementation Rules

- Keep solver and poker-domain logic in Rust and separate from UI and Tauri glue.
- Keep UI solve-input state aligned with the canonical config model in `architecture.md`.
- Version persisted formats and make cache behavior explainable.
- Never silently reuse incompatible artifacts.
- If implementation reveals the plan is wrong, fix the plan instead of working around it in code.
- Prefer small, task-scoped changes mapped to a `PLAN.md` task ID.

## Testing Expectations

- Solver, evaluator, math, and game-state changes require automated tests.
- Canonical serialization, hashing, compatibility logic, and migrations require regression coverage.
- UI changes that modify solve inputs must be validated against the canonical config model.
- Toy-game validation must pass before trusting poker-specific solver behavior.
- Do not mark a task `done` if its required tests or validation evidence are missing.

## Done Checklist

A `PLAN.md` item can move to `done` only when:
- the implementation exists
- the stated acceptance criteria are met
- required tests or validation exist and pass
- documentation still matches behavior
- any format, contract, or workflow change is documented
- newly discovered blockers or follow-on work are reflected in `PLAN.md`

## Default Expectation

Leave the repository more explicit, more internally consistent, and easier for the next agent to execute without guessing.
