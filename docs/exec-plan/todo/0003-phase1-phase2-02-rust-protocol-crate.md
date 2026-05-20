# Phase 1/2 02 Rust Protocol Crate
**Execution**: Use `/execute-task` to implement this plan.

## Objective

Create the Rust compatibility crate(s) that let `reversi-ai-arena` implement
the public `ai-arena` game master and AI-player transport contracts without
depending on a non-existent upstream Rust SDK.

This plan is the contract prerequisite for both Phase 1 and Phase 2 in
[docs/project-plan.md](../../project-plan.md). It must land before the actual
game master and WASM AI-player implementations so those later plans can focus
on Reversi behavior instead of transport re-design.

## Confirmed Direction

- `ai-arena` remains the contract authority; this repository only creates Rust
  compatibility code for consuming that public contract.
- The transport remains stdio JSON-RPC 2.0 over NDJSON.
- The crate surface should cover both the trusted game-master sidecar path and
  the AI-player sidecar path needed by WASM players.
- The compatibility layer must avoid importing or re-creating platform-internal
  `ai-arena` packages.

## Code Changes

- Add one or more Rust crates under repository-owned surfaces, likely:
  - `games/reversi/` for game-master-facing protocol support
  - `players/` or a narrow shared Rust crate for AI-player-facing protocol DTOs
- Implement public DTOs, method names, request/response types, and NDJSON I/O
  helpers that match the documented `ai-arena` contract.
- Add tests for protocol encoding/decoding, method dispatch shape, and metadata
  compatibility helpers.
- If shared protocol code is needed by both the game master and the Rust
  player, place it in the narrowest reusable crate that still preserves the
  product-surface-first layout.

## Spec Changes

- Add a spec that defines the Rust-side compatibility layer for:
  - game-master transport methods
  - AI-player transport methods
  - DTO mapping and serialization expectations
  - what is mirrored locally versus referenced from `ai-arena`
- Update [docs/specs/platform-boundary.md](../../specs/platform-boundary.md) to
  record the allowed dependency direction and the fact that the Rust protocol
  layer exists because no official Rust package is available yet.
- If the shared crate placement creates a reusable internal boundary, update
  [docs/specs/repository-structure.md](../../specs/repository-structure.md) to
  describe that ownership explicitly.

## Design Decisions

- Mirror the documented contract shape, not the current internal implementation
  shape of `ai-arena`.
- Prefer shared DTO/transport helpers over copy-pasting ad hoc JSON handling
  into both the game master and the AI-player crates.
- Keep the crate boundary small enough that future upstream Rust SDK adoption,
  if it ever appears, can replace this layer without rewriting Reversi logic.

## Sub-tasks

- [ ] Define the Rust protocol-crate responsibility in specs before adding
      transport code.
- [ ] [parallel] Implement game-master-facing DTOs and NDJSON transport helpers.
- [ ] [parallel] Implement AI-player-facing DTOs and NDJSON transport helpers.
- [ ] [depends on: game-master DTOs, AI-player DTOs] Consolidate shared serde
      or framing helpers into the narrowest stable crate boundary.
- [ ] [depends on: shared helpers] Add unit tests for request/response framing,
      metadata compatibility fields, and malformed-payload handling.
- [ ] [depends on: tested crate surface] Document how later plans consume the
      protocol crate from `games/reversi/` and `players/`.

## Parallelism

- The game-master and AI-player DTO surfaces can be designed in parallel after
  the contract spec is fixed.
- Shared helper extraction must wait until both surfaces are concrete enough to
  avoid premature abstraction.

## Verification

- Rust unit tests cover DTO serialization and NDJSON framing.
- The compatibility layer exposes the method names and metadata fields expected
  by the `ai-arena` public documentation.
- No crate in this repo depends on `ai-arena` internal packages or a local
  workspace checkout contract.

## Out of Scope

- Reversi rules implementation
- Tagged `arena-runner` e2e execution
- WASM AI search/evaluation logic
- Replay visualizer work
