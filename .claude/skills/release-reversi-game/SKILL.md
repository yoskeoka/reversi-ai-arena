---
name: release-reversi-game
description: Publish one approved, merged Reversi game release with a source-aligned semantic version.
---

# Release Reversi Game

Use this skill only for an official `reversi-ai-arena` game release after its
version-bump PR has merged. It publishes release artifacts only; it never
registers a game or bot with ai-arena and never activates a scope.

## Required stopping conditions

Stop and request user direction before proceeding if any of these is not
explicit:

- the semantic version to release;
- the intended external GitHub repository and release target;
- authorization to create the immutable Git tag and GitHub Release.

Do not store, print, or modify credentials. Use the existing authenticated Git
and GitHub CLI session only after authorization.

## Preflight

From a clean worktree on current `main`:

1. Confirm the version-bump PR is merged and no local changes are present.
2. Read `games/reversi/src/lib.rs` and obtain `GAME_VERSION`.
3. Require the requested tag to be exactly `v${GAME_VERSION}`.
4. Fetch tags and releases. Stop if that tag or GitHub Release already exists.
5. Run the verified local release-artifact command against the pinned ai-arena
   checkout:

   ```sh
   AI_ARENA_DIR=<pinned-ai-arena> RELEASE_VERSION=v${GAME_VERSION} make verify-release-artifacts
   ```

   Confirm both embedded `manifest.json` files report `${GAME_VERSION}` and
   that `SHA256SUMS` verifies the two ZIP files.

## Publication and verification

After the user authorizes tagging, create and push only the exact annotated
`v${GAME_VERSION}` tag on the reviewed `main` commit. Poll the
`release-artifacts` GitHub workflow until it has a terminal result. It must
succeed before continuing.

Download the published game ZIP, AI ZIP, and `SHA256SUMS` from the GitHub
Release. Locally compute and compare both SHA-256 values, then inspect each
embedded `manifest.json` and require `game_version` to equal `${GAME_VERSION}`.
Report the release URL, both asset URLs, workflow URL, checksums, and manifest
versions.

ai-arena registration, staging activation, and staging matches are separate
later actions and must not be performed by this skill.
