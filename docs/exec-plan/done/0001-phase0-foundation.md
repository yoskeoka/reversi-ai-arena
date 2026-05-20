# Phase 0 Foundation
**Execution**: Use `/execute-task` to implement this plan.

## Objective

Complete Phase 0 from [docs/project-plan.md](../../project-plan.md) by fixing
the repository boundary, top-level directory ownership, language/toolchain
choices, and visualizer architecture before any Reversi implementation work
starts.

This plan establishes a product-surface-first repository shape so later phases
can implement the game master, AI players, and replay tooling without needing a
second repository reorganization.

## Confirmed Design Direction

- Top-level layout uses product surfaces rather than language buckets.
- `cmd/` owns runnable entrypoints.
- `games/reversi/` owns the Reversi game master and core rules implementation.
- `players/` owns AI-player implementations and reference bots.
- `visualizer/` owns replay and future watcher clients.
- `e2e/`, `testdata/`, `tools/`, and `docs/` own verification assets, fixtures,
  tooling, and documentation.

Related architectural record:

- [docs/design-decisions/adr.md](../../design-decisions/adr.md)

## Code Changes

- Create the agreed top-level directories:
  - `cmd/`
  - `games/reversi/`
  - `players/`
  - `visualizer/`
  - `e2e/`
  - `testdata/`
- Add minimal scaffold files needed to make the intended ownership visible.
- Add initial toolchain/config roots required by the chosen language split:
  - Rust mainline for the game master and primary AI-player path
  - Go lane only for lightweight reference-bot or fixture support
  - lightweight web stack for the Phaser-based visualizer without React
- Add placeholder README or package metadata where needed so later work can add
  implementation without changing the root layout again.

## Spec Changes

- Update [docs/project-plan.md](../../project-plan.md) during execution to mark
  the Phase 0 milestone complete once the accepted Phase 0 deliverables land.
- Add a repository-structure spec that defines the ownership and allowed
  contents of each top-level directory.
- Add a Phase 0 boundary spec that explains what remains in `ai-arena` versus
  what belongs in `reversi-ai-arena`.
- Add a visualizer architecture spec that fixes Phaser as the board-rendering
  layer and keeps surrounding spectator UI lightweight and framework-minimal.
- Add a language/toolchain policy spec that records:
  - Rust as the mainline game-master and AI-player implementation language
  - Go as an optional lightweight support lane
  - browser tooling expectations for the visualizer

## Design Decisions

- Record the top-level layout decision in `docs/design-decisions/adr.md`.
- If execution reveals additional irreversible choices about shared logic
  placement or visualizer packaging, append follow-up ADR entries instead of
  leaving them implicit in code layout.

## Sub-tasks

- [ ] Define the Phase 0 completion contract in specs before adding scaffolding.
- [ ] [parallel] Write the repository-layout and boundary specs.
- [ ] [parallel] Write the visualizer architecture and language/toolchain specs.
- [ ] [depends on: spec updates] Add the agreed top-level scaffolding and
      minimal config files.
- [ ] [depends on: top-level scaffolding] Update `docs/project-plan.md` to
      check off Phase 0 after the acceptance criteria are satisfied.
- [ ] [depends on: top-level scaffolding] Verify the repo shape, docs links, and
      Phase 0 acceptance criteria locally.

## Parallelism

- The boundary/layout specs and the visualizer/toolchain specs can be drafted in
  parallel once the root shape is fixed.
- Directory scaffolding must wait until the specs define the ownership model.

## Verification

- Confirm the repository tree matches the specified Phase 0 layout.
- Confirm every new scaffold root is covered by a spec.
- Confirm `docs/project-plan.md` marks Phase 0 complete only after the
  repository-layout, boundary, toolchain, and visualizer decisions are landed.
- Run the repo's applicable documentation or workflow checks for the planning
  and execution changes.

## Out of Scope

- Implementing the Reversi rules engine or game master logic
- Implementing the Rust AI-player
- Building a working replay visualizer
- Integrating with real-time spectator APIs
