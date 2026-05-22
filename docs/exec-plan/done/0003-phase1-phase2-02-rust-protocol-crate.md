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
- Shared transport/framing code should live in a neutral internal crate rather
  than under `games/reversi/` or `players/`, because neither product surface
  should own the generic `ai-arena` contract by itself.
- Reversi-specific payload DTOs remain owned by `games/reversi/` and are reused
  by both the game master and Rust AI-player implementations.

## Code Changes

- Add a neutral shared Rust crate, likely under `internal/aiarena-protocol/`,
  that owns:
  - JSON-RPC 2.0 envelope types
  - NDJSON framing helpers
  - game-master RPC schema encode/decode
  - AI-player RPC schema encode/decode
- Keep Reversi-specific request/response payload DTOs in `games/reversi/` so
  the game master and the Rust AI player share the same game-owned types rather
  than duplicating board/action/result payload definitions across surfaces.
- Implement public DTOs, method names, request/response types, and NDJSON I/O
  helpers that match the documented `ai-arena` contract.
- Add tests for protocol encoding/decoding, method dispatch shape, and metadata
  compatibility helpers.
- Wire both `games/reversi/` and the Rust AI-player crate under `players/` to
  consume the shared transport crate plus the Reversi-owned payload DTOs.
- Update the root `Cargo.toml` workspace membership and any required path
  dependencies so the shared crate is built and tested as part of the repo's
  standard Rust workspace.

## Spec Changes

- Add a spec that defines the Rust-side compatibility layer for:
  - the neutral internal crate boundary
  - shared JSON-RPC / NDJSON transport helpers
  - game-master transport methods
  - AI-player transport methods
  - DTO mapping and serialization expectations
  - what is mirrored locally versus referenced from `ai-arena`
- Update [docs/specs/platform-boundary.md](../../specs/platform-boundary.md) to
  record the allowed dependency direction and the fact that the Rust protocol
  layer exists because no official Rust package is available yet.
- Update [docs/specs/repository-structure.md](../../specs/repository-structure.md)
  to describe the new `internal/` ownership boundary explicitly if that
  top-level surface is introduced for shared Rust support code.
- Update [docs/specs/language-toolchain-policy.md](../../specs/language-toolchain-policy.md)
  so the documented Rust workspace layout matches the added shared internal
  crate and any resulting verification-scope changes.

## Design Decisions

- Mirror the documented contract shape, not the current internal implementation
  shape of `ai-arena`.
- Prefer one shared transport/framing crate with separate `gamemaster` and
  `player` modules over copy-pasting ad hoc JSON handling into both runtime
  surfaces.
- Keep transport/framing DTOs separate from Reversi payload DTOs so game-owned
  state/action/result types continue to live under `games/reversi/`.
- Keep the crate boundary small enough that future upstream Rust SDK adoption,
  if it ever appears, can replace this layer without rewriting Reversi logic.

## Sub-tasks

- [x] Define the Rust protocol-crate responsibility in specs before adding
      transport code.
- [x] [parallel] Implement game-master-facing DTOs and NDJSON transport helpers.
- [x] [parallel] Implement AI-player-facing DTOs and NDJSON transport helpers.
- [x] [depends on: game-master DTOs, AI-player DTOs] Consolidate shared serde
      or framing helpers into the neutral `internal/aiarena-protocol` boundary.
- [x] [depends on: shared helpers] Register the shared crate in the root
      `Cargo.toml` workspace and wire the required path dependencies from
      `games/reversi/` and the Rust AI-player crate.
- [x] [depends on: shared helpers] Define how `games/reversi/` owns the
      Reversi payload DTOs consumed by both the game master and the AI-player.
- [x] [depends on: shared helpers] Add unit tests for request/response framing,
      metadata compatibility fields, and malformed-payload handling.
- [x] [depends on: tested crate surface] Document how later plans consume the
      shared transport crate plus the Reversi-owned payload DTOs.

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
- The resulting ownership split is explicit: generic transport in
  `internal/aiarena-protocol/`, Reversi payload types in `games/reversi/`, and
  runtime-specific usage in the game-master and AI-player crates.
- The root Cargo workspace and the toolchain-policy spec both reflect the added
  shared crate, so execution work does not have to infer hidden workspace
  wiring.

## Out of Scope

- Reversi rules implementation
- Tagged `arena-runner` e2e execution
- WASM AI search/evaluation logic
- Replay visualizer work
