# Rust Protocol Compatibility

## Purpose

This specification defines the Rust compatibility layer that lets
`reversi-ai-arena` implement the public `ai-arena` game-master and AI-player
transport contracts without depending on `ai-arena` internal packages.

## Ownership Boundary

- `internal/aiarena-protocol/` owns game-agnostic Rust transport code.
- `games/reversi/` owns Reversi-specific payload DTOs.
- `players/rust-reference/` and future game-master binaries consume both the
  shared transport crate and the Reversi-owned payload DTOs.

The shared crate exists because `ai-arena` currently publishes a Go-side public
package for game masters, but no official Rust package yet.

## Source of Truth

- Method names, envelope rules, metadata fields, and failure classifications
  must mirror the public `ai-arena` documentation.
- Platform-internal Go packages are reference material only. Their package
  boundaries must not be recreated as Rust dependencies in this repository.
- Game-specific payload shapes belong in this repository because Reversi owns
  its visible state, legal-action hints, actions, public snapshots, and result
  summaries.

## Shared Crate Surface

`internal/aiarena-protocol/` must provide:

- JSON-RPC 2.0 request and response envelope types
- NDJSON encoder/decoder helpers for stdio transport
- envelope validation helpers for malformed JSON, invalid envelopes, and
  mismatched response IDs
- metadata compatibility helpers for `game_id`, `game_version`, and
  `ruleset_version`
- a `gamemaster` module for public game-master DTOs
- a `player` module for AI-player session DTOs and sidecar manifest DTOs

## Game-Master Module Contract

The `gamemaster` module mirrors the public `ai-arena` game-master contract for:

- method names:
  - `metadata`
  - `initialize_match`
  - `next_decision_step`
  - `normalize_action`
  - `apply_decision_results`
  - `current_snapshot`
  - `current_exported_snapshot`
  - `current_result`
  - `shutdown`
- shared metadata tuple:
  - `game_id`
  - `game_version`
  - `ruleset_version`
- player identity, decision-step, action-status, snapshot, and result DTOs
- JSON payload slots that can be specialized with Reversi-owned DTOs

The Rust layer may use generics or typed aliases so `games/reversi/` can plug
its DTOs into these game-agnostic transport envelopes.

## AI-Player Module Contract

The `player` module mirrors the public `ai-arena` AI-player runtime contract
for:

- request method names:
  - `init`
  - `turn`
  - `game_over`
- result payload shapes:
  - `{"ready": true}`
  - `{"action": ...}`
  - `{"ack": true}`
- sidecar manifest fields:
  - `ai_id`
  - `protocol.transport`
  - `protocol.game_id`
  - `protocol.game_version`
  - `protocol.ruleset_version`
  - `runtime.kind`
  - runtime-specific launch fields

`protocol.transport` is fixed to `stdio-jsonrpc-ndjson` for the current phase.

## Reversi DTO Ownership

`games/reversi/` must define the payload DTOs that both the future game master
and the Rust player consume:

- init-state payload
- visible-state payload
- legal-action-hint payload
- action payload
- public-state payload
- game-summary payload

Those DTOs must be used through shared transport types instead of duplicated
copies under `games/reversi/` and `players/`.

## Verification Contract

- Unit tests must cover JSON-RPC request and response framing.
- Unit tests must cover metadata compatibility rules, including
  `game_version` major matching.
- Unit tests must cover malformed-envelope or malformed-payload failures.
- The Rust reference-player crate must compile against the shared transport
  crate and Reversi DTOs even before a full WASM AI implementation lands.
