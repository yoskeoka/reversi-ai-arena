# Reversi game-version release skill

> **Execution**: Use `/execute-task` to implement this plan. After implementation is complete, use `/review-task` to prepare and create the PR.

## Objective and completion boundary

Make every admitted Reversi game release use one source-controlled semantic
version across its Git tag, release asset filenames, game and AI bundle
manifests, and runtime metadata. Add a repository-local skill that performs
the post-merge release operation safely. Validate that skill by releasing
`v1.1.0` from the merged implementation commit and confirming the published
artifacts.

This plan ends after the GitHub `v1.1.0` release has passed its artifact
workflow and the skill has verified its assets and manifests. Registering the
new game artifact in ai-arena, activating a staging scope, and creating a
staging match are out of scope.

The already-published `v0.1.1` assets remain immutable historical assets. Do
not overwrite their tag, release, manifest, or ai-arena registration.

Addresses: N/A - no local issue exists.

## Chosen release identity

Use source-controlled `reversi_game::GAME_VERSION` as the authoritative game
release version. A registered release's Git tag must be exactly
`v${GAME_VERSION}`; its game and AI bundle manifests and game-master runtime
metadata must report `${GAME_VERSION}`. A source change that changes an
admitted game artifact must select and commit the next appropriate semantic
version before release. The `current_public_replay` capability is a
backwards-compatible addition, so this execution uses `1.1.0` and tag
`v1.1.0`.

Do not derive runtime metadata by rewriting a packaged manifest from the tag:
that would let a manifest and the compiled game-master metadata diverge. The
release workflow validates the tag-to-source relationship and only packages
the already-versioned source.

## Current evidence

- `games/reversi/src/lib.rs:10-12` owns the single `GAME_VERSION`, currently
  `1.0.0`; game metadata reads it at `126-132`.
- `games/reversi/src/gamemaster.rs:272-297` emits that constant in snapshot
  and exported-snapshot metadata. `players/rust-reference/src/lib.rs:44` also
  uses it for the reference AI protocol.
- `tools/release-packager/src/main.rs:1-48` builds both manifests from those
  Rust identities, while `tools/release-packager/package.sh:5-51` uses a
  separate caller-provided version only for filenames and checksums.
- `.github/workflows/release.yml:3-37` currently accepts any `v*` tag or
  manual input and forwards that separate value to packaging without checking
  the compiled metadata.
- `docs/specs/submittable-artifacts.md:12-38` currently states that the asset
  filename version is separate from the fixed `1.0.0` compatibility version.
  That conflicts with immutable ai-arena registration identity.
- ai-arena uses exact `(game_id, game_version)` uniqueness for immutable
  releases while resolving compatibility by major; therefore new Reversi
  bytes cannot be admitted as `reversi@1.0.0` but a `1.1.0` game remains
  compatible with already registered `1.0.0` bots.

## Change map

- `(NEW) .claude/skills/release-reversi-game/SKILL.md` — repository-local,
  discoverable operational skill for version selection, merged-release
  preflight, tag publication, workflow polling, and released-artifact
  verification. It must stop for user direction when the semantic bump or
  external release target is not explicit.
- `(MODIFY) docs/specs/submittable-artifacts.md` — make the source game
  version and `v`-prefixed Git/release tag the same release identity; define
  immutable-version and legacy-`v0.1.1` handling; state the tag/manifest
  validation and the exact `v1.1.0` acceptance evidence.
- `(MODIFY) docs/specs/reversi-game-master.md` — advance the declared
  `game_version` to `1.1.0` and document that its value is the
  source-controlled release identity shared by runtime metadata and bundles.
- `(MODIFY) README.md` — replace stale `1.0.0` local manifest examples with
  `1.1.0` and link the canonical release-version policy rather than duplicate
  the operational procedure.
- `(MODIFY) games/reversi/src/lib.rs` — set `GAME_VERSION` to `1.1.0`; its
  existing consumers propagate the value to game-master and reference-player
  runtime protocol metadata.
- `(MODIFY) e2e/reversi-runner/src/lib.rs` — update hard-coded Reversi
  metadata expectations and fixture JSON to the source identity, preferably
  by consuming the shared constant where dependency boundaries permit.
- `(MODIFY) tools/release-packager/package.sh` and
  `tools/release-packager/verify.sh` — make a release-named package reject a
  mismatch between `v<semver>` asset version and the generated game/AI manifest
  versions, while retaining explicit `dev` artifacts for local verification.
- `(MODIFY) .github/workflows/release.yml` — make a release originate only
  from the exact `v${GAME_VERSION}` tag, run the version check before
  publication, and use the tag unchanged for asset filenames and GitHub
  release creation. Do not permit manual-input version overrides that produce
  a different release identity.

## Black-box contract changes

- A public Reversi release tag `vX.Y.Z` publishes game and AI bundles whose
  `manifest.json` files both report `game_version: X.Y.Z`; the game-master
  runtime reports the same value.
- A tag whose value does not match the checked-out source `GAME_VERSION`
  fails before GitHub Release creation and does not publish assets.
- A source version bump in the same major produces a separately admitted game
  release without requiring existing same-major bots to be re-registered.
- Local `dev` artifact verification remains available but cannot be mistaken
  for an admissible tagged release.

## Implementation tasks

1. Update the black-box specs first, including the semantic-version selection
   policy, tag alignment, immutability, and the fact that `v0.1.1` cannot be
   retroactively made admissible as a duplicate `1.0.0` game release.

2. Change the shared Reversi source identity to `1.1.0`, then update all
   direct test and documentation assertions. Confirm the game-master metadata,
   exported snapshot, game bundle manifest, and AI bundle manifest agree.

3. Harden the package and release workflow boundary. Build a narrow reusable
   version check from generated manifests rather than duplicating a literal in
   YAML; it must normalize only the leading `v` for the Git tag and reject
   malformed or unequal release versions. Preserve deterministic ZIP bytes for
   a fixed source and tag. Cover success, mismatch rejection, and `dev`
   behavior with focused automated tests or shell-level checks.

4. Create `.claude/skills/release-reversi-game/SKILL.md` using the verified
   package/workflow commands. Keep it specific to official Reversi game
   releases: require a merged version-bump PR, a clean current `main`, exact
   tag/source equality, no pre-existing tag or release, and explicit user
   authorization before creating the tag. It must poll the release workflow,
   retrieve release assets, verify `SHA256SUMS` and both embedded manifests,
   and report that ai-arena registration is a separate later action. Do not
   encode credentials or automatically register a game/bot or activate a
   scope.

5. Run the applicable non-AI checks, open and complete the implementation PR.
   After merge, invoke the new skill against the merged `main` to create only
   `v1.1.0`. Treat the resulting successful GitHub workflow, downloaded
   checksums, game/AI manifest values `1.1.0`, and release URLs as the skill's
   end-to-end acceptance evidence. Stop before any ai-arena registration.

## Dependencies and sequencing

Tasks 1 and 2 establish the contract and source identity before any packaging
or workflow changes. Task 3 supplies deterministic guards used by task 4.
Task 4 can be drafted in parallel with task 3 but must be finalized from the
implemented commands. Task 5 is strictly post-merge because a release tag must
point at the reviewed `main` commit, not an implementation worktree branch.

## Verification

- Run the focused Rust unit tests for `reversi-game`, `reversi-release-packager`,
  `reversi-rust-reference-player`, and affected runner E2E assertions; run
  `make verify-rust` and `make verify-workflows`.
- With the pinned ai-arena checkout, run
  `AI_ARENA_DIR=<pinned-ai-arena> RELEASE_VERSION=v1.1.0 make verify-release-artifacts`.
  Inspect both generated `manifest.json` files and assert `game_version` is
  `1.1.0`; confirm the runner match completes.
- Exercise the release-version guard with a deliberately mismatched tag value
  and verify no artifact/release is published; separately confirm `dev` local
  verification still succeeds.
- Validate the new skill with the skill validator and an independent
  read-only review of its release preflight and stopping conditions.
- After merge, use the skill to tag `v1.1.0`; confirm the GitHub release
  workflow succeeds, its two ZIP URLs and `SHA256SUMS` are downloadable, the
  local computed checksums match, and both release manifests report `1.1.0`.
