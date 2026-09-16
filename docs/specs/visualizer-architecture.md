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
  /api/v1-alpha/public/matches` with `game_id=reversi`,
  `game_version_major=1`, `page=1`, `limit=20`, `sort=completed_at`, and
  `sort_order=desc`. It adds `ruleset_version` only for a visitor-selected
  ruleset, then reads its selected match through
  `GET /api/v1-alpha/public/matches/{id}`, `/state`, and `/replay`. Every
  request omits credentials. API bases and match IDs are URL configuration,
  never credentials or private artifact locators.
- The server owns completed-record filtering and newest-first ordering for that
  Reversi major-version scope. The client validates returned scope and immutable
  public metadata but neither downloads a global list nor re-sorts it. The
  page-one response supplies `pagination` and scope-wide
  `available_ruleset_versions`; those metadata values, not page items or static
  configuration, define selector options. `selected_run_id` is informational
  metadata used to validate the selected public resources; it is never a
  viewer control or query parameter.
- A supported completed Reversi record has an immutable, calendar-valid
  `completed_at` timestamp in the platform's RFC 3339 UTC form and exactly two
  complete public `participants` entries. This form uses uppercase `T` and
  `Z`, and ordinary seconds from `00` through `59`; leap-second (`:60`)
  timestamps are unsupported because the public API's timestamp source does
  not generate them. The public spectator contract preserves submitted
  game-player order: entry zero is Black and entry one is White for standard
  Reversi. Each entry has a non-empty `display_name` and `ai_submission_id`.
  Detail and state resources must contain equal completion timestamps and
  participant field values in that sequence; JSON object-property order does
  not affect that comparison. Missing, malformed, or differently shaped
  metadata makes a record unsupported; a selected deep link reports that error
  rather than inventing player colours, names, or timestamps.
- The selector keeps the full ID as its value and deep-link parameter. A `match-` prefix
  followed by a canonical UUID displays as `match-` plus its first eight
  hexadecimal characters; other IDs remain complete. Revisions use the same
  UUID-only shortening rule. The replay summary identifies Black and White by
  colour, public display name, revision, and score.
- The base selector offers local, staging, and production public API profiles.
  A valid `api` query URL outside those profiles remains selected as a custom
  base rather than being replaced. A `match` query remains a deep-link default
  associated with its API base. An optional `ruleset` query selects and
  requests a single ruleset; omitting it requests every Reversi major-1
  ruleset. Changing either API base or ruleset clears `match` and reloads the
  first page. A deep-linked ruleset missing from response metadata remains
  visible as unavailable rather than being silently replaced. With neither
  API nor match selection the viewer renders its built-in fixture.
- Game-major selection is deliberately unsupported in this version.
- Terminal replay accepts `format: "reversi/replay"` and `version: "1"` only.
  Its Reversi-owned payload supplies `board_size`, `opening`, `ruleset`, and
  accepted placement/pass transcript entries. Its ruleset must exactly match
  the selected/list ruleset; its opening must be structurally valid for that
  ruleset. The final exported state is
  separately fetched from the public state resource and must agree with the
  replayed board, score, turn/player state, and terminal status.
- Unknown initial-position parameters, incompatible format/version, malformed
  transcript, incomplete/canceled terminal state, or disagreement with the
  final exported state is an explainable viewer error; the client never guesses
  an opening or falls back to `record.json`, `history.json`, or filesystem data.
- A state poller accepts only newer `(selected_run_id, state_version)` values,
  and stops when lifecycle is terminal or `retry_after_ms` is zero. Event
  streaming is a later phase.
- The selectable replay sequence contains the opening, every accepted turn,
  and the separately validated terminal state exactly once. Semantic `Initial
  state` and `Final state` controls select its first and last frame around the
  previous/play/next controls. At the terminal frame, the status says the
  match has ended and does not imply a next player.

## Hosting and Accessibility

The provider builds and hosts this static application independently from
`ai-arena`; the platform neither serves its assets nor executes it as a plugin.
Phaser renders the board only. Playback buttons, turn/score/result summaries,
and errors are semantic HTML so keyboard and assistive-technology users do not
need canvas access.
