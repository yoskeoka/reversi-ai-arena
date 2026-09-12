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

## Required Review Decision

Do not source-copy the Rust filesystem-oriented kifu helper into browser code. Before `/execute-task`, plan review must select one browser boundary
and record its rationale, ownership, and version compatibility in `docs/specs/visualizer-architecture.md`:

1. Normalize a versioned neutral DTO in TypeScript and verify Rust/browser parity through common fixtures.
2. Make a shared JSON fixture the canonical exchange, with independently implemented Rust and TypeScript parsers verified by golden parity.
3. Introduce a verified WASM bridge and verify bundle size, failure behavior, and version parity.

Until this selection is made, do not add React, a Reversi-specific backend endpoint, or a private artifact loader.

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

## Change Map

- `(MODIFY) docs/specs/visualizer-architecture.md` and `(MODIFY) docs/specs/artifact-kifu-export.md`:
  define public replay input, final-exported-snapshot consistency, replay format/version compatibility, the selected browser boundary, and error
  behavior. Keep private-artifact precedence restricted to the local helper.
- `(NEW) visualizer/src/replay/*`:
  implement the public-envelope decoder, format/version validator, immutable Reversi replay model, and turn-step/seek/playback reducer.
- `(NEW) visualizer/src/renderer/*` and `(NEW) visualizer/src/controls/*`:
  implement the Phaser board scene and playback controls. Provide current turn/player, score, result, and errors as text outside the canvas.
- `(MODIFY) visualizer/src/main.ts` and `(MODIFY) visualizer/src/style.css`:
  replace the scaffold with the viewer shell and local public-fixture load/error state.
- `(NEW) visualizer/src/replay/*.test.ts`, fixtures, and browser E2E:
  add placement/pass-bearing, terminal-failure, malformed, and version-mismatch fixtures, state goldens, and the browser replay path.
- `(DELETE)`: N/A. Do not relax the Rust kifu CLI/local audit input contract for the public viewer or introduce React, private artifact access,
  or a network bypass.

## Black-box Contract

- Given only A's terminal public replay payload and final exported snapshot, the viewer reconstructs a legal progression from the initial board to
  terminal board. Accepted placements and explicit accepted passes remain lossless; non-turn and non-accepted outcomes are never rendered as moves.
- The viewer supports previous/next turn, play/pause, seek, pass, score/current player, and terminal result. Malformed input, unsupported
  format/version, and failed/canceled terminal outcomes render an explainable error without guessing board state.
- The replay model is immutable: seek and playback never mutate prior model state. A final snapshot that conflicts with the transcript is rejected as
  a valid replay.
- Phaser is limited to board rendering. Controls, summary, and errors are semantic HTML text with keyboard-accessible controls. React, browser
  filesystem reads, private storage locators, and Reversi-specific platform APIs are out of scope.

## Implementation Order and Dependencies

1. Select the browser boundary in review and make the public envelope/fixture/version compatibility and failure behavior explicit in the specs.
2. Implement the selected decoder/normalizer and immutable replay reducer from placement/pass/failure fixtures.
3. Establish Phaser-independent state tests and fixture parity, then connect the board scene and controls to the reducer.
4. Implement fixture load, semantic summary, and error rendering in the shell; verify the complete terminal replay through browser E2E.
5. Consume A's public fixture/final exported snapshot and update version compatibility plus final-state consistency checks.

Steps 2 and 3's renderer/control work may proceed in parallel once the reducer contract is fixed. A's HTTP resource is not a dependency, but no
alternative envelope/version may be fixed ahead of A's versioned fixture. Network fetch and polling belong to the later platform-connection plan.

## Verification

- Verify selected-boundary Rust/TypeScript fixture parity, and WASM bundle compatibility when that boundary is selected, for every versioned fixture.
- Use replay-reducer goldens for accepted placement, explicit pass, seek/play/pause, final score/player/result, and input immutability. Add negative
  coverage for malformed data, unsupported version, early failed/canceled result, and final-snapshot mismatch.
- Run Phaser-independent unit tests, `npm run typecheck`, `npm run build`, and browser E2E from load through terminal replay.
- Assert a non-canvas text summary, keyboard control, and error announcement with browser/accessible-DOM tests.
- Inspect browser request logs to prove no request for `record.json`, `history.json`, private storage locators, or Reversi-specific backend endpoints.

## Follow-up

After A and this plan are implemented and merged, split ai-arena's
`docs/exec-plan/todo/0128-phase8-public-state-and-reversi-visualizer-platform-connection.md` into a detailed execution plan for the public
resource adapter, stale-response discard, and terminal polling stop.
