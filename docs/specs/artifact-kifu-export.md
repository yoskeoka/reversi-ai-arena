# Artifact Kifu Export

## Purpose

`reversi-ai-arena` provides a lightweight local helper that turns
`arena-runner` artifacts into a shareable Reversi move transcript.

This helper is Phase 2 follow-up review tooling. It is not the Phase 3 replay
visualizer, but its artifact parsing and transcript model must remain reusable
by the later replay path.

## Input Contract

The helper accepts exactly one directory argument.

Supported directory shapes:

- `<output-dir>/<match-id>`
- `<output-dir>` when it contains exactly one match subdirectory

The helper does not accept raw JSON files as the primary user-facing contract.

## Artifact Resolution

When the selected directory is resolved to a match artifact directory:

1. `record.json` is the source of truth when present
2. `history.json` is accepted only when `record.json` is absent

If the input points at an `output-dir` with multiple candidate match
subdirectories, the helper must fail and ask the user to pass
`<output-dir>/<match-id>` explicitly.

## Transcript Model

The helper must normalize artifacts into a lossless ordered transcript of
Reversi turns.

The lossless transcript:

- preserves accepted placement turns
- preserves explicit accepted `pass` turns
- ignores non-turn events such as `turn_requested`
- ignores non-accepted action outcomes such as timeout or protocol errors

Compact move text uses the same coordinate tokens as the repository's scripted
fixtures:

- placements use lowercase file plus rank such as `c4`
- explicit pass uses the token `pass`

## CLI Contract

The helper lives under `cmd/` as a small Rust CLI.

Default CLI behavior:

- reads one artifact directory argument
- prints the compact transcript to `stdout`
- omits `pass` tokens from the printed text
- exits non-zero on malformed artifacts or unresolved directories

Optional behavior:

- `--include-pass` prints the lossless transcript view and keeps `pass` tokens

## Reuse Boundary

The artifact parsing and transcript-building logic must live in a reusable
Reversi-owned Rust surface rather than inside the CLI entrypoint.

That reusable core must stay suitable for later:

- replay-model construction
- browser visualizer artifact loading
- transcript-oriented verification helpers

## Verification Expectations

Verification must cover:

- direct match-directory input
- `<output-dir>` input that resolves to one match subdirectory
- `record.json` precedence over `history.json`
- lossless preservation of explicit `pass`
- default compact output dropping `pass`
- `--include-pass` output keeping `pass`
- conversion of a real locally produced runner artifact directory
