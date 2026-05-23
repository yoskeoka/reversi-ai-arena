# Tagged Runner Consumption

## Purpose

This specification fixes how `reversi-ai-arena` consumes the external
`ai-arena` runner host for Phase 1 end-to-end verification.

The goal is to keep the local and CI entrypoint identical:

1. install the tagged runner
2. build the Reversi-owned local artifacts
3. launch the match through the tagged runner
4. validate the produced artifacts

## Tagged Runner Version

- The pinned runner version for Phase 1 is
  `github.com/yoskeoka/ai-arena/cmd/arena-runner@v0.2.0`.
- Version changes must be explicit in this repository rather than inherited
  from a sibling workspace checkout.

## Game-Master Manifest Overlay

Phase 1 uses the runner's dev-only game-master manifest overlay:

- invocation uses `--game-master-manifest <path>`
- the manifest supplies:
  - `metadata.game_id`
  - `metadata.game_version`
  - `metadata.ruleset_version`
  - `runtime.kind = local-subprocess`
  - `runtime.command`
- if `runtime.command[0]` is a relative path, it resolves relative to the
  manifest file directory

Phase 1 uses this overlay because Reversi lives in an external consumer repo
rather than the runner's built-in registry.

## Scope Limits

- The manifest overlay is used for fresh runs only.
- Snapshot resume and history replay through the manifest overlay are out of
  scope for Phase 1.
- The manifest overlay is for the game master only. Player fixtures continue to
  use the normal AI sidecar or fallback entry-path rules.

## Fixture Player Contract

Phase 1 requires two deterministic fixture-player modes:

- a legal-move-first fixture that always chooses the first legal placement and
  emits `pass` only when `legal_actions` is empty
- a scripted fixture that replays a fixed move sequence and can also emit
  explicit `pass` when required by the game state

These fixtures exist to verify the game-master and runner integration. They are
not the main competitive AI path.

## Artifact Contract

A successful Phase 1 runner execution must produce the standard runner artifact
set under the chosen output directory:

- `record.json`
- `structured-log.ndjson`
- `snapshot.json`
- `exported-snapshot.json`
- `history.json`
- `result-summary.json`

At minimum, verification must assert that:

- the result summary status is `completed`
- the metadata tuple matches Reversi
- the exported snapshot is terminal
- the history includes explicit forced-pass turns when they occur

## Local And CI Alignment

- Local verification and CI must install the same tagged runner version.
- Local verification and CI must build the same Reversi-owned artifacts before
  launching the runner.
- Local verification and CI must use the same manifest-backed runner path
  rather than a repo-local special case.
