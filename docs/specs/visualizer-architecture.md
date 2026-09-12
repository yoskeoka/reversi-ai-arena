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

- The initial client reads versioned exported-state and public replay JSON from
  `ai-arena`'s anonymous public API; checked-in equivalent fixtures support
  offline verification.
- The client may reuse the Reversi-owned artifact parsing and transcript core
  introduced for kifu export rather than reimplementing runner-artifact
  decoding from scratch.
- Replay input must be reconstructible without private engine state.
- Snapshot polling for an in-progress match consumes the same public API and
  discards stale versions; it stops at terminal lifecycle. Event streaming is
  a later phase, not a Reversi-specific bypass.
