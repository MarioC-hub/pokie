# Pokie Architecture

`architecture.md` is the source of truth for the repository design.

If implementation details, plans, or agent instructions conflict with this file, this file should be updated first or in the same change set as the code that changes behavior.

## 1. System Goals

Pokie is intended to be a local-first poker analysis workstation that can evolve from a restricted equilibrium tool into a more capable solver platform.

Primary goals:
- provide a usable desktop UI for configuring and exploring poker solves
- make customization workflows fast enough for iterative analysis
- keep heavy computation in `Rust`
- preserve a clean path from a desktop-only tool to a reusable backend core

Non-goals for early versions:
- browser-first multi-user deployment
- unconstrained full-game no-limit solving
- premature algorithmic complexity before restricted games are correct and validated

## 2. Product Principles

The system should optimize for:
- correctness before complexity
- local responsiveness
- deterministic behavior where practical
- explicit cache behavior
- modular backend crates

The product should feel like a workstation:
- edit a scenario
- run or reuse a solve
- inspect the result
- compare it with a nearby variant
- iterate quickly

## 3. Top-Level Architecture

The repository should be organized as a monorepo with three layers:

1. `frontend`
The React application responsible for configuration, job control, exploration, and comparison workflows.

2. `desktop shell`
The Tauri host that packages the app, exposes Rust commands, and forwards progress events between the frontend and backend.

3. `backend core`
A Rust cargo workspace containing the poker model, evaluator, tree builder, solver engine, persistence, and application API.

## 4. Recommended Repository Layout

```text
pokie/
  AGENTS.md
  PLAN.md
  architecture.md
  README.md
  desktop/
    package.json
    src/
    src-tauri/
  crates/
    poker-core/
    range-core/
    equity-core/
    tree-core/
    solver-core/
    solve-cache/
    app-api/
  tests/
    fixtures/
    regression/
    benchmarks/
```

This layout is intentional:
- the UI remains replaceable without disturbing solver logic
- the backend can later be exposed via CLI or HTTP without rewriting core crates
- tests and fixtures are visible at the repository level

## 5. Technology Choices

### Backend

- Language: `Rust`
- Reasoning:
  - strong performance profile for solver workloads
  - memory safety for long-running computation
  - good fit for deterministic domain modeling and binary artifact formats

### Desktop Shell

- Framework: `Tauri v2`
- Reasoning:
  - minimal overhead relative to heavier desktop containers
  - good interoperability with Rust
  - straightforward packaging for local desktop use

### Frontend

- Framework: `React + Vite + TypeScript`
- Reasoning:
  - fastest path to a dense, highly interactive workstation UI
  - strong ecosystem for routing, async job control, and custom editors
  - less implementation risk than a Rust-native UI for a tool with complex editors

Recommended UI support libraries:
- `TanStack Router` for typed navigation and scenario-driven URL state
- `TanStack Query` for long-running job orchestration and cache-aware data fetching
- `shadcn/ui` for adaptable UI primitives
- `Tailwind CSS` for rapid interface iteration

## 6. Core Domain Model

The backend should model poker in layers.

### 6.1 Card and Range Layer

Responsibilities:
- card encoding
- suit and rank utilities
- deck masks
- combo enumeration
- weighted range representation
- range parsing and normalization

This layer must be deterministic and allocation-conscious because it is used by every higher-level subsystem.

### 6.2 Public Game State Layer

Responsibilities:
- blind and ante structure
- player stacks
- pot construction
- betting round progression
- public board state
- terminal state detection

This layer models what is publicly observable and should stay independent from any specific solving algorithm.

### 6.3 Information-Set Layer

Responsibilities:
- grouping states by private information visibility
- representing decision points for imperfect-information solving
- mapping action histories and private holdings into solver-facing information sets

This layer is essential because poker is not a perfect-information search problem.

### 6.4 Tree Construction Layer

Responsibilities:
- generate restricted action trees from a configuration
- enforce stack legality and action legality
- support node locks and user-imposed constraints

The tree must be generated from a canonical config so that equivalent user inputs map to equivalent solve requests.

## 7. Solver Architecture

The initial architecture should support restricted game solving first, then richer re-solving workflows later.

### 7.1 Why Not Start With Exact Full-Game Solving

Full no-limit poker is too large to solve directly in an unrestricted form for an early local-first product.

The right approach is:
- define a restricted game
- solve it well
- make restricted changes fast
- expand capability gradually

### 7.2 Algorithm Families

The main algorithm family should be `Counterfactual Regret Minimization` and its practical variants.

Recommended progression:
- use simple tabular `CFR` for toy games and small validation cases
- use `CFR+` or `DCFR` for restricted poker trees in the first serious solver
- add sampling or more advanced re-solving only when profiling shows a real need

### 7.3 Why CFR-Family Algorithms

Reasoning:
- they are the dominant practical family for large imperfect-information poker games
- they fit extensive-form games with information sets
- they converge to approximate Nash equilibria in the settings that matter here
- they are easier to implement incrementally than more exotic approaches

### 7.4 Proposed Initial Algorithms

#### Toy-Game Validation

Use tabular `CFR` on:
- Kuhn Poker
- Leduc Poker or a similarly small imperfect-information game

Purpose:
- verify information-set indexing
- verify regret updates
- verify average strategy accumulation
- catch structural bugs before poker-specific abstractions obscure them

#### Restricted Poker Solving

Use `CFR+` or `DCFR` on the chosen v1 restricted game.

Reasoning:
- better practical convergence than naïve CFR
- still understandable and maintainable
- fits a local-first desktop analysis product

`DCFR` is attractive if faster practical convergence outweighs slightly more implementation detail.
`CFR+` is attractive if implementation simplicity and conventional solver lineage are prioritized.

### 7.5 What the Solver Must Persist

At minimum:
- config hash
- algorithm version
- tree version
- average strategy
- regret state
- convergence diagnostics
- artifact metadata

Persistence is required not only for resumability but for fast customization via warm starts and compatible reuse.

## 8. Fast Customization Design

Fast customization is a product requirement, not a later optimization.

The system should support four latency classes:

- `instant`
  - pure presentation changes
  - view filters
  - compare toggles

- `interactive`
  - cached result switching
  - loading nearby variants

- `fast re-solve`
  - node lock changes
  - small range changes
  - small tree changes that preserve enough structure

- `background solve`
  - large structural changes
  - major abstraction changes
  - changes that invalidate prior artifacts

### 8.1 Required Mechanisms

To achieve this, the system needs:
- canonical config hashing
- exact artifact caching
- compatibility-aware warm starts
- partial re-solving or subtree-local re-solving
- streamed partial results

### 8.2 Config Hashing

Every solve request must serialize into a canonical `SolveConfig`.

This means:
- no hidden defaults that can alter semantics silently
- stable ordering of fields and lists
- versioned serialization
- equivalent configs produce identical hashes

The config hash becomes the primary key for:
- cache lookup
- comparison baselines
- result loading
- reproducibility

### 8.3 Warm Starts

Warm starts should be allowed only when:
- the tree structure is compatible enough
- the action abstraction is compatible enough
- private/public state mapping remains coherent
- the solver version declares compatibility

Warm starts should not be hidden from the user. The UI should expose whether a solve is:
- exact cache hit
- warm-started from a nearby solve
- partially re-solved
- solved from scratch

### 8.4 Partial Re-Solving

Partial re-solving is appropriate for:
- local node locks
- limited action availability changes
- some range changes

It is not appropriate when:
- the overall tree topology changes substantially
- the abstraction changes invalidate action mapping
- solver invariants are no longer preserved

This should be implemented conservatively at first. Incorrect reuse is worse than a slower recomputation.

## 9. Persistence and Storage

The storage architecture should separate metadata from large artifacts.

### 9.1 Metadata Store

Use `SQLite` for:
- solve jobs
- configs
- presets
- recent scenarios
- artifact manifests
- compatibility metadata

### 9.2 Artifact Store

Use versioned binary files for:
- average strategies
- regrets
- node summaries
- diagnostics snapshots

Compression such as `zstd` is appropriate for large artifacts.

### 9.3 Versioning

Every persisted structure must be versioned:
- config schema version
- tree generation version
- solver algorithm version
- artifact format version

This avoids silent corruption or invalid reuse after architectural changes.

## 10. UI Architecture

The UI should be structured around workflows rather than around backend entities.

### 10.1 Main Screens

- `Scenario Builder`
  - define blinds, stacks, ante, payouts, rake, ranges, board, and tree template

- `Range Editor`
  - edit matrices, combos, weights, and presets

- `Tree Builder`
  - configure allowed actions and node locks

- `Solve Queue`
  - launch, cancel, resume, and inspect jobs

- `Explorer`
  - inspect action frequencies, EV, and combo-level details

- `Compare`
  - compare two solve artifacts or nearby scenario variants

### 10.2 UI Rendering Guidelines

The UI should prefer:
- custom `canvas` rendering for dense matrices
- virtualized trees and tables for large result sets
- asynchronous loading boundaries for large artifacts

The UI should avoid:
- large naïve DOM grids for matrix-heavy interactions
- hidden defaults that diverge from the canonical config
- mixing transient view state with persisted solve input state

## 11. Backend-to-Frontend API

The application API should be thin and explicit.

Command categories:
- `validate_config`
- `start_solve`
- `cancel_solve`
- `resume_solve`
- `list_solves`
- `load_result`
- `compare_results`
- `save_preset`
- `load_preset`

Event categories:
- `job_started`
- `job_progress`
- `job_checkpointed`
- `job_completed`
- `job_failed`

The backend should own:
- validation
- config canonicalization
- cache compatibility decisions
- artifact loading rules

The frontend should own:
- editing workflows
- layout state
- selection state
- visualization state

## 12. Crate Responsibilities

### `poker-core`

Contains:
- cards
- ranks and suits
- deck masks
- board helpers
- public-state primitives

Must not contain:
- solver logic
- UI-facing command code

### `range-core`

Contains:
- range parsing
- combo weighting
- presets
- normalization helpers

Must not contain:
- tree generation
- persistence orchestration

### `equity-core`

Contains:
- evaluator logic
- showdown computation
- all-in equity routines

Must not contain:
- solver updates
- Tauri glue

### `tree-core`

Contains:
- game-tree generation
- information-set mapping
- action abstraction
- node-lock modeling

Must not contain:
- persistence policy
- frontend concerns

### `solver-core`

Contains:
- CFR-family implementations
- strategy and regret storage
- convergence tracking
- checkpoint support

Must not contain:
- direct SQLite code
- Tauri command handlers

### `solve-cache`

Contains:
- config hashing
- artifact manifests
- compatibility logic
- storage loading and saving

Must not contain:
- domain-specific UI logic

### `app-api`

Contains:
- validated DTOs
- command handlers
- application service orchestration

May depend on:
- domain crates
- cache crate

Should be the boundary consumed by Tauri.

## 13. Validation Strategy

Validation should happen in ascending realism.

### 13.1 Unit Validation

Test:
- card encoding
- evaluator correctness
- range normalization
- config hashing stability

### 13.2 Toy-Game Validation

Test:
- information-set construction
- regret update correctness
- average strategy accumulation

### 13.3 Restricted Poker Regression

Test:
- stable outputs on representative scenarios
- node-lock handling
- cache and warm-start compatibility behavior

### 13.4 Performance Validation

Measure:
- solve runtime
- memory usage
- artifact size
- cache hit rate
- warm-start speedup

## 14. Operational Rules

The repository should follow these architectural rules:

- architecture changes require an update to `architecture.md`
- task status changes require an update to `PLAN.md`
- implementation should match documented crate boundaries
- incompatible persistence changes must increment versioning metadata
- correctness validation should precede optimization work

## 15. Expected Evolution Path

The expected system evolution is:

1. establish repository standards and scaffolding
2. ship a correct restricted-game solver
3. add a usable desktop workflow around it
4. make customization fast via caching and reuse
5. broaden game support only after the above is stable

This ordering is intentional. The system should become more capable without losing correctness, explainability, or architectural clarity.
