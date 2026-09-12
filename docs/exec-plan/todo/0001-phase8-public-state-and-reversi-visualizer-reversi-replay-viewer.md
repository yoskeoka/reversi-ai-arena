# phase8-public-state-and-reversi-visualizer-reversi-replay-viewer
> **Execution**: Use `/execute-task` to implement this plan. After implementation is complete, use `/review-task` to prepare and create the PR.

Addresses: N/A

## Objective and Completion Boundary

Implement an artifact-first browser replay viewer in `reversi-ai-arena` that accepts only Phase 8 A's terminal public replay payload and final
exported snapshot. It replays board progression from the initial position through terminal state, including accepted placements, explicit passes,
score, current player, and terminal result. It has no dependency on private engine state, `record.json`, `history.json`, or browser filesystem access.

Completion is a versioned-public-fixture normalizer and immutable replay model, a Phaser board renderer with web-standard controls, and parser,
state, and browser end-to-end verification. Fetching an ai-arena public resource or polling a running match is excluded; those require the later
platform-connection plan after both repositories complete their first delivery.

## Selected Browser Boundary

The browser boundary is a versioned neutral DTO normalized in TypeScript. Reversi owns checked-in shared public JSON fixtures, generated only from
Phase 8 A's public envelope/final exported snapshot contract, and verifies Rust/browser parity against those fixtures. The Rust filesystem-oriented
kifu helper is neither source-copied nor invoked by browser code; verified WASM and a separate canonical fixture parser are rejected for this first
delivery because they add a toolchain/bundle or duplicate-parser boundary without helping the artifact-first viewer.

No React, Reversi-specific backend endpoint, private artifact loader, browser filesystem access, or alternative pre-A envelope/version is allowed.

## Existing References

- `docs/project-plan.md:15-31,103-119`: ai-arena owns public contracts; Reversi owns the visualizer and its game-specific fixtures. Phase 3 is
  artifact-driven replay; Phase 4 is separately gated on public spectator APIs.
- `docs/specs/platform-boundary.md:1-30`: Reversi consumes public platform contracts and must not depend on internal match state.
- `docs/specs/visualizer-architecture.md:5-32` and `visualizer/README.md:1-6`: Vite + TypeScript owns the shell, Phaser owns board rendering,
  and real-time support must use future public APIs.
- `docs/specs/artifact-kifu-export.md:20-80`: the local helper preserves accepted placement and explicit pass, but its `record.json` / `history.json`
  input precedence is not a public-viewer input contract.
- `visualizer/src/main.ts:1-20` and `visualizer/src/style.css`: the current surface is a scaffold without replay model, board renderer, or controls.
- `ai-arena/docs/exec-plan/todo/0126-phase8-public-state-and-reversi-visualizer.md`: Phase 8 A supplies the terminal public replay envelope,
  final exported snapshot, and versioned cross-repository public fixture.
- `docs/specs/reversi-game-master.md:90-144`: an immediate-loss turn is a valid completed terminal outcome with `current_player = null` and a
  surviving winner; it is not a canceled or malformed replay by itself.
- `visualizer/package.json:1-15` and `.github/workflows/visualizer-ci.yml:1-30`: the current Vite surface has no unit/browser test command or
  runner, and CI currently runs only typecheck/build.

## Change Map

- `(MODIFY) docs/specs/visualizer-architecture.md` and `(MODIFY) docs/specs/artifact-kifu-export.md`:
  define public replay input, final-exported-snapshot consistency, replay format/version compatibility, the selected TypeScript DTO/shared-fixture boundary, and error
  behavior. Keep private-artifact precedence restricted to the local helper.
- `(NEW) visualizer/src/replay/*`:
  implement the public-envelope decoder, format/version validator, immutable Reversi replay model, and turn-step/seek/playback reducer.
- `(NEW) visualizer/src/renderer/*` and `(NEW) visualizer/src/controls/*`:
  implement the Phaser board scene and playback controls. Provide current turn/player, score, result, and errors as text outside the canvas.
- `(MODIFY) visualizer/src/main.ts` and `(MODIFY) visualizer/src/style.css`:
  replace the scaffold with the viewer shell and local public-fixture load/error state.
- `(MODIFY) visualizer/package.json`、`visualizer/package-lock.json`、`.github/workflows/visualizer-ci.yml` と `(NEW) visualizer/vitest.config.ts`、
  `visualizer/playwright.config.ts`:
  add reproducible unit and browser E2E scripts, pinned test dependencies, browser installation/cache, and CI commands so state/unit, browser E2E,
  and accessible-DOM assertions are executable and required.
- `(NEW) visualizer/src/replay/*.test.ts`, `visualizer/e2e/*`, and Reversi-owned public fixtures:
  add placement/pass-bearing, terminal-failure, malformed, and version-mismatch fixtures, state goldens, and the browser replay path.
- `(DELETE)`: N/A. Do not relax the Rust kifu CLI/local audit input contract for the public viewer or introduce React, private artifact access,
  or a network bypass.

## Black-box Contract

- Given only A's terminal public replay payload and final exported snapshot, the viewer reconstructs a legal progression from the initial board to
  terminal board. Accepted placements and explicit accepted passes remain lossless; non-turn and non-accepted outcomes are never rendered as moves.
- The viewer supports previous/next turn, play/pause, seek, pass, score/current player, and terminal result. A completed immediate-loss terminal
  with its valid surviving winner is rendered as a terminal result even though its final action failed. Malformed input, unsupported format/version,
  canceled/incomplete terminal outcome, or transcript/final-snapshot inconsistency render an explainable error without guessing board state.
- The replay model is immutable: seek and playback never mutate prior model state. A final snapshot that conflicts with the transcript is rejected as
  a valid replay.
- Phaser is limited to board rendering. Controls, summary, and errors are semantic HTML text with keyboard-accessible controls. React, browser
  filesystem reads, private storage locators, and Reversi-specific platform APIs are out of scope.

## Implementation Order and Dependencies

1. Consume A's merged versioned public envelope/schema and final exported-snapshot fixture; record the exact compatible versions and the selected
   TypeScript DTO/shared-fixture boundary in the specs. Do not begin decoder work against a guessed envelope.
2. Implement the selected decoder/normalizer and immutable replay reducer from placement/pass/immediate-loss/failure fixtures.
3. Establish Phaser-independent state tests and fixture parity, then connect the board scene and controls to the reducer.
4. Implement fixture load, semantic summary, and error rendering in the shell; verify the complete terminal replay through browser E2E.
5. Wire the unit/browser test runners into package scripts and CI, then verify A fixture compatibility and final-state consistency on the required
   CI lane.

Steps 2 and 3's renderer/control work may proceed in parallel once A's fixture and the reducer contract are fixed. A's HTTP resource is not a
dependency. Network fetch and polling belong to the later platform-connection plan.

## Verification

- Verify the selected TypeScript DTO/Rust fixture parity for every versioned public fixture.
- Use replay-reducer goldens for accepted placement, explicit pass, valid immediate-loss result, seek/play/pause, final score/player/result, and
  input immutability. Add negative coverage for malformed data, unsupported version, canceled/incomplete result, and final-snapshot mismatch.
- Run the new Phaser-independent unit script, `npm run typecheck`, `npm run build`, and the new browser E2E script from load through terminal replay;
  CI must run all of them.
- Assert a non-canvas text summary, keyboard control, and error announcement with browser/accessible-DOM tests.
- Inspect browser request logs to prove no request for `record.json`, `history.json`, private storage locators, or Reversi-specific backend endpoints.

## Follow-up

After A and this plan are implemented and merged, split ai-arena's
`docs/exec-plan/todo/0128-phase8-public-state-and-reversi-visualizer-platform-connection.md` into a detailed execution plan for the public
resource adapter, stale-response discard, and terminal polling stop.
