# Phase 2 Post-Review Artifact Kifu Export
**Execution**: Use `/execute-task` to implement this plan.

## Objective

Add a lightweight Reversi-owned helper that turns `arena-runner` output
artifacts into a human-shareable move transcript so Phase 2 outcomes can be
inspected without manually reading raw JSON.

This is a post-Phase-2 follow-up, not the start of the Phase 3 browser
visualizer. Its acceptance bar is a compact inspection/export path that is
useful immediately and does not become throwaway work when the replay
visualizer is implemented later.

## Confirmed Direction

- The primary user-facing input surface is an `arena-runner` artifact
  directory:
  - `<output-dir>`
  - or `<output-dir>/<match-id>`
- Source-of-truth resolution still prefers `record.json` when it exists.
- `history.json` may be accepted as a fallback only when `record.json` is not
  available in the selected directory.
- The helper internally normalizes artifacts into a lossless move transcript
  that preserves explicit `pass` turns.
- The default CLI output is the compact shared notation that omits `pass`,
  because that is the common human copy/paste form.
- `--include-pass` switches the CLI output to the lossless transcript view.
- The helper should be implemented as a small Rust CLI under `cmd/`, while the
  artifact parsing and transcript-building logic lives in a reusable
  repository-owned Rust surface rather than being trapped inside the CLI
  entrypoint.
- The reusable parsing/transcript core must remain suitable for later Phase 3
  replay-model construction.

## Code Changes

- Add a lightweight Rust helper CLI under `cmd/` for exporting Reversi move
  transcripts from runner artifacts.
- Add a shared Rust module or crate in the Reversi-owned surface that:
  - resolves directory input to the correct artifact files
  - loads `record.json` or fallback `history.json`
  - extracts a lossless ordered move transcript
  - renders compact or pass-inclusive text output
- Add tests and deterministic fixture inputs for:
  - directory resolution
  - `record.json` precedence over `history.json`
  - explicit `pass` preservation in the internal transcript
  - compact output dropping `pass`
  - `--include-pass` output retaining `pass`

## Spec Changes

- Add a new spec for the artifact-kifu export helper covering:
  - intended role as a Phase 2 post-review inspection tool
  - accepted input directory shapes
  - artifact resolution order
  - the distinction between internal lossless transcript and default compact
    output
  - the `--include-pass` behavior
- Update `docs/specs/verification-assets.md` to record any checked-in
  transcript fixtures or transcript expectations introduced for verification.
- Update `docs/specs/visualizer-architecture.md` to state that the future
  visualizer may reuse the same artifact parsing/transcript core rather than
  re-implementing independent runner-artifact decoding.

## Design Decisions

- Keep this work positioned as Phase 2 follow-up review tooling rather than
  prematurely relabeling it as Phase 3 delivery.
- Match the human-facing CLI default to the common compact notation even though
  the repository's correctness model keeps explicit `pass` internally.
- Use directory input as the public CLI contract so the helper matches the
  runner artifact layout that humans already have on disk.
- Keep `record.json` as the source-of-truth artifact in line with the platform
  contract, while allowing `history.json`-only fallback for narrower local use.
- Avoid a throwaway script path: parsing and transcript construction should be
  reusable by the later visualizer implementation.

## Sub-tasks

- [ ] Define the helper contract in specs before implementing code.
- [ ] [parallel] Design the reusable artifact parsing and transcript-building
      core.
- [ ] [parallel] Define the CLI surface, including default compact output and
      `--include-pass`.
- [ ] [depends on: reusable parsing core, CLI surface] Implement the Rust
      helper entrypoint under `cmd/`.
- [ ] [depends on: reusable parsing core] Add deterministic fixtures and tests
      for artifact resolution and transcript extraction.
- [ ] [depends on: helper implementation, test fixtures] Verify that a real
      `arena-runner` output directory can be converted into both compact and
      pass-inclusive outputs.

## Parallelism

- Spec writing can define the helper contract before implementation begins.
- Parsing/transcript core design and CLI-surface design can proceed in parallel
  once the scope is fixed.
- Fixture definition can start alongside implementation as long as it stays
  aligned with the agreed artifact resolution order.

## Verification

- Rust unit tests cover transcript extraction from deterministic Reversi
  artifacts.
- Tests prove `record.json` is preferred when both `record.json` and
  `history.json` are present.
- Tests prove explicit `pass` turns survive the lossless internal transcript.
- Tests prove default CLI output omits `pass`.
- Tests prove `--include-pass` emits the pass-inclusive transcript.
- A real runner output directory from this repository can be converted by the
  helper without manual JSON editing.

## Out of Scope

- Phaser board rendering
- Browser replay controls or artifact upload UI
- Real-time watcher integration
- Replacing `record.json` as the platform source of truth
- General-purpose notation import back into the game master or runner
