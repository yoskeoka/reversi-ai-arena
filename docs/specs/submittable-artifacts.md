# Submittable Reversi Artifacts

## Purpose

This specification defines the immutable release artifacts that this repository
hands to `ai-arena`. The platform owns the `arena-bundle/v1` schema and
validator and bundle runner; Reversi pins and consumes commit
`bd2de02252e0e5925aa19402c1ee569588d05a10` of `yoskeoka/ai-arena`.

## Release Assets

A public release uses the source-controlled Reversi game version `X.Y.Z` and
the exact Git tag `vX.Y.Z`. It publishes exactly these files:

- `reversi-game-vX.Y.Z.arena.zip`
- `reversi-rust-reference-ai-vX.Y.Z.arena.zip`
- `SHA256SUMS`

Each ZIP has exactly two root entries, in this order: `manifest.json` and its
declared WASM module. Packaging fixes ZIP timestamps and ordering, so identical
source and release version produce identical bytes. `SHA256SUMS` records the
SHA-256 of those exact ZIP bytes.

## Manifests

The game manifest declares `schema_version` `arena-bundle/v1`, `artifact_kind`
`game`, `game_id` `reversi`, and the source-controlled `game_version` `X.Y.Z`,
plus one `standard` ruleset with `player_count` 2 and
`max_active_bots_per_owner` 3. Its runtime is `wasm-wasi` and declares
`reversi-gamemaster.wasm`.

The AI manifest declares the same schema, game identity, and `standard`
ruleset; it declares `artifact_kind` `ai`, `ai_id` `rust-reference`, and the
`wasm-wasi` module `rust-reference-ai.wasm`. The technical AI identity is not
the user-visible bot name. The game-master runtime metadata reports that same
source-controlled version.

No manifest extensions are permitted. In particular, manifests do not include
hashes, decision-mode fields, game-master protocol versions, or zero-valued
resource limits. A tagged release must reject a malformed tag or a tag whose
version differs from either generated manifest before GitHub Release creation.
Local `dev` artifacts remain available for verification but are not admissible
tagged releases.

Each admitted release version is immutable. A source change that changes the
admitted artifact selects and commits its next semantic version before
publication. Same-major versions remain compatible with existing same-major
bots, but are distinct immutable game releases. The historical `v0.1.1`
assets remain immutable and cannot be retroactively changed or used to
duplicate the `1.0.0` game release.

## Validation and Handoff

`make verify-release-artifacts` validates both ZIPs with the pinned upstream
validator and compares its `sha256:<hex>` result with `SHA256SUMS`. It then
starts a standard two-player match using the game ZIP as `--game-master-bundle`
and the AI ZIP twice as `--player-bundle`; the resulting standard artifact
summary and exported snapshot must both report `completed`. The GitHub Release
uploads these same bytes without rewriting their manifests. Staging submits
those release assets without repacking them.

The accepted `v1.1.0` release has game and AI manifests with
`game_version: 1.1.0`, game-master runtime metadata with `game_version:
1.1.0`, and matching downloaded ZIP checksums recorded in `SHA256SUMS`.
