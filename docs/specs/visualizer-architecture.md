# Visualizer Architecture

## Purpose

The initial visualizer is a replay-first browser client for runner-exported
match artifacts.

## Architecture

- Phaser owns the board-rendering layer.
- A lightweight browser shell owns playback controls, artifact loading, and
  layout outside the canvas.
- The shell should prefer web-standard APIs and minimal tooling rather than a
  heavy component framework.

## Packaging Rule

- `visualizer/` uses a lightweight Vite plus TypeScript shell.
- React and similar large UI frameworks are out of scope for the initial
  visualizer path.
- Phaser is introduced as a rendering dependency, not as the owner of
  application state outside the board view.

## Data Contract

- The initial client reads exported JSON artifacts produced by the runner path.
- The client may reuse the Reversi-owned artifact parsing and transcript core
  introduced for kifu export rather than reimplementing runner-artifact
  decoding from scratch.
- Replay input must be reconstructible without private engine state.
- Real-time watch support is a later phase that must consume future
  spectator-facing public APIs rather than a Reversi-specific bypass.
