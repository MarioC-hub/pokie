# AGENTS.md

This file defines the operating rules for contributors and coding agents working in this repository.

## Source of Truth

- `architecture.md` is the source of truth for repository architecture, component boundaries, algorithm choices, and system design.
- `PLAN.md` is the source of truth for execution state: what is done, what is in progress, what is blocked, and what remains to be built.
- If code, plans, and documentation diverge, update `architecture.md` and `PLAN.md` in the same change set that resolves the divergence.

## Required Read Order

Before making substantial changes:

1. read `architecture.md`
2. read the relevant sections of `PLAN.md`
3. check whether the intended change affects architecture, task status, or both

## Documentation Rules

- Do not treat `architecture.md` as optional background context. It is the design contract for the repo.
- Do not mark a task as `done` in `PLAN.md` unless the implementation and acceptance criteria actually match.
- When introducing a new subsystem, add it to both `architecture.md` and `PLAN.md`.
- When changing a subsystem boundary, algorithm, persistence format, or frontend-backend contract, update `architecture.md`.
- When completing, blocking, re-scoping, or adding work, update `PLAN.md`.

## Component Practices

### Backend and Solver

- Keep solver code in Rust and separate it from UI and Tauri glue code.
- Prefer explicit crate boundaries consistent with `architecture.md`.
- Validate solver behavior on toy games before trusting poker-specific outputs.
- Favor deterministic and versioned serialization for configs and artifacts.
- Do not introduce aggressive cache reuse unless compatibility rules are explicit and documented.

### UI and Product Workflow

- The UI is a workstation for analysis, not a marketing surface.
- Keep solver input state aligned with the canonical config model defined by the backend.
- Build for fast iteration: edit, solve or reuse, inspect, compare.
- Dense interactive surfaces such as range editors should prioritize performance and clarity over generic component reuse.

### Persistence and Caching

- Version persisted formats.
- Make cache reuse explainable.
- Never silently reuse incompatible artifacts.
- Treat exact hits, warm starts, and partial re-solves as distinct behaviors.

### Planning and Execution

- Break work into component-scoped tasks with explicit exit criteria.
- Preserve the task IDs in `PLAN.md` when updating status so history remains traceable.
- If implementation discovers that the current plan is wrong, correct the plan rather than working around it silently.
- Prefer updating the plan before or alongside major implementation changes, not after the fact.

## Good Practices

- Keep changes consistent with documented architecture.
- Make tradeoffs explicit when deviating from the current design.
- Prefer small, reviewable increments over broad undocumented rewrites.
- Keep naming, folder structure, and API boundaries coherent across the repo.
- Add or update tests when changing core logic, solver behavior, config canonicalization, or persistence behavior.

## When to Update Which File

- Update `architecture.md` when the system design changes.
- Update `PLAN.md` when execution status, priorities, or acceptance criteria change.
- Update both when a completed change also changes architecture.

## Default Expectation

Contributors should leave the repository in a more documented and more internally consistent state than they found it.
