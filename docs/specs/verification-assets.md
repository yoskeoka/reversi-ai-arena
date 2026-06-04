# Verification Assets

## Purpose

This specification records the deterministic assets and ownership boundaries
used to prove that the Reversi game-master path and the Rust WASM AI-player
path complete through the tagged runner.

## Phase 1 Scripted Fixture Suite

Phase 1 owns four canonical scripted Reversi lines under
`testdata/reversi/scripted-games/`:

- `end with 1 empty cell(forced-pass for both)`
  - `f5f6e6f4e3c5c4d6b5d3c3e2f2c2d2b3b4f3c1e1g3g4h4h5c6h3g5f1c7a4a5h6d7g6a3e7f8d1f7b1g2b2h2h1g1b8c8e8d8g8a1a2h7h8g7b7b6a6a7passpass`
- `fastest black win`
  - `f5d6c5f4e7f6g5e6e3passpass`
- `short white win`
  - `f5f6e6d6e7f7d7f4c5c7c6b6passpass`
- `multiple passes in the middle and ends with some empty cells`
  - `d3c5f6f5e6e3d6f7b6e7f3c6d7c8g5f4g7g6e8c7d8h8b5f2h5h4f1g4h7h6b8g3g2h3passh2passc4b3g8passf8passa8passb4c3h1a3g1e2e1passb7passc2d1a7d2a6passc1passa4a5a2passpass`

This suite is the canonical scripted-fixture acceptance set for proving:

- deterministic completion through the tagged runner
- explicit forced-pass turn requests and preserved pass history
- terminal completion by consecutive passes even when empty cells remain
- short black-win and white-win completions in the same scripted-fixture lane

## Asset Ownership

- `games/reversi/` owns core rules, validation, snapshot logic, and game-master
  state transitions
- `cmd/` owns the runnable game-master entrypoint
- `players/` owns local fixture-player executables and the Rust WASM player
  artifact source
- `e2e/` owns helper code that prepares manifests, binaries, and runner
  invocations
- `testdata/` owns deterministic scripted lines and any expected replay-safe
  artifacts checked into the repository

## Build And Regeneration Policy

- Local fixture binaries may be rebuilt during test setup.
- The Rust WASM player fixture must be built once per verification scope and
  then reused through cached artifact paths for the remaining tests in that
  scope.
- Cached WASM artifacts may live under a repository-local temporary directory
  or under `target/`-adjacent build output, but the helper that owns them must
  hide those details from the individual tests.
- Temporary manifests that point at built local binaries may be generated during
  test setup and do not need to be committed.
- Temporary player sidecars that point at cached `.wasm` modules may be
  generated during test setup and do not need to be committed.
- Checked-in fixed move lines under `testdata/` are canonical and may change
  only when the intended game contract changes.
- Checked-in kifu-export fixtures under `testdata/reversi/artifacts/` may store
  compact runner-artifact slices that prove transcript extraction behavior.

## Minimum Verification Set

- unit tests for legal-move generation, flipping, pass detection, and terminal
  scoring
- unit tests for the Rust WASM player decision logic
- at least one tagged-runner match completed by the legal-move-first fixture
- at least one tagged-runner match completed by the Rust WASM player fixture
- tagged-runner deterministic replay of the four canonical scripted lines
- tagged-runner coverage for immediate loss on invalid or unusable action
- unit coverage for artifact-to-transcript extraction from checked-in
  `record.json` and `history.json` fixtures

## Review Expectations

When these assets change, review should confirm:

- the scripted lines remain intentional
- pass-bearing scripted lines still encode explicit `pass` turns correctly
- local and CI use the same runner version
- no fixture asset quietly becomes the mainline competitive player surface
- the WASM fixture path is cache-backed rather than rebuilt independently by
  each runner test
