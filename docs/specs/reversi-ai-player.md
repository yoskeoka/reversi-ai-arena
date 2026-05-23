# Reversi AI Player

## Purpose

This specification defines the Phase 2 mainline AI-player surface for
`reversi-ai-arena`.

The owner of this surface is `players/rust-reference/`. It must remain a local
consumer of the Reversi game and protocol crates while shipping through the
`wasm-wasi` runtime path used by `ai-arena`.

## Runtime Contract

- The mainline Reversi AI-player is a Rust program compiled for
  `wasm32-wasip1`.
- The sidecar manifest must declare:
  - `runtime.kind = wasm-wasi`
  - `runtime.module = ./reversi-rust-reference-player.wasm`
  - `runtime.args = ["./reversi-rust-reference-player.wasm"]`
  - `runtime.memory_limit_pages = 64` unless a future ruleset changes the
    budget explicitly
- The player must use the `stdio-jsonrpc-ndjson` transport contract and emit
  only JSON-RPC responses on `stdout`.

## Turn Contract

- `legal_action_hint.legal_actions` is the authoritative player-side move set.
- When `legal_actions` is empty, the player must respond with `pass`.
- When `legal_actions` is non-empty, the player must not respond with `pass`.
- The player must return one explicit action for every `turn` request. Forced
  pass turns are not silent skips.
- If the player has one or more legal moves and returns an illegal action or
  times out, the game-master path is expected to treat that result as an
  immediate loss for the player.

## Decision Quality Bar

Phase 2 is not satisfied by a first-legal or scripted completion bot.

The Rust mainline player must exceed the Phase 1 fixture bots by applying
non-trivial move selection logic that combines:

- legal-move simulation on the current visible board
- corner preference
- corner-adjacent risk penalties when the corner is still empty
- mobility pressure on the opponent
- edge and disc-balance heuristics as tie-break support

A shallow search is acceptable for Phase 2 as long as it is deterministic and
clearly more selective than the fixture bots.

## Fixture Ownership

- `players/rust-reference/` owns the WASM module and its sidecar manifest shape.
- `e2e/reversi-runner/` owns helper code that builds, caches, and exposes the
  WASM fixture paths consumed by runner-based tests.
- `players/fixture-bots/` remains the local subprocess verification surface for
  deterministic scripted opponents and invalid-action fixtures.
- `players/go-reference/` remains optional and subordinate. It is not required
  for Phase 2 completion if the Rust WASM lane already covers the intended
  runner path.

## Verification Expectations

- Rust unit tests must cover deterministic move selection behavior.
- Player tests must prove:
  - pass is emitted only when `legal_actions` is empty
  - a legal placement is chosen when one or more legal moves exist
  - the heuristic path prefers a clearly stronger move in at least one
    representative board state
- Runner-based verification must execute the player through its cached
  `wasm-wasi` artifact rather than through a native subprocess fallback.
