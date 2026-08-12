# Submittable Reversi Artifacts

## Purpose

This specification defines the immutable release artifacts that this repository
hands to `ai-arena`. The platform owns the `arena-bundle/v1` schema and
validator and bundle runner; Reversi pins and consumes commit
`bd2de02252e0e5925aa19402c1ee569588d05a10` of `yoskeoka/ai-arena`.

## Release Assets

A release version `R` publishes exactly these files:

- `reversi-game-R.arena.zip`
- `reversi-rust-reference-ai-R.arena.zip`
- `SHA256SUMS`

Each ZIP has exactly two root entries, in this order: `manifest.json` and its
declared WASM module. Packaging fixes ZIP timestamps and ordering, so identical
source and release version produce identical bytes. `SHA256SUMS` records the
SHA-256 of those exact ZIP bytes.

## Manifests

The game manifest declares `schema_version` `arena-bundle/v1`, `artifact_kind`
`game`, `game_id` `reversi`, `game_version` `1.0.0`, and one `standard`
ruleset with `player_count` 2 and `max_active_bots_per_owner` 3. Its runtime is
`wasm-wasi` and declares `reversi-gamemaster.wasm`.

The AI manifest declares the same schema, game identity, and `standard`
ruleset; it declares `artifact_kind` `ai`, `ai_id` `rust-reference`, and the
`wasm-wasi` module `rust-reference-ai.wasm`. The technical AI identity is not
the user-visible bot name.

No manifest extensions are permitted. In particular, manifests do not include
hashes, decision-mode fields, game-master protocol versions, or zero-valued
resource limits. Release filename version identifies the build; the game
compatibility version remains `1.0.0`.

## Validation and Handoff

`make verify-release-artifacts` validates both ZIPs with the pinned upstream
validator and compares its `sha256:<hex>` result with `SHA256SUMS`. It then
starts a standard two-player match using the game ZIP as `--game-master-bundle`
and the AI ZIP twice as `--player-bundle`; the resulting standard artifact
summary and exported snapshot must both report `completed`. The GitHub Release
uploads these same bytes. Staging submits those release assets without
repacking them.
