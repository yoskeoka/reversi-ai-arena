# Tagged Runner Consumption

## Purpose

This specification fixes how `reversi-ai-arena` consumes the external
`ai-arena` runner host for Phase 1 and Phase 2 end-to-end verification.

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

The scripted-fixture lane must cover the repository's canonical four-case suite:

- `end with 1 empty cell(forced-pass for both)` proves terminal double-pass
  completion with one empty square still on the board
- `fastest black win` proves a short deterministic black-win completion
- `short white win` proves a short deterministic white-win completion
- `multiple passes in the middle and ends with some empty cells` proves
  required-pass turns before the final terminal double-pass

Checked-in scripted lines under `testdata/reversi/scripted-games/` must encode
forced-pass turns as literal `pass` tokens at the exact turn where the player
response is required.

## Phase 2 WASM Player Contract

Phase 2 extends the same runner path with a cached WASM AI fixture:

- the player sidecar must declare `runtime.kind = wasm-wasi`
- the sidecar must point at a cached `.wasm` module built from
  `players/rust-reference/`
- runner-based tests may reuse the same cached `.wasm` module and generated
  sidecar across multiple test cases in the same verification scope
- scripted or legal-move-first subprocess bots remain valid opponents and
  failure fixtures for the WASM lane

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
- the terminal double-pass fixture leaves one empty square after normal
  completion rather than converting the ending into an immediate loss
- the mid-game forced-pass fixture preserves one or more accepted `pass` turns
  before the final terminal state
- the WASM player path resolves through a sidecar manifest rather than a native
  player binary path

## Local And CI Alignment

- Local verification and CI must install the same tagged runner version.
- Local verification and CI must build the same Reversi-owned artifacts before
  launching the runner.
- Local verification and CI must use the same manifest-backed runner path
  rather than a repo-local special case.
- Local verification and CI must build the Rust WASM fixture once per
  verification scope and then reuse that artifact across runner tests.
