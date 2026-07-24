# AI Arena Release Artifacts
> **Execution**: Use `/execute-task` to implement this plan. After implementation is complete, use `/review-task` to prepare and create the PR.

Addresses: N/A

## Objective

Turn the existing runner-verified Reversi implementation into the first real
game and AI artifacts that can be submitted to the `ai-arena` Phase 7 operator
flow. A tagged GitHub Release must publish separate, deterministic
`arena-bundle/v1` ZIPs for the Reversi game master and Rust reference AI, plus
`SHA256SUMS`.

Completion requires the exact released bytes to pass the upstream bundle
validator and complete a two-player standard Reversi match after unpacking and
running both game master and AI through WASM/WASI. Debug binaries, handwritten
test-only manifests, or a native local-subprocess staging exception do not meet
the completion boundary.

## Artifact Contract

This repository consumes the bundle schema and validator owned by `ai-arena`;
it does not fork that contract.

- Game asset: `reversi-game-1.0.0.arena.zip`
  - root `manifest.json`
  - one declared Reversi game-master WASM module
  - game identity `reversi`, exact version `1.0.0`
  - standard ruleset with `player_count = 2`
  - standard ruleset with `max_active_bots_per_owner = 3`, the smallest cap
    that permits a non-trivial Shuffle selection for a two-player game
  - sequential decision mode, the existing turn deadline, and platform-owned
    AI module/memory budgets required by the upstream schema
- AI asset: `reversi-rust-reference-ai-<release-version>.arena.zip`
  - root `manifest.json`
  - one declared `wasm32-wasip1` reference-player module
  - technical `ai_id = rust-reference` and the Reversi compatibility tuple
- Release asset: `SHA256SUMS`
  - checksum for both ZIPs; the same digest must be observed by GitHub Release,
    upstream validation, and staging submission

The user-visible bot/player name is not part of the AI bundle. It remains
control-plane metadata entered when a user creates a bot in `ai-arena`.
Game and AI bundles remain separate so they can be updated, owned, submitted,
and retired independently.

## Existing Implementation References

- `docs/project-plan.md`
  - platform/repository ownership and ordinary registration requirement, lines 5-30, 50-58
  - current Phase 1/2 completion and intent, lines 137-159
- `docs/specs/platform-boundary.md`
  - consume reviewed upstream contracts instead of copying platform behavior, lines 23-34
- `docs/specs/tagged-runner-consumption.md`
  - current native local-subprocess, fresh-run-only dev overlay, lines 22-45
- `docs/specs/reversi-ai-player.md`
  - current WASM reference-player delivery assumptions, lines 8-23
- `games/reversi/src/lib.rs`
  - `GAME_ID`, `GAME_VERSION`, and `RULESET_VERSION`, lines 9-11
- `games/reversi/src/gamemaster.rs`
  - two-player check, resume-capable initialization, decision settings, lines 15, 65-98, 140-157
- `cmd/reversi-gamemaster/src/main.rs`
  - current stdio JSON-RPC binary and method dispatch, lines 21-139
- `players/rust-reference/src/lib.rs`
  - canonical sidecar manifest construction, lines 32-53
- `tools/rust-ci.sh`
  - current player-only WASM build, lines 24-25
- `e2e/reversi-runner/src/lib.rs`
  - current debug builds/cached WASM/handwritten native manifest, lines 387-479
- `.github/workflows/rust-ci.yml`
  - current verification-only workflow with no release publication, lines 1-92

## Code Change Map

- `docs/specs/submittable-artifacts.md` (NEW)
  - Reversi-owned manifest values, asset names, deterministic build/release,
    compatibility and checksum behavior
- `docs/specs/reversi-game-master.md` (MODIFY)
  - official WASM/WASI game-master delivery and runtime limits
- `docs/specs/reversi-ai-player.md` (MODIFY)
  - bundle delivery while preserving technical AI identity vs bot name
- `docs/specs/tagged-runner-consumption.md` (MODIFY)
  - bundle validator/runner tag and exact-archive verification path
- `docs/specs/platform-boundary.md` (MODIFY)
  - upstream schema/validator ownership and release-asset handoff
- `docs/specs/verification-assets.md` (MODIFY)
  - release bundle vs debug fixture responsibilities
- `docs/project-plan.md` (MODIFY)
  - record deployable/registered artifact evidence without changing later visualizer milestones
- `README.md` (MODIFY)
  - replace absolute native-command guidance with tagged bundle validation/run guidance
- `tools/release-packager/` (NEW)
  - repository-owned deterministic manifest generation, ZIP creation, and checksum verification;
    reuse game constants and `players/rust-reference::sidecar_manifest()` rather than copy JSON
- `Makefile` (MODIFY)
  - canonical build/verify release-artifact targets
- `tools/rust-ci.sh` (MODIFY)
  - release-mode game-master and AI WASM verification
- `rust-toolchain.toml` / Cargo workspace metadata (MODIFY if required)
  - pin the game-master `wasm32-wasip1` target/tooling
- `cmd/reversi-gamemaster/` (MODIFY only as required)
  - make the existing protocol implementation build and run under WASI without changing game rules
- `e2e/reversi-runner/src/lib.rs` (MODIFY)
  - consume the packager's exact ZIPs instead of handwritten manifest copies
- `.github/workflows/rust-ci.yml` (MODIFY)
  - build and validate release bundles on PR/main as ephemeral Actions artifacts
- `.github/workflows/release.yml` (NEW)
  - pinned tag/manual release workflow that publishes both ZIPs and `SHA256SUMS`

## Spec Changes

- The Reversi game master becomes an official WASM artifact; native
  local-subprocess remains a development-only convenience.
- `standard` fixes player count 2 and per-owner active-bot limit 3 in the game
  manifest; those values must match Reversi code/spec expectations.
- Artifact release version and AI bot identity are independent from the
  game compatibility tuple.
- The GitHub Release asset and the submitted platform artifact are byte-for-byte
  identical and identified by SHA-256.

## Sub-tasks

- [ ] After the upstream schema/validator is available, update specs first and
      pin the exact upstream tag/contract consumed by this repository.
- [ ] Make `reversi-gamemaster` build and pass its logical protocol checks as
      `wasm32-wasip1`.
- [ ] Add one canonical manifest generator and parity tests for game constants,
      ruleset policy, and the existing reference-AI manifest.
- [ ] Add deterministic release-mode builds and ZIP/checksum packaging.
- [ ] Replace handwritten E2E manifests with exact packaged-artifact inputs and
      cover fresh run, resume/replay compatibility required by the upstream
      official game descriptor, and a completed two-AI standard match.
- [ ] Run the upstream validator against both assets, including digest and
      declared size/hash checks.
- [ ] Add PR build/verification and tag/manual GitHub Release publication.
- [ ] Document the release tag, asset URLs, and digests used by the `ai-arena`
      staging acceptance flow.

## Dependencies and Parallelism

- depends on: `ai-arena/docs/exec-plan/todo/0100-phase7-submission-operations-01-artifact-bundle-and-wasm-game-runtime.md`
- blocks: `ai-arena/docs/exec-plan/todo/0104-phase7-submission-operations-05-deploy-topology-and-reversi-staging.md`
- [parallel] Game-master WASM compatibility work and deterministic packager
  scaffolding can proceed after the upstream manifest schema is fixed.
- [parallel] Documentation and CI dry-run work can proceed while exact-archive
  E2E is being added.

## Verification

- `make verify-rust`
- new canonical `make build-release-artifacts` and
  `make verify-release-artifacts` targets
- repeated clean builds produce the same ZIP content/digests for the same source/tag
- upstream validator accepts both assets and rejects tampered copies
- released game WASM starts through the platform game-master adapter
- two separately named bots may use the released AI bundle and complete a
  standard two-player match through the released game bundle
- exact bundles support the upstream fresh/resume/replay contract required for registration
- PR CI publishes ephemeral Actions artifacts only; tag/manual release creates
  the GitHub Release and checksum assets
- applicable workflow lint and pinned-action checks pass

## Risks and Mitigations

- Game-master WASI compilation exposes host assumptions hidden by native E2E.
  - Mitigation: make WASI build/start/protocol tests an early gate and keep core
    rule changes out of scope unless compilation evidence requires them.
- Generated manifests drift from game/AI constants.
  - Mitigation: generate from typed sources and parity-test the emitted schema.
- Release ZIPs differ from E2E fixtures.
  - Mitigation: E2E must unpack the exact packager output used by the release job.
- Cross-repository `latest` dependencies make staging irreproducible.
  - Mitigation: pin upstream runner/validator and downstream release tags and record URLs/digests.

## Design Decisions

- Both official Reversi deliverables are WASM-only `arena-bundle/v1` ZIPs.
- Native game-master execution remains dev-only and is not a staging exception.
- `standard` owns `player_count = 2` and `max_active_bots_per_owner = 3`.
- GitHub Release assets are the canonical bytes submitted to the platform.
