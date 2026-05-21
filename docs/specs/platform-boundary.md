# Platform Boundary

## Purpose

`reversi-ai-arena` owns Reversi-specific implementation. `ai-arena` remains the
platform and public-contract repository.

## `ai-arena` Responsibilities

- public game-master protocol and compatibility contracts
- runtime, registry, replay export, and future spectator platform APIs
- generic runner behavior and WASM execution policy

## `reversi-ai-arena` Responsibilities

- Reversi rule implementation and game-master behavior
- Reversi AI players and reference bots
- Reversi replay visualizer and watcher-facing client code
- Reversi fixtures, deterministic artifacts, and public game-master examples
- Rust compatibility code that mirrors public `ai-arena` contracts when no
  official Rust package exists yet

## Boundary Rules

- Reversi code must consume public `ai-arena` contracts rather than reaching
  into platform-internal packages.
- The local Rust compatibility layer may mirror public transport DTOs and
  method names, but it must not import or re-create `ai-arena` internal package
  structure as a dependency boundary.
- Replay and future spectator flows must depend on exported runner artifacts or
  public spectator APIs, not on internal-only match state.
- Generic platform fixes discovered while building Reversi belong in
  `ai-arena`; this repository may only hold the game-specific integration and
  example side of that boundary.
- Knowledge or code ideas may be adapted from `reversi-adventure`, but this
  repository does not assume a shared-library boundary with it.
