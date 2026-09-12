# phase8-public-state-and-reversi-visualizer-reversi-replay-viewer
> **Execution**: Use `/execute-task` to implement this plan. After implementation is complete, use `/review-task` to prepare and create the PR.

Addresses: N/A

## Objective and Completion Boundary

Implement an artifact-first browser replay viewer in `reversi-ai-arena` that the Reversi provider can host separately from ai-arena. It accepts
only Phase 8 A's anonymous terminal public replay payload and final exported snapshot. It replays board progression from the initial position
through terminal state, including accepted placements, explicit passes, score, current player, and terminal result. It has no dependency on private
engine state, `record.json`, `history.json`, browser filesystem access, or an ai-arena-hosted visualizer runtime.

Completion is a versioned-public-fixture normalizer and immutable replay model, a Phaser board renderer with web-standard controls, an anonymous
public API loader / polling adapter, and parser, state, and browser end-to-end verification. The resulting Vite build is a deployable static
reference viewer owned and hosted by the Reversi provider; ai-arena does not receive, static-link, or execute its program.

## Fixed Browser Boundary

Select **a versioned neutral DTO in TypeScript plus shared public fixtures**. The browser owns a small TypeScript normalizer and Reversi owns
checked-in fixtures generated from A's public envelope, with Rust/browser parity verified by goldens. This is the smallest dependency for the
official Vite/Phaser reference viewer and keeps the viewer independently hostable by the game provider.

WASM is not selected for this delivery: ai-arena is not loading a game-provided visualizer as a plugin, so it needs no dynamic visualizer ABI.
The TypeScript decision does not constrain a future native or CLI Reversi viewer, nor the deferred platform-hosted visualizer issue. Do not
source-copy the Rust filesystem-oriented kifu helper into browser code. React, Reversi-specific backend endpoints, private artifact loaders,
browser filesystem access, and an alternative pre-A envelope/version remain excluded.

## Existing References

- `docs/project-plan.md:15-31,103-119`: ai-arena owns public contracts; Reversi owns the visualizer and its game-specific fixtures. Phase 3 is
  artifact-driven replay; Phase 4 is separately gated on public spectator APIs.
- `docs/specs/platform-boundary.md:1-30`: Reversi consumes public platform contracts and must not depend on internal match state.
- `docs/specs/visualizer-architecture.md:5-32` and `visualizer/README.md:1-6`: Vite + TypeScript owns the shell, Phaser owns board rendering,
  the Reversi provider owns hosting, and real-time support consumes public APIs.
- `docs/specs/artifact-kifu-export.md:20-80`: the local helper preserves accepted placement and explicit pass, but its `record.json` / `history.json`
  input precedence is not a public-viewer input contract.
- `visualizer/src/main.ts:1-20` and `visualizer/src/style.css`: the current surface is a scaffold without replay model, board renderer, or controls.
- [ai-arena PR #348](https://github.com/yoskeoka/ai-arena/pull/348) (its merge target is
  `docs/exec-plan/todo/0126-phase8-public-state-and-reversi-visualizer.md`): Phase 8 A supplies the terminal public replay envelope, final
  exported snapshot, and versioned cross-repository public fixture. This plan must not execute until the A implementation PR planned from that file
  has merged.
- `docs/specs/reversi-game-master.md:90-144`: an immediate-loss turn is a valid completed terminal outcome with `current_player = null` and a
  surviving winner; it is not a canceled or malformed replay by itself.
- `visualizer/package.json:1-15` and `.github/workflows/visualizer-ci.yml:1-30`: the current Vite surface has no unit/browser test command or
  runner, and CI currently runs only typecheck/build.

## Change Map

- `(MODIFY) docs/specs/visualizer-architecture.md` and `(MODIFY) docs/specs/artifact-kifu-export.md`:
  document consumer-side validation of A-owned public envelope/schema/version, final-exported-snapshot consistency, the selected boundary, and
  error behavior. Record that the static browser application is hosted by the Reversi provider and uses credential-free public API reads; it is
  not an ai-arena plugin. Only Reversi's opaque payload meaning belongs here; field-level public envelope definition remains in ai-arena A.
  Keep private-artifact precedence restricted to the local helper.
- `(NEW) visualizer/src/replay/*`:
  implement the public-envelope decoder, format/version validator, immutable Reversi replay model, and turn-step/seek/playback reducer.
- `(NEW) visualizer/src/api/*`:
  implement a configured public API base URL / `match_id` loader and polling adapter. It reads only A's documented anonymous `GET` resources,
  rejects stale `(selected_run_id, version)` responses, stops at terminal lifecycle, and gives a documented unavailable / network error without
  falling back to private artifacts or credentials.
- `(NEW) visualizer/src/renderer/*` and `(NEW) visualizer/src/controls/*`:
  implement the Phaser board scene and playback controls. Provide current turn/player, score, result, and errors as text outside the canvas.
- `(MODIFY) visualizer/src/main.ts` and `(MODIFY) visualizer/src/style.css`:
  replace the scaffold with the viewer shell, local public-fixture load/error state, and configured public API load/error state.
- `(MODIFY) visualizer/package.json`、`visualizer/package-lock.json`、`.github/workflows/visualizer-ci.yml` と `(NEW) visualizer/vitest.config.ts`、
  `visualizer/playwright.config.ts`:
  add reproducible unit and browser E2E scripts, pinned test dependencies, browser installation/cache, and CI commands so state/unit, browser E2E,
  and accessible-DOM assertions are executable and required.
- `(NEW) visualizer/src/replay/*.test.ts`, `visualizer/e2e/*`, and Reversi-owned public fixtures:
  add placement/pass-bearing, terminal-failure, malformed, and version-mismatch fixtures, state goldens, and the browser replay path.
- `(DELETE)`: N/A. Do not relax the Rust kifu CLI/local audit input contract for the public viewer or introduce React, private artifact access,
  or a network bypass.

## Black-box Contract

- Given only A's terminal public replay payload and final exported snapshot, the viewer validates the Reversi-owned opaque replay payload's
  `board_size`, `opening`, and `ruleset` initial-position parameters before reconstructing a legal progression from the specified initial board to
  terminal board. Unknown/incompatible initial-position parameters are rejected rather than silently using a four-disc standard opening. Accepted
  placements and explicit accepted passes remain lossless; non-turn and non-accepted outcomes are never rendered as moves.
- The viewer supports previous/next turn, play/pause, seek, pass, score/current player, and terminal result. A completed immediate-loss terminal
  with its valid surviving winner is rendered as a terminal result even though its final action failed. Malformed input, unsupported format/version,
  canceled/incomplete terminal outcome, or transcript/final-snapshot inconsistency render an explainable error without guessing board state.
- The replay model is immutable: seek and playback never mutate prior model state. A final snapshot that conflicts with the transcript is rejected as
  a valid replay.
- Phaser is limited to board rendering. Controls, summary, and errors are semantic HTML text with keyboard-accessible controls. React, browser
  filesystem reads, private storage locators, credentials, and Reversi-specific platform APIs are out of scope. The separately hosted web
  application reads A's anonymous public API with a configured base URL; it never assumes ai-arena serves its assets.

## Implementation Order and Dependencies

1. Consume A's merged versioned public envelope/schema and final exported-snapshot fixture. Record exact compatible versions, the TypeScript
   normalizer boundary, provider hosting boundary, and API base-URL configuration in the specs. Do not begin decoder or fetch work against a
   guessed envelope.
2. Implement the selected decoder/normalizer and immutable replay reducer from placement/pass/immediate-loss/failure fixtures.
3. Establish Phaser-independent state tests and fixture parity, then connect the board scene and controls to the reducer.
4. Implement fixture load plus anonymous public API load / polling, semantic summary, and error rendering in the shell; verify completed replay
   and running-to-terminal state through browser E2E.
5. Wire the unit/browser test runners into package scripts and CI, then verify A fixture compatibility, public request boundary, and final-state
   consistency on the required CI lane.

Steps 2 and 3's renderer/control work may proceed in parallel once A's fixture and the reducer contract are fixed. API adapter work begins after
A's anonymous HTTP contract is merged. Access protection is explicitly deferred to the later ai-arena `0128` parent plan.

## Verification

- Verify the selected boundary's compatibility contract for every versioned public fixture.
- Use replay-reducer goldens for accepted placement, explicit pass, valid immediate-loss result, seek/play/pause, final score/player/result, and
  input immutability. Add negative coverage for malformed data, unsupported version, canceled/incomplete result, and final-snapshot mismatch.
- Run the new Phaser-independent unit script, `npm run typecheck`, `npm run build`, and the new browser E2E script from load through terminal replay;
  CI must run all of them.
- Assert a non-canvas text summary, keyboard control, and error announcement with browser/accessible-DOM tests.
- Verify a separately served static build loads the configured anonymous public API base URL, drops stale response, stops polling after terminal,
  and renders documented unavailable / network errors without credentials. Inspect browser request logs to prove no request for `record.json`,
  `history.json`, private storage locators, cookies/tokens, or Reversi-specific backend endpoints.

## Follow-up

After A and this plan are implemented and merged, create the detailed C execution plan from the intentional parent that will be added by
[ai-arena PR #348](https://github.com/yoskeoka/ai-arena/pull/348) at
`docs/exec-plan/todo/0128-phase8-public-state-and-reversi-visualizer-platform-connection.md`. It will cover protected access for external
visualizer and non-browser consumers; it must not replace the anonymous first delivery without a reviewed migration contract.
