# Reversi release skill with version match

> **Execution**: Use `/execute-task` to implement this plan. After implementation is complete, use `/review-task` to prepare and create the PR.

## Objective and completion boundary

Use one source-controlled semantic version for each admitted Reversi game
release. It must match the Git tag, asset names, bundle manifests, and runtime
metadata. Add a repository-local skill for the safe post-merge release step.

Validate the skill by releasing `v1.1.0` from the merged commit. Confirm the
published assets. The plan ends after the GitHub release workflow and asset
checks pass.

ai-arena registration, staging activation, and staging matches are out of
scope.

The already-published `v0.1.1` assets remain immutable historical assets. Do
not overwrite their tag, release, manifest, or ai-arena registration.

Addresses: N/A - no local issue exists.

## Chosen release identity

- `reversi_game::GAME_VERSION` is the source of truth.
- A registered release uses tag `v${GAME_VERSION}`.
- Game and AI manifests, plus game-master runtime metadata, report
  `${GAME_VERSION}`.
- A changed admitted artifact selects and commits its next semantic version
  before release.
- `current_public_replay` is a backward-compatible addition. This plan uses
  version `1.1.0` and tag `v1.1.0`.

The workflow packages already-versioned source. It must not rewrite a manifest
from the tag, because that could diverge from compiled runtime metadata.

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

4. Create `.claude/skills/release-reversi-game/SKILL.md` from the verified
   package and workflow commands. Limit it to official Reversi game releases.
   Require a merged version-bump PR, clean current `main`, exact tag/source
   equality, no existing tag or release, and user authorization before tagging.
   It polls the workflow and verifies `SHA256SUMS` plus both embedded manifests.
   It reports ai-arena registration as a later action. It never stores
   credentials or registers a game/bot or activates a scope.

5. Run the applicable non-AI checks, open and complete the implementation PR.
   After merge, invoke the new skill against the merged `main` to create only
   `v1.1.0`. Treat the resulting successful GitHub workflow, downloaded
   checksums, game/AI manifest values `1.1.0`, and release URLs as the skill's
   end-to-end acceptance evidence. Stop before any ai-arena registration.

## Dependencies and sequencing

- Tasks 1 and 2 establish the contract and source identity first.
- Task 3 supplies the guards that task 4 uses.
- Task 4 may be drafted with task 3. Finish it from the implemented commands.
- Task 5 runs after merge. Its tag targets the reviewed `main` commit.

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
