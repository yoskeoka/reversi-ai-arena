# Visualizer Architecture

## Purpose

The initial visualizer is a replay-first browser client for public exported
match state. It is an optional program owned and hosted by the game provider,
not a program that `ai-arena` hosts or executes.

## Architecture

- Phaser owns the board-rendering layer.
- A lightweight browser shell owns playback controls, artifact loading, and
  layout outside the canvas.
- The shell should prefer web-standard APIs and minimal tooling rather than a
  heavy component framework.
- The Reversi provider builds and hosts the static application separately from
  `ai-arena`; its configured API base URL points only to documented public
  spectator resources.

## Packaging Rule

- `visualizer/` uses a lightweight Vite plus TypeScript shell.
- React and similar large UI frameworks are out of scope for the initial
  visualizer path.
- Phaser is introduced as a rendering dependency, not as the owner of
  application state outside the board view.

## Data Contract

- The client discovers replay candidates only through anonymous `GET
  /api/v1-alpha/public/matches`, then reads its selected match through
  `GET /api/v1-alpha/public/matches/{id}`, `/state`, and `/replay`. Every
  request omits credentials. API bases and match IDs are URL configuration,
  never credentials or private artifact locators.
- Discovery presents only completed records for the supported Reversi game,
  major version, and `standard` ruleset. `selected_run_id` is informational
  metadata used to validate the selected public resources; it is never a
  viewer control or query parameter.
- The base selector offers local, staging, and production public API profiles.
  A valid `api` query URL outside those profiles remains selected as a custom
  base rather than being replaced. A `match` query remains a deep-link default
  associated with its API base. With neither API nor match selection the
  viewer renders its built-in fixture.
- Ruleset and game-major selection are deliberately unsupported in this
  version. The viewer validates the replay payload as `ruleset: "standard"`;
  configurable rulesets remain deferred to `docs/issues/0004-visualizer-ruleset-selection.md`.
- Terminal replay accepts `format: "reversi/replay"` and `version: "1"` only.
  Its Reversi-owned payload supplies `board_size`, `opening`, `ruleset`, and
  accepted placement/pass transcript entries. The final exported state is
  separately fetched from the public state resource and must agree with the
  replayed board, score, turn/player state, and terminal status.
- Unknown initial-position parameters, incompatible format/version, malformed
  transcript, incomplete/canceled terminal state, or disagreement with the
  final exported state is an explainable viewer error; the client never guesses
  an opening or falls back to `record.json`, `history.json`, or filesystem data.
- A state poller accepts only newer `(selected_run_id, state_version)` values,
  and stops when lifecycle is terminal or `retry_after_ms` is zero. Event
  streaming is a later phase.

## Hosting and Accessibility

The provider builds and hosts this static application independently from
`ai-arena`; the platform neither serves its assets nor executes it as a plugin.
Phaser renders the board only. Playback buttons, turn/score/result summaries,
and errors are semantic HTML so keyboard and assistive-technology users do not
need canvas access.
