# Repository Structure

## Purpose

Phase 0 fixes the repository shape before game logic, AI logic, or replay
features are implemented. The top level is organized by product surface rather
than by language.

## Top-Level Ownership

- `cmd/`: runnable entrypoints and thin launch wrappers
- `games/reversi/`: Reversi rule engine, game-master code, and game-owned state
  transforms
- `players/`: AI players, reference bots, and player-specific support code
- `visualizer/`: replay and future watcher clients
- `e2e/`: cross-surface verification assets
- `testdata/`: deterministic fixtures and golden artifacts
- `tools/`: repository-local support scripts
- `docs/`: plans, specs, ADRs, and references

## Allowed Contents

### `cmd/`

- thin binaries or launch adapters
- local developer wrappers
- no rule-engine or AI-search implementation

### `games/reversi/`

- Rust crates for the Reversi game master and core rules
- game-owned transforms between engine state and exported replay state
- no generic `ai-arena` platform contract definitions

### `players/`

- Rust mainline player implementations
- optional Go reference-bot or fixture support code
- no spectator UI code

### `visualizer/`

- artifact-driven replay client code
- Phaser board-rendering code
- lightweight surrounding UI shell and browser tooling
- no server-only runtime or internal-only match-state dependencies

### `e2e/` and `testdata/`

- deterministic fixtures
- golden replay artifacts
- end-to-end verification helpers

### `tools/`

- repository-local verification helpers that are called from root-level
  developer commands and CI workflows
- shell wrappers or small scripts that keep local and CI verification paths
  aligned
- no hidden machine-specific setup that bypasses the repository toolchain

## Layout Rules

- Surface ownership is the primary boundary; language-specific files may exist
  inside a surface, but language is not a top-level organizing principle.
- New top-level directories require a spec update or ADR that explains their
  ownership.
- Shared logic should prefer the narrowest owning surface first. A future shared
  internal package is allowed only when duplication across `games/reversi/` and
  `players/` becomes concrete.

## Phase 0 Completion Contract

Phase 0 is complete when:

- each agreed top-level surface exists in the repository tree
- each surface has a documented owner and allowed contents
- Rust, Go, and browser tooling expectations are recorded in specs
- later phases can add implementation without another top-level reorganization
