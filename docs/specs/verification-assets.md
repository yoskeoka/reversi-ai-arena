# Verification Assets

## Purpose

This specification records the deterministic assets and ownership boundaries
used to prove that the Reversi game-master path completes through the tagged
runner.

## Phase 1 Fixed Completion Lines

Phase 1 owns two fixed scripted completion lines:

- `c4e3f6e6f5c5f4g6f7g5d6d3f3c3h4h5g4h3e2f2d2d1g3e1b6c2b4e8f8g8b3h2f1g1b1a3h7a5a4b5a6a7c6e7d7d8g7a2c8b8c7c1h1h8h6g2b2b7a8a1`
- `c4e3f4c5d6f3e6c3d3e2b6f5b4f6c2e7d2c7f1c6f2a6d7c8f8d8g5g6e8h4b8f7g8b5g3g4h3a3h5b3b7h6h7e1d1b1c1g1g2a8a7h8g7h2a4h1a5b2a2a1`

These lines are the canonical scripted fixtures for proving:

- deterministic completion
- end-of-game score stability
- no silent forced-pass skips inside the game-master path

## Asset Ownership

- `games/reversi/` owns core rules, validation, snapshot logic, and game-master
  state transitions
- `cmd/` owns the runnable game-master entrypoint
- `players/` owns local fixture-player executables
- `e2e/` owns helper code that prepares manifests, binaries, and runner
  invocations
- `testdata/` owns deterministic scripted lines and any expected replay-safe
  artifacts checked into the repository

## Build And Regeneration Policy

- Local fixture binaries may be rebuilt during test setup.
- Temporary manifests that point at built local binaries may be generated during
  test setup and do not need to be committed.
- Checked-in fixed move lines under `testdata/` are canonical and may change
  only when the intended game contract changes.

## Minimum Verification Set

- unit tests for legal-move generation, flipping, pass detection, and terminal
  scoring
- at least one tagged-runner match completed by the legal-move-first fixture
- tagged-runner deterministic replay of both fixed scripted completion lines
- tagged-runner coverage for immediate loss on invalid or unusable action

## Review Expectations

When these assets change, review should confirm:

- the scripted lines remain intentional
- local and CI use the same runner version
- no fixture asset quietly becomes the mainline competitive player surface
