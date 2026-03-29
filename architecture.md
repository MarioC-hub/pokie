# Pokie Architecture

`architecture.md` is the source of truth for repository design.

If implementation details, plans, or agent instructions conflict with this file, update this file first or in the same change set as the behavior change.

## 1. Product Charter

Pokie is a local-first GTO workstation for serious poker study on a single desktop or laptop.

The long-term product should cover the poker formats most commonly studied by casino and card-room players, especially no-limit hold'em cash games first and tournament study later.

The committed v1 product is narrower on purpose:
- heads-up no-limit Texas Hold'em
- cash-game semantics only
- postflop solving from the start of the flop, turn, or river
- common single-raised-pot and 3-bet-pot study spots
- exact board and exact player ranges supplied by the user

The system should optimize for:
- correctness before breadth
- deterministic behavior where practical
- explicit cache and artifact behavior
- a workstation workflow: define a spot, solve or reuse, inspect, iterate

## 2. Scope Decisions

### 2.1 North-Star Goal

Over time, Pokie should help users study the kinds of hold'em decisions they encounter in live casinos, card rooms, and common online equivalents.

That does not mean v1 should attempt to solve every poker format. The architecture must protect the v1 slice from scope creep.

### 2.2 Committed v1 Support

| Dimension | v1 Support |
| --- | --- |
| Variant | `NLHE` |
| Game mode | `cash` only |
| Active players in the solve | exactly `2` |
| Root street | start of `flop`, `turn`, or `river` |
| Root origin | common `SRP` and `3BP` study spots entered as exact postflop state |
| Board input | exact public board cards consistent with the root street |
| Range input | weighted combo ranges for both players |
| Positions | solver-facing roles are `OOP` and `IP`; seat labels are study metadata |
| Stack model | exact chip counts with a validated effective-stack range |
| Blind and rake model | cash-game blinds plus a simple rake profile |
| Action abstraction | versioned, finite postflop templates with discrete size sets |
| Node locks | supported on documented postflop decision nodes |
| Runtime model | single-machine local execution, no remote service required |

The v1 root is a street-rooted postflop state, not an arbitrary mid-street subtree.

### 2.3 Explicit v1 Non-Goals

The following are out of scope for v1 unless this file and `PLAN.md` are updated first:
- full preflop solving
- multiway postflop solving
- tournament ICM
- push-fold or reshove chart generation
- Pot-Limit Omaha or non-hold'em variants
- exploitative population models
- arbitrary continuous bet sizing
- browser-first or multi-user deployment
- subtree-grafting partial re-solve

### 2.4 Current Implementation Staging

The broader v1 product target remains flop-, turn-, and river-rooted heads-up `NLHE` cash solving.

The first landed backend correctness milestone is intentionally narrower:
- river-rooted only
- exact public board
- exact weighted combo ranges for `OOP` and `IP`
- single-bet / no-raise river template
- exact showdown evaluation
- tabular `CFR` for this slice
- exact exploitability on bounded conformance fixtures

The first landed desktop integration milestone layers a thin workstation shell over the same slice:
- `Tauri v2` packaging
- `React + Vite + TypeScript` frontend
- a DTO-based `app-api` boundary
- synchronous `sample_river_request`, `validate_config`, and `solve_river_spot` commands
- a minimal screen that shows canonical config output, root EV, exploitability, and root infoset strategies

This staging note is an implementation milestone, not a rewrite of the broader v1 scope.

## 3. User Workstation Flows

The product is a workstation, not a solver library with a decorative UI.

The core user loop is:
1. define a study spot
2. validate and canonicalize it
3. load an exact hit, resume a compatible artifact, or start a fresh solve
4. inspect strategy, EV, and combo details
5. tweak ranges or locks and iterate

The required v1 workstation flows are:
- create a postflop cash-game scenario from a template or exact street-rooted state
- run a solve locally with progress and checkpoint visibility
- reopen a saved artifact and inspect results without recomputing
- apply supported node locks and compare the new result against the prior baseline later in the roadmap

## 4. Top-Level System Architecture

The repository is a monorepo with three layers:

1. `frontend`
   - `React + Vite + TypeScript`
   - owns editing workflows, layout state, explorer state, and dense study surfaces

2. `desktop shell`
   - `Tauri v2`
   - packages the application, exposes commands, and forwards progress events

3. `backend core`
   - `Rust` workspace
   - owns poker modeling, tree generation, solving, persistence, and compatibility decisions

The system is local-first. No remote service is required to build, run, or use the v1 product.

## 5. Recommended Repository Layout

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
    config-core/
    equity-core/
    tree-core/
    solver-core/
    solve-cache/
    job-core/
    app-api/
  tests/
    fixtures/
    regression/
    benchmarks/
```

This layout is intentional:
- the UI stays replaceable
- solver logic is isolated from desktop glue
- config and artifact rules have a dedicated home
- job orchestration is separate from solver internals

## 6. Crate Responsibilities

### `poker-core`

Contains:
- card encoding
- rank and suit utilities
- deck masks
- street and board primitives
- public-state helpers shared across v1 postflop roots

Must not contain:
- solver logic
- persistence policy
- UI-facing command code

### `range-core`

Contains:
- range parsing
- combo weighting
- normalization helpers
- range import and export utilities

Must not contain:
- tree generation
- solver updates

### `config-core`

Contains:
- canonical `SolveConfig`
- config validation
- versioned serialization
- canonical hashing rules
- distinction between semantic config and UI metadata

Must not contain:
- tree generation
- solver iteration logic

### `equity-core`

Contains:
- hand evaluator logic
- showdown comparison
- all-in equity routines

Must not contain:
- solver updates
- Tauri glue

### `tree-core`

Contains:
- street-rooted postflop tree generation for v1
- information-set mapping
- versioned action templates
- node-lock modeling
- tree identity calculation

Must not contain:
- SQLite access
- desktop concerns

### `solver-core`

Contains:
- CFR-family implementations
- average-strategy and regret storage
- convergence tracking
- checkpoint state
- best-response or exploitability approximation helpers

Must not contain:
- direct filesystem policy
- Tauri command handlers

### `solve-cache`

Contains:
- artifact manifests
- metadata queries
- compatibility checks
- artifact load and save routines

Must not contain:
- solver updates
- frontend-specific logic

### `job-core`

Contains:
- local solve queue
- cancellation and resume orchestration
- progress snapshot scheduling
- runtime resource limits

Must not contain:
- UI layout state
- poker-domain math

### `app-api`

Contains:
- validated DTOs
- command handlers
- event payloads
- orchestration between `job-core`, `solve-cache`, and domain crates

This is the boundary consumed by Tauri.

## 7. Canonical Data Contracts

### 7.1 `ScenarioRecord` Versus `SolveConfig`

The system must distinguish between:

- `ScenarioRecord`
  - user-facing workspace data such as title, notes, tags, and last-viewed state
  - not part of solver semantics

- `SolveConfig`
  - the canonical semantic input to the solver
  - fully determines validation, tree generation, cache lookup, and reproducibility

UI metadata must never affect the `SolveConfig` hash.

### 7.2 Canonical `SolveConfig`

Every solve request must canonicalize into a versioned `SolveConfig` with these field groups:

- `schema_version`
- `game`
  - `variant = nlhe`
  - `mode = cash`
  - `active_players = 2`
- `root_state`
  - `street`
  - `board`
  - `oop_stack`
  - `ip_stack`
  - `pot_size`
  - `player_to_act`
  - `blind_profile`
  - `rake_profile`
- `ranges`
  - `oop_range`
  - `ip_range`
- `tree_template`
  - `template_id`
  - `template_version`
  - `first_bet_size`
  - `after_check_bet_size`
  - `max_raises_per_street`
  - `allow_all_in`
- `node_locks`
  - zero or more lock entries with a canonical node path and normalized action frequencies
  - current backend slice requires this list to be empty
- `solver_settings`
  - algorithm family
  - iteration or stopping budget
  - checkpoint cadence
  - thread count
  - deterministic seed where applicable

Canonicalization rules:
- materialize semantic defaults before hashing
- reject unsupported fields instead of silently ignoring them
- sort lists and maps into stable order
- include version fields in the serialized form
- exclude non-semantic workspace metadata from the hash

### 7.3 Result and Artifact Model

The result layer must distinguish between:

- manifest metadata
  - `artifact_id`
  - `config_hash`
  - solver, tree, and artifact schema versions
  - creation time
  - parent artifact lineage when reuse occurs
  - compatibility class used to create the artifact

- queryable summaries
  - root frequencies
  - node summaries
  - EV summaries
  - convergence diagnostics

- dense strategy payloads
  - combo-level strategy by node
  - optional regret or checkpoint state

## 8. Tree, Solver, and Runtime Policy

### 8.1 Action Abstraction

The v1 solver uses versioned finite action templates.

Broader v1 rules:
- no arbitrary continuous bet sizes
- each template is versioned and named
- template choice is part of tree identity

Current landed backend slice:
- template id: `river_single_bet_v1`
- root actor may `check` or make one fixed-size bet
- after a root `check`, the opponent may `check` or make one fixed-size bet
- facing a bet, the player may `fold` or `call`
- no raises
- no all-in override policy
- node locks are not yet implemented and must be rejected

The UI may expose a small vetted palette of discrete size overrides later, but v1 should not accept unconstrained float inputs as action sizes.

### 8.2 Solver Strategy

Validation path:
- tabular `CFR` on Kuhn Poker
- tabular `CFR` on Leduc Poker or an equivalent toy imperfect-information game

Production path:
- `DCFR` is the preferred v1 production algorithm for heads-up postflop trees
- `CFR+` is acceptable only if implementation simplicity materially reduces delivery risk
- plain tabular `CFR` is acceptable for the first exact river-only correctness slice

Requirements:
- average strategy accumulation is mandatory
- checkpoint state must be versioned
- convergence diagnostics must be stored and surfaced
- best-response or exploitability approximation is required for validation work

### 8.3 Job Execution Model

The runtime should start simple:
- one active solve job at a time in v1
- queued local jobs are allowed
- jobs are cancellable
- checkpoints are periodic and resumable
- progress snapshots stream to the UI

This keeps resource contention understandable on consumer hardware.

## 9. Storage and Compatibility

### 9.1 Storage Layout

Use:
- `SQLite` for manifests, presets, recent scenarios, and job metadata
- versioned binary artifacts for dense strategy and checkpoint payloads
- `zstd` compression where artifacts are large enough to justify it

### 9.2 Compatibility Classes

The system must label reuse explicitly:

- `A: exact cache hit`
  - same semantic config and same versioned solver/tree/artifact contract

- `B: resume or refine`
  - same config hash and same artifact lineage, typically for checkpoint resume or stricter stopping criteria

- `C: compatible warm start`
  - same root state and tree identity with documented safe reuse rules
  - v1 may implement this conservatively or defer it until after the first end-to-end slice

- `D: incompatible`
  - solve from scratch

Subtree-local partial re-solve is not a v1 commitment.

### 9.3 Versioning Rules

Every persisted structure must carry explicit versioning:
- config schema version
- tree template version
- solver-core version
- artifact format version

Never silently migrate or silently reuse incompatible artifacts.

## 10. Backend-to-Frontend Contract

The backend owns:
- config validation and canonicalization
- tree generation
- compatibility decisions
- artifact loading rules
- progress and result payloads

The frontend owns:
- editing workflows
- layout state
- current selections
- filters and visualization state

The minimal command surface is:
- `validate_config`
- `start_solve`
- `cancel_solve`
- `resume_solve`
- `list_artifacts`
- `load_result`
- `save_scenario`
- `load_scenario`

The minimal event surface is:
- `job_queued`
- `job_started`
- `job_progress`
- `job_checkpointed`
- `job_completed`
- `job_failed`
- `artifact_reused`

The current landed desktop slice is intentionally narrower than the full v1 command surface. Today the shell exposes:
- `sample_river_request`
- `validate_config`
- `solve_river_spot`

These commands operate only on the exact river-only backend slice documented in section 2.4.

## 11. Validation and Performance Gates

### 11.1 Correctness Gates

Before trusting poker-specific results:
- evaluator correctness tests must pass on exhaustive or fixture-driven hand classes
- `SolveConfig` hashing must be stable across regression fixtures
- Kuhn Poker validation must converge within documented tolerance
- Leduc or equivalent toy-game validation must converge within documented tolerance
- first poker regression scenarios must reproduce stable strategy and EV summaries within documented tolerances

Validation work must prefer exact oracles over pattern-matching heuristics:
- exact chance enumeration where the game is small enough
- exact terminal utility checks
- exact best-response or exploitability checks for toy games
- exact step-state regression fixtures for deterministic CFR updates
- exact bounded river-poker equilibrium fixtures for the first poker slice

Named baseline conformance fixtures:
- `tests/fixtures/conformance/kuhn_reference_equilibrium.fixture`
  - representative Kuhn Poker equilibrium profile from the documented equilibrium family
  - exact expected value for `P0`: `-1/18`
  - exact exploitability target: `nash_conv <= 1e-12`
- `tests/fixtures/conformance/skewed_matrix_cfr_iteration_1.fixture`
  - exact first-iteration regret and average-strategy state for a hidden-action `2x2` zero-sum matrix game
  - protects regret-sign, reach-weight, and averaging logic
- `tests/fixtures/conformance/river_polarized_equilibrium.fixture`
  - exact bounded river poker equilibrium for the first production poker slice
  - exact expected value for `OOP`: `2.5`
  - exact exploitability target: `nash_conv <= 1e-12`
- `crates/solver-core/tests/conformance.rs`
  - exact Kuhn game-shape, terminal-utility, equilibrium-value, and exploitability checks
  - exact first-iteration CFR regression for a hidden-action matrix game
- `crates/solver-core/tests/poker_conformance.rs`
  - exact river poker game-shape checks
  - exact river equilibrium and exploitability checks
  - exact first-iteration CFR regression for the river poker slice

### 11.2 Reference Hardware

Reference planning target:
- recent 8-core desktop or laptop CPU
- 32 GB RAM
- NVMe SSD
- no GPU requirement

### 11.3 Latency and Resource Targets

These are planning targets for v1:
- pure UI interactions: under `100 ms`
- artifact reopen on an exact hit: under `1 s`
- first progress update after solve start: under `5 s`
- baseline flop-rooted SRP solve writes its first checkpoint within `60 s`
- baseline flop-rooted SRP solve produces a usable artifact within `10 min`
- baseline v1 solve memory stays under `8 GB`

These targets may be revised only by updating this file and `PLAN.md` together.

## 12. Expected Evolution Path

The intended expansion order is:

1. ship a correct heads-up postflop cash-game workstation
2. add stronger artifact reuse and study ergonomics around that slice
3. add tournament push-fold and ICM as a separate product phase
4. revisit wider postflop coverage, preflop solving, and multiway only after earlier phases are stable

Future phases must not weaken the clarity of the v1 contract.

## 13. Operational Rules

The repository must follow these rules:
- architecture changes require an update to `architecture.md`
- execution-state changes require an update to `PLAN.md`
- unsupported formats remain out of scope by default
- correctness validation precedes optimization work
- persistence compatibility changes require version bumps and documented invalidation behavior
