# Submittable Reversi Artifacts

## Purpose

This specification defines the immutable release artifacts that this repository
hands to `ai-arena`. The platform owns the `arena-bundle/v1` schema and
validator and bundle runner. Reversi pins ai-arena v0.3.0 at
`212839f537274d8874a2e29afc2bb292cd842dd0`.

## Release Assets

An operator-built release uses the source-controlled Reversi game version
`X.Y.Z` and the exact Git tag `vX.Y.Z`. It produces exactly these files:

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
plus `standard`, `standard-xot-large`, and `standard-xot-small`, each with
`player_count` 2 and `max_active_bots_per_owner` 3. Its runtime is
`wasm-wasi` and declares `reversi-gamemaster.wasm`.

The AI manifest declares the same schema, game identity, and three rulesets.
It sets `artifact_kind` to `ai`, `ai_id` to `rust-reference`, and the
`wasm-wasi` module to `rust-reference-ai.wasm`. The technical AI identity is not
the user-visible bot name. The game-master runtime metadata reports that same
source-controlled version.

No manifest extensions are permitted. In particular, manifests do not include
hashes, decision-mode fields, game-master protocol versions, or zero-valued
resource limits. A tagged build rejects a malformed tag and any version that
differs from either generated manifest.
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
starts a two-player match for each ruleset using the game ZIP as
`--game-master-bundle` and the AI ZIP twice as `--player-bundle`. The artifact
summary and exported snapshot both report `completed`.

The operator verification registers one immutable game digest for all three
rulesets. The platform selects the registered ruleset at WASI startup. It
runs the matches concurrently and persists each public replay. The exact
game version maps to one game digest.

Version `v1.2.0` uses the operator preparation procedure in the
[XOT reference](../references/xot.md). Preparation validates the official input
bytes, row counts, unique coordinate sequences, and legality in the Reversi
engine before compiling both lists into the WASM artifact. Missing or modified
input fails the build. Matches require no download. While the source pages
provide no explicit redistribution terms, tagged CI verifies the bundles and
operators prepare the bundle from source for their own deployment.
