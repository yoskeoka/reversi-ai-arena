# Phase 1/2 04 WASM AI Player Fixtures
**Execution**: Use `/execute-task` to implement this plan.

## Objective

Deliver Phase 2 by implementing the Rust AI-player mainline as a WASM-targeted
consumer of the Phase 1 game and protocol surfaces, while adding the fixture
build/cache strategy required to test that path repeatedly without rebuilding
every player binary inside each test case.

This plan takes over where Phase 1 stops: the minimal runner-driving fixture
players already exist, and this phase upgrades the player surface into a real,
non-trivial AI-player line while preserving fixture-build caching for repeated
WASM verification.

## Confirmed Direction

- The primary AI-player is Rust and ships through the `wasm-wasi` runtime path.
- Test cases should treat AI-player builds as pre-built fixtures via
  `assume()`-style setup and reusable cached artifacts; repeated per-test
  rebuilds are not an acceptable steady-state strategy.
- The Reversi game master binary remains a separate build artifact and must be
  present for the Phase 2 end-to-end path.
- Phase 1 already owns the minimum fixture-player surface needed to drive the
  runner, so Phase 2 should not spend its main effort on another trivial bot.
  Its responsibility is to deliver a meaningfully thinking AI-player on top of
  that base.
- A lightweight Go reference bot may exist, but it is a support lane rather
  than the main competitive implementation path.
- `reversi-adventure` is the logic reference for the Reversi engine and AI
  techniques, but code is copied and adapted locally rather than shared as a
  live library boundary.

## Code Changes

- Implement the Rust AI-player crate(s) under `players/`, targeting WASM/WASI.
- Add reusable fixture-build helpers that:
  - build Rust WASM players once per verification scope
  - cache the produced `.wasm` and manifest artifacts
  - expose stable fixture paths to e2e and integration tests
- Evolve beyond the Phase 1 fixture-player baseline by implementing search,
  evaluation, or other non-trivial move-selection behavior derived from the
  `reversi-adventure` reference line.
- Add a minimal Go reference bot or helper only if needed for comparison lanes,
  protocol examples, or non-Rust fixture coverage.
- Add e2e coverage that runs the tagged `arena-runner` host with:
  - Rust WASM AI player fixtures
  - the Reversi game master
  - deterministic opponent/setup fixtures

## Spec Changes

- Add a Reversi AI-player spec covering:
  - WASM runtime assumptions
  - manifest layout and metadata
  - fixture-build/cache policy for tests
  - the role of the optional Go reference bot
- Add or extend a verification-assets spec to document:
  - where cached build artifacts may live during tests
  - when fixture artifacts may be regenerated
  - which tests own Rust-WASM versus Go reference coverage
- Update [docs/project-plan.md](../../project-plan.md) during execution to mark
  Phase 2 complete only after the Rust WASM lane is continuously verifiable.

## Design Decisions

- Treat build-artifact caching as part of the verification architecture, not as
  an incidental test optimization.
- Keep the Go support lane minimal and explicitly subordinate to the Rust
  mainline.
- Reuse Reversi rule and AI ideas from `reversi-adventure`, but keep crate
  ownership and protocol integration fully local to this repository.
- Treat the Phase 1 fixture players as scaffolding. Phase 2's acceptance bar is
  a player with meaningful decision logic, not merely another completion-only
  scripted or first-legal-move bot.

## Sub-tasks

- [ ] Define the Phase 2 AI-player and fixture-cache contract in specs before
      implementing the player.
- [ ] [parallel] Port or adapt the Reversi AI logic needed for the initial Rust
      WASM player from `reversi-adventure` references.
- [ ] [parallel] Add reusable build/cache helpers for Rust WASM test fixtures
      and manifests.
- [ ] [depends on: Rust AI logic] Define how the Phase 2 player exceeds the
      Phase 1 fixture bots in move selection quality and verification coverage.
- [ ] [parallel] Add the optional Go reference bot only if a concrete fixture or
      protocol-coverage need remains after the Rust lane is defined.
- [ ] [depends on: Rust AI logic, build/cache helpers] Add unit/integration/e2e
      tests that consume cached WASM fixtures instead of rebuilding inside every
      test case.
- [ ] [depends on: green WASM e2e path] Update `docs/project-plan.md` to mark
      Phase 2 complete.

## Parallelism

- Rust AI logic and fixture-cache helpers can proceed in parallel once the Phase
  2 spec and protocol crate are in place.
- The Go reference bot should remain an optional side stream and must not block
  the Rust WASM mainline unless a test fixture explicitly depends on it.

## Verification

- Rust unit tests cover the AI-player logic that was adapted for the WASM lane.
- Fixture helpers can build and reuse WASM artifacts across multiple tests.
- Tagged-runner e2e tests complete with the Rust WASM AI player.
- CI runs the Rust WASM verification lane without hidden local-only setup.

## Out of Scope

- Replay visualizer implementation
- Real-time watcher integration
- Stronger post-Phase-2 tuning or advanced analysis overlays
