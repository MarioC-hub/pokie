# Pokie Execution Plan

This file is the execution tracker for the repository.

Use it to track:
- what scope is locked
- what is done
- what is next
- what is blocked
- what is explicitly deferred

Status values:
- `done`
- `in_progress`
- `blocked`
- `todo`

`Depends On` means a task should not start until the listed tasks are complete unless this file is updated first.

## Product Snapshot

Committed v1 product:
- local-first GTO workstation
- heads-up `NLHE`
- cash-game semantics only
- postflop solving from flop-, turn-, or river-rooted states
- common `SRP` and `3BP` study spots

Explicitly deferred beyond v1:
- full preflop solving
- multiway postflop
- tournament `ICM`
- push-fold charts
- `PLO`
- exploitative population models
- subtree-grafting partial re-solve

## Current Critical Path

The next implementation work should follow this order:

1. `SLICE-01` backend correctness slice — `done`
2. `REPO-02` to `REPO-04`
3. `CORE-03` to `CORE-05`
4. `EVAL-02` to `EVAL-03`
5. `TREE-01` to `TREE-05`
6. `SOLVE-05`
7. `V1-01` to `V1-05`
8. `FAST-01` to `FAST-03`
9. `UI-01` to `UI-06`

The immediate next task is `REPO-02`.

## Phase Gates

| Gate | Meaning | Status |
| --- | --- | --- |
| Gate 0 | Docs and v1 scope are locked and synchronized | `done` |
| Gate 1 | Repo scaffolding and canonical config foundation exist | `done` |
| Gate 2 | Toy-game validation proves solver machinery works | `done` |
| Gate 3 | One poker scenario solves end to end and loads in the desktop app | `todo` |
| Gate 4 | Full v1 workflow exists with exact cache hits and a usable explorer | `todo` |

## Task Board

### 0. Repository Governance

| ID | Task | Status | Depends On | Exit Criteria |
| --- | --- | --- | --- | --- |
| GOV-01 | Create `architecture.md` | `done` | - | Repository architecture and product scope are documented. |
| GOV-02 | Create `PLAN.md` | `done` | - | Execution tracker exists with status conventions and critical-path guidance. |
| GOV-03 | Create `AGENTS.md` | `done` | - | Contributor instructions enforce read order, scope discipline, and doc updates. |
| GOV-04 | Synchronize docs when scope or architecture changes | `done` | GOV-01, GOV-02, GOV-03 | `AGENTS.md`, `architecture.md`, and `PLAN.md` currently agree on v1 scope and update rules. |

### 1. Product Definition

| ID | Task | Status | Depends On | Exit Criteria |
| --- | --- | --- | --- | --- |
| PROD-01 | Lock v1 product scope | `done` | GOV-04 | V1 is explicitly documented as heads-up `NLHE` cash-game postflop solving for common `SRP` and `3BP` spots. |
| PROD-02 | Define latency classes and reference hardware | `done` | GOV-04 | `architecture.md` contains numeric latency targets and a reference machine definition. |
| PROD-03 | Define canonical scenario and result surface | `done` | GOV-04 | `architecture.md` documents canonical `SolveConfig` fields, result payload categories, and supported v1 outputs. |
| PROD-04 | Define success metrics and deferred phases | `done` | GOV-04 | Validation gates, performance targets, and explicit post-v1 work are documented. |

### 2. Architecture Contracts

| ID | Task | Status | Depends On | Exit Criteria |
| --- | --- | --- | --- | --- |
| ARCH-01 | Document canonical `SolveConfig` | `done` | PROD-01, PROD-03 | Semantic fields, canonicalization rules, and hashing policy are documented in `architecture.md`. |
| ARCH-02 | Document artifact manifest and compatibility classes | `done` | PROD-03 | Exact-hit, resume, warm-start, and incompatible cases are defined. |
| ARCH-03 | Document repository layout and crate boundaries | `done` | PROD-01 | Backend responsibilities are assigned to named crates with clear exclusions. |
| ARCH-04 | Document minimal backend/frontend contract | `done` | PROD-03 | Command and event categories plus state-ownership rules are documented. |
| ARCH-05 | Define named validation and benchmark fixtures | `done` | ARCH-01, ARCH-02 | Baseline toy-game and poker fixtures are named and documented in-repo for future regression and benchmark work. |

### 3. Repository and Build System

| ID | Task | Status | Depends On | Exit Criteria |
| --- | --- | --- | --- | --- |
| REPO-01 | Initialize Rust workspace | `done` | ARCH-03 | Cargo workspace exists with placeholder crates matching `architecture.md`, and `cargo check --workspace` passes. |
| REPO-02 | Initialize `Tauri v2` desktop shell | `todo` | REPO-01 | Desktop app launches and can call a trivial Rust command through `app-api`. |
| REPO-03 | Initialize `React + Vite + TypeScript` frontend | `todo` | REPO-01 | Frontend builds, hot reload works, and the app renders inside Tauri. |
| REPO-04 | Add formatting, linting, test runners, and CI | `todo` | REPO-01, REPO-02, REPO-03 | Rust and TypeScript checks run locally and in CI. |
| REPO-05 | Add fixture, regression, and benchmark directories | `done` | REPO-01 | Repository contains stable locations for fixtures, regressions, and benchmarks, with seed manifests checked in. |

### 3.5 Staged Backend Slice

| ID | Task | Status | Depends On | Exit Criteria |
| --- | --- | --- | --- | --- |
| SLICE-01 | Land exact river-only first poker correctness slice | `done` | REPO-01, SOLVE-01 | Canonical river config → validated river public tree → CFR solve → exact exploitability → deterministic exact conformance fixture all pass in the Rust workspace. |

### 4. Backend Domain Model

| ID | Task | Status | Depends On | Exit Criteria |
| --- | --- | --- | --- | --- |
| CORE-01 | Implement card encoding and deck utilities | `done` | REPO-01 | Deterministic card representation supports masking, ordering, and parsing with tests. |
| CORE-02 | Implement range representation and parsing | `done` | REPO-01, CORE-01 | Weighted combos, normalization, and import/export work with regression tests. |
| CORE-03 | Implement v1 public-state model | `in_progress` | REPO-01, CORE-01 | Street-rooted postflop state, stacks, pot, board, actor, blinds, and rake are modeled correctly. Current landed slice covers exact river roots and explicitly rejects unsupported broader cases. |
| CORE-04 | Implement scenario config validation | `in_progress` | ARCH-01, REPO-01, CORE-02, CORE-03 | Invalid v1 scenarios are rejected with explicit errors, and supported scenarios canonicalize into `SolveConfig`. Current landed slice validates the exact river-only backend contract. |
| CORE-05 | Implement canonical serialization and hashing | `in_progress` | CORE-04 | Equivalent configs hash identically, fixtures prove stability, and non-semantic metadata does not affect hashes. Current landed slice hashes the river-only canonical config deterministically. |

### 5. Evaluation and Equity

| ID | Task | Status | Depends On | Exit Criteria |
| --- | --- | --- | --- | --- |
| EVAL-01 | Implement hand evaluator | `done` | REPO-01, CORE-01 | Showdown ranking is correct for the exact river-root slice now implemented, with deterministic tests. |
| EVAL-02 | Implement deterministic equity engine | `in_progress` | EVAL-01, CORE-02, CORE-03 | Heads-up all-in equity is reproducible for fixed inputs and test fixtures. Current landed slice includes exact deterministic river showdown comparison, not full all-in equity yet. |
| EVAL-03 | Add evaluator correctness tests | `in_progress` | EVAL-01, EVAL-02 | Tests cover ties, blockers, board edge cases, and invalid inputs. Current landed slice has exact river-showdown regression coverage; broader equity coverage remains. |
| EVAL-04 | Add lookup tables only where benchmarks justify them | `todo` | EVAL-03 | Any precomputed tables have benchmark evidence and documented tradeoffs. |

### 6. Tree Construction

| ID | Task | Status | Depends On | Exit Criteria |
| --- | --- | --- | --- | --- |
| TREE-01 | Model v1 public-state transitions | `in_progress` | CORE-03 | Street progression, stack updates, pot updates, and terminal states are correct for v1 trees. Current landed slice covers the exact river single-bet tree only. |
| TREE-02 | Model information sets | `in_progress` | TREE-01 | Heads-up imperfect-information mapping is correct and covered by tests. Current landed slice covers the exact river infoset mapping used by `solver-core`. |
| TREE-03 | Implement versioned action templates | `in_progress` | ARCH-01, TREE-01 | Supported v1 `SRP` and `3BP` templates generate only legal discrete actions. Current landed slice implements `river_single_bet_v1` only. |
| TREE-04 | Add node-lock representation and validation | `in_progress` | TREE-03 | Supported node locks serialize canonically and reject invalid frequency payloads. Current landed slice represents node locks at the config boundary and rejects them explicitly. |
| TREE-05 | Add tree validation and identity rules | `in_progress` | TREE-02, TREE-03, TREE-04 | Illegal trees are rejected, and tree identity is stable enough for cache compatibility decisions. Current landed slice validates exact river trees and emits deterministic tree identity. |

### 7. Solver Engine

| ID | Task | Status | Depends On | Exit Criteria |
| --- | --- | --- | --- | --- |
| SOLVE-01 | Implement toy-game solver support first | `done` | REPO-01 | Kuhn and Leduc or equivalent toy games solve through the same core regret/strategy machinery intended for poker. |
| SOLVE-02 | Implement first poker solver loop | `done` | SOLVE-01, TREE-05, EVAL-03, CORE-05 | The chosen production algorithm runs against one supported v1 poker tree and produces strategy output. The exact river-only backend slice now solves end to end. |
| SOLVE-03 | Track average strategy and regrets | `done` | SOLVE-02 | Average strategy and regret state persist in a versioned in-memory model suitable for checkpointing. |
| SOLVE-04 | Add convergence metrics | `done` | SOLVE-02 | Best-response or equivalent convergence diagnostics are recorded and queryable. Exact exploitability is now queryable for the bounded river slice. |
| SOLVE-05 | Add checkpointing | `todo` | SOLVE-03, SOLVE-04 | Long solves resume without restarting from zero, and checkpoint compatibility is versioned. |

### 8. First Poker Vertical Slice

| ID | Task | Status | Depends On | Exit Criteria |
| --- | --- | --- | --- | --- |
| V1-01 | Solve one documented baseline poker scenario end to end | `todo` | CORE-05, EVAL-03, TREE-05, SOLVE-01 | A named flop-rooted `SRP` scenario solves from canonical config to versioned artifact. |
| V1-02 | Persist and reload the baseline artifact | `todo` | V1-01, SOLVE-03, SOLVE-05 | Strategy, diagnostics, and lineage metadata load from disk without recomputation. |
| V1-03 | Expose minimal desktop commands and events | `todo` | REPO-02, REPO-03, V1-02 | `validate_config`, `start_solve`, `job_progress`, and `load_result` work through the app boundary. |
| V1-04 | Build a minimal desktop workflow for one spot | `todo` | V1-03 | User can define the baseline spot, start a solve, and inspect root strategy and EV in the app. |
| V1-05 | Add baseline regression fixture and benchmark | `todo` | V1-01 | Named artifact fixture and benchmark numbers exist for the baseline poker scenario. |

### 9. Cache and Reuse

| ID | Task | Status | Depends On | Exit Criteria |
| --- | --- | --- | --- | --- |
| FAST-01 | Build solve artifact store | `todo` | ARCH-02, REPO-01, SOLVE-05 | Strategies, regrets, metadata, and diagnostics persist locally in versioned formats. |
| FAST-02 | Add exact cache reuse | `todo` | FAST-01, CORE-05 | Identical config requests load existing artifacts immediately and report `A: exact cache hit`. |
| FAST-03 | Add conservative warm-start compatibility matching | `todo` | FAST-01, TREE-05, SOLVE-05 | Documented `C`-class warm starts are accepted only when compatibility rules are satisfied and visible to the user. |
| FAST-04 | Add partial re-solve support | `blocked` | FAST-03, QA-02 | Do not start until exact-hit and warm-start behavior are stable, benchmarked, and documented as insufficient. |
| FAST-05 | Stream partial results to the UI | `todo` | SOLVE-04, V1-03 | Users can inspect early summaries before full convergence. |

### 10. Frontend Workflow

| ID | Task | Status | Depends On | Exit Criteria |
| --- | --- | --- | --- | --- |
| UI-01 | Build application shell and navigation | `todo` | REPO-02, REPO-03 | Desktop shell supports the v1 workflow on common screen sizes. |
| UI-02 | Build Scenario Builder | `todo` | UI-01, CORE-04 | Users can define all supported v1 solve inputs and receive validation feedback. |
| UI-03 | Build Range Editor | `todo` | UI-02, CORE-02 | Matrix editing, combo drilldown, and range presets are usable for v1 ranges. |
| UI-04 | Build Tree Template Editor | `todo` | UI-02, TREE-03, TREE-04 | Users can select supported templates and supported node locks without leaving v1 scope. |
| UI-05 | Build Solve Queue | `todo` | UI-01, V1-03 | Job status, progress, cancel, and resume are visible. |
| UI-06 | Build Explorer | `todo` | UI-01, V1-02 | Users can inspect node frequencies, EV, and combo-level strategy details. |
| UI-07 | Build Compare view | `blocked` | UI-06, FAST-02 | Do not start until the explorer and stable artifact loading already exist. |

### 11. Quality and Packaging

| ID | Task | Status | Depends On | Exit Criteria |
| --- | --- | --- | --- | --- |
| QA-01 | Add solver validation against toy games | `done` | SOLVE-01 | Known equilibrium behavior matches documented tolerances before poker solves are trusted. |
| QA-02 | Add regression fixtures for poker scenarios | `todo` | V1-05 | Representative v1 scenarios protect against silent solver and compatibility regressions. |
| QA-03 | Add benchmark suite | `todo` | V1-05, REPO-05 | Runtime, memory, and artifact-size changes are measurable across commits. |
| QA-04 | Add crash recovery tests | `todo` | FAST-01, SOLVE-05 | Interrupted solve jobs recover correctly from persisted checkpoint state. |
| OPS-01 | Add desktop packaging pipeline | `todo` | UI-05, UI-06 | Windows build output can be installed and launched reliably. |
| OPS-02 | Add versioned artifact migration and invalidation strategy | `todo` | FAST-01 | Old cached solves are invalidated or migrated explicitly, never silently reused. |
| OPS-03 | Add optional diagnostics and telemetry mode | `blocked` | QA-03 | Do not start until benchmark coverage exists and a concrete operational need is documented. |

## Explicitly Deferred Work

These items are not part of the current implementation track:
- tournament `ICM`
- push-fold and reshove charts
- multiway postflop solving
- full preflop solving
- non-`NLHE` variants
- subtree-grafting partial re-solve as a near-term feature

If any of these move into scope, update `architecture.md` and this file first.
