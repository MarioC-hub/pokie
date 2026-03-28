# Pokie Execution Plan

This file is the execution tracker for the repository.

Use it to track:
- what has already been completed
- what is currently in progress
- what still needs to be built
- the acceptance criteria for each major component
- the dependencies between tasks

Status values:
- `done`
- `in_progress`
- `blocked`
- `todo`

## Current Focus

The current product direction is a local-first desktop poker analysis workstation with:
- a `Rust` backend and solver core
- a `Tauri v2` desktop shell
- a `React + Vite + TypeScript` UI
- a strong emphasis on fast customization through caching, warm starts, and partial re-solving

## High-Level Milestones

| Area | Goal | Status |
| --- | --- | --- |
| Repository standards | Establish source-of-truth documentation and contribution rules | `done` |
| Product definition | Lock v1 scope, supported game types, and performance targets | `todo` |
| App skeleton | Create desktop shell, frontend shell, and Rust workspace | `todo` |
| Core poker engine | Implement cards, ranges, evaluator, and scenario model | `todo` |
| Solver engine | Implement first equilibrium solver and persistence model | `todo` |
| Fast customization | Add caching, warm starts, and local re-solving | `todo` |
| User workflows | Build builder, editor, queue, explorer, and compare screens | `todo` |
| Validation | Add toy-game validation, regression suites, and performance benchmarks | `todo` |
| Packaging | Produce a usable desktop deliverable | `todo` |

## Task Board

### 0. Repository Governance

| ID | Task | Status | Notes / Exit Criteria |
| --- | --- | --- | --- |
| GOV-01 | Create `architecture.md` | `done` | Repository architecture, algorithms, and rationale are documented. |
| GOV-02 | Create `PLAN.md` | `done` | Execution tracker exists and defines status conventions. |
| GOV-03 | Create `AGENTS.md` | `done` | Contributor instructions point to `architecture.md` as source of truth. |
| GOV-04 | Keep docs synchronized when architecture or priorities change | `todo` | Any architectural change updates `architecture.md` and affected plan items in the same change set. |

### 1. Product Definition

| ID | Task | Status | Notes / Exit Criteria |
| --- | --- | --- | --- |
| PROD-01 | Lock v1 problem scope | `todo` | Decide whether v1 is `push/fold + ICM` only or includes restricted HU postflop solving. |
| PROD-02 | Define latency classes | `todo` | Set explicit targets for instant, interactive, fast re-solve, and full solve operations. |
| PROD-03 | Define supported game configuration model | `todo` | Blinds, ante, stacks, payouts, rake, player count, ranges, board state, and tree template are explicitly specified. |
| PROD-04 | Define success metrics | `todo` | Correctness, runtime, memory, artifact size, and cache hit targets are documented. |

### 2. Repository and Build System

| ID | Task | Status | Notes / Exit Criteria |
| --- | --- | --- | --- |
| REPO-01 | Initialize cargo workspace | `todo` | Workspace compiles with placeholder crates for each major backend concern. |
| REPO-02 | Initialize `Tauri v2` desktop shell | `todo` | Desktop app launches and can call a trivial Rust command. |
| REPO-03 | Initialize `React + Vite + TypeScript` frontend | `todo` | Frontend hot reload works inside the Tauri shell. |
| REPO-04 | Add formatting, linting, and CI checks | `todo` | Rust and TypeScript checks run locally and in CI. |
| REPO-05 | Add test harness and fixture directories | `todo` | Repository has a clear place for unit, regression, and benchmark fixtures. |

### 3. Backend Domain Model

| ID | Task | Status | Notes / Exit Criteria |
| --- | --- | --- | --- |
| CORE-01 | Implement card encoding and deck utilities | `todo` | Deterministic card representation supports fast masking and comparisons. |
| CORE-02 | Implement range representation and parsing | `todo` | Weighted combos, presets, and import/export are supported. |
| CORE-03 | Implement board-state model | `todo` | Street, public cards, dead cards, and blockers are modeled correctly. |
| CORE-04 | Implement scenario config schema | `todo` | All UI-defined solve inputs serialize into a stable config object. |
| CORE-05 | Implement canonical hashing for configs | `todo` | Equivalent configs hash identically and become cache keys. |

### 4. Evaluation and Equity

| ID | Task | Status | Notes / Exit Criteria |
| --- | --- | --- | --- |
| EQ-01 | Implement hand evaluator | `todo` | Showdown ranking is correct for all valid hand classes. |
| EQ-02 | Implement all-in equity engine | `todo` | Exhaustive or sampled equity is deterministic for fixed seeds and settings. |
| EQ-03 | Add precomputed indexing tables where justified | `todo` | Expensive repeated lookups are moved to indexed tables or caches. |
| EQ-04 | Add evaluator correctness tests | `todo` | Tests cover edge cases, ties, blockers, and invalid inputs. |

### 5. Tree Construction

| ID | Task | Status | Notes / Exit Criteria |
| --- | --- | --- | --- |
| TREE-01 | Model public game state transitions | `todo` | Betting rounds, stack changes, pot updates, and terminal states are correct. |
| TREE-02 | Model information sets | `todo` | Private information is grouped correctly for imperfect-information solving. |
| TREE-03 | Implement restricted action abstraction | `todo` | Supported bet sizes and tree templates can be generated from config. |
| TREE-04 | Add node-lock representation | `todo` | Forced frequencies and constraints can be expressed and validated. |
| TREE-05 | Add tree validation | `todo` | Illegal trees, impossible actions, and invalid stacks are rejected before solve. |

### 6. Solver Engine

| ID | Task | Status | Notes / Exit Criteria |
| --- | --- | --- | --- |
| SOLVE-01 | Implement toy-game solver support first | `todo` | Kuhn and Leduc or comparable toy games can be solved for validation. |
| SOLVE-02 | Implement first poker solver loop | `todo` | Initial `CFR`, `CFR+`, or `DCFR` loop runs against the chosen v1 game class. |
| SOLVE-03 | Track average strategy and regrets | `todo` | Strategy persistence supports resumed solving and analysis. |
| SOLVE-04 | Add convergence metrics | `todo` | Exploitability proxy or equivalent stopping metrics are visible. |
| SOLVE-05 | Add checkpointing | `todo` | Long solves can be resumed without restarting from zero. |

### 7. Fast Customization

| ID | Task | Status | Notes / Exit Criteria |
| --- | --- | --- | --- |
| FAST-01 | Build solve artifact store | `todo` | Strategies, regrets, metadata, and diagnostics persist locally. |
| FAST-02 | Add exact cache reuse | `todo` | Identical config requests load existing artifacts immediately. |
| FAST-03 | Add warm-start compatibility matching | `todo` | Nearby configs can reuse prior strategy/regret state safely. |
| FAST-04 | Add partial re-solve support | `todo` | Local node changes do not require rebuilding the full solve when avoidable. |
| FAST-05 | Stream partial results to the UI | `todo` | Users can inspect early outputs before full convergence. |

### 8. Frontend UX

| ID | Task | Status | Notes / Exit Criteria |
| --- | --- | --- | --- |
| UI-01 | Build application shell and navigation | `todo` | Split-pane desktop workflow works on common screen sizes. |
| UI-02 | Build Scenario Builder | `todo` | Users can define all supported solve inputs through the UI. |
| UI-03 | Build Range Editor | `todo` | Matrix editing, combo drilldown, and presets are usable and fast. |
| UI-04 | Build Tree Builder | `todo` | Supported bet templates and node locks can be edited safely. |
| UI-05 | Build Solve Queue | `todo` | Job status, progress, and cancellation are visible. |
| UI-06 | Build Explorer | `todo` | Users can inspect frequencies, EV, and combo-level details. |
| UI-07 | Build Compare view | `todo` | Users can compare two solve artifacts at node and aggregate levels. |

### 9. Validation and Quality

| ID | Task | Status | Notes / Exit Criteria |
| --- | --- | --- | --- |
| QA-01 | Add solver validation against toy games | `todo` | Known equilibrium behavior matches acceptable tolerances. |
| QA-02 | Add regression fixtures for poker scenarios | `todo` | Representative configurations prevent silent behavioral regressions. |
| QA-03 | Add benchmark suite | `todo` | Runtime and memory changes are measurable across commits. |
| QA-04 | Add crash recovery tests | `todo` | Interrupted solve jobs can recover from persisted state. |

### 10. Packaging and Operations

| ID | Task | Status | Notes / Exit Criteria |
| --- | --- | --- | --- |
| OPS-01 | Add desktop packaging pipeline | `todo` | Windows build output can be installed and launched reliably. |
| OPS-02 | Add versioned artifact migration strategy | `todo` | Old cached solves can be invalidated or migrated explicitly. |
| OPS-03 | Add optional diagnostics and telemetry mode | `todo` | Performance debugging can be enabled without polluting normal workflows. |

## Component Checklists

### Backend Components

- Keep crate boundaries narrow and explicit.
- Avoid coupling solver code to Tauri or UI concerns.
- Prefer deterministic serialization and stable hashes.
- Every performance optimization must have a benchmark or rationale.

### Solver Components

- Validate logic on toy games before trusting poker-specific outputs.
- Track correctness and convergence separately from runtime.
- Persist enough state to resume or warm-start solves safely.
- Do not add algorithmic complexity without a measurable latency or quality benefit.

### UI Components

- Treat the UI as a workstation, not a marketing site.
- Optimize for dense interaction, keyboard flow, and comparison workflows.
- Keep expensive visualizations off the main thread where possible.
- Any UI that changes solver input must map cleanly to the canonical config model.

### Data and Persistence

- Store metadata in a queryable form and large artifacts in a compact binary form.
- Version all persisted formats.
- Cache reuse must be explicit and explainable to the user.
- Never silently reuse incompatible artifacts.

## Change Management

When a task changes status:
- update this file in the same change set
- keep the status and exit criteria aligned with reality
- update `architecture.md` as well if the task changes the system design

When a new major component is introduced:
- add it to the task board
- add acceptance criteria
- document how it interacts with the rest of the system
