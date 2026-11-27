# 0040 Scope local Rust lint to affected workspace crates

> **Execution**: Use `/execute-task` to implement this plan. After implementation is complete, use `/review-task` to prepare and create the PR.

## Outcome and completion

Contributors can lint the crates and compilation targets affected by their own changes. Rust workspace changes still run the full verification job in CI, including changes confined to the shared protocol crate. The implementation is complete when the local command and shared-crate filter checks pass and the implementation PR's latest commit passes `workspace-verify`.

This is a child of [workspace plan 0048](https://github.com/yoskeoka/vibe-coding-workspace/pull/172). The [Adventure implementation](https://github.com/yoskeoka/reversi-adventure/pull/312) verified exact package selection, 17 recording-Cargo cases, the job limit, and a fresh engine/AI lint run that compiled no Godot code. Apply the verified selection and job defaults here, with an additional WASI target option. The AI Arena fixture plan can proceed independently.

## Contract and change map

- (MODIFY) `docs/specs/quality-gates.md:10–22,41–66` and `docs/specs/language-toolchain-policy.md:28–54`. Require local Clippy when Rust source, a manifest, a build script, or a consumed input changes. Select all crates affected by shared inputs. If the effect of shared dependency or toolchain changes cannot be scoped, record representative local coverage and the scope left to CI. Skip Rust lint for changes with no Rust impact. Use normal lib/bin targets for ordinary changes and all targets for test, example, bench, or related conditional code. Run WASI code on `wasm32-wasip1` and explicitly select affected non-default features. Keep applicable formatting, tests, WASM builds and E2E verification. Document `make verify-rust` as the full CI gate and the availability of the same command locally.
- (MODIFY) `Makefile:3–21` and `tools/rust-ci.sh:4–26,48–58`. Add a `rust-clippy-local` Make entrypoint and a scoped script mode. Accept a whitespace-separated list of exact workspace package names via `CLIPPY_PACKAGES`. Reject empty, unknown, and non-exact specifications before Cargo runs. Pass a separate `-p` per package and `-- --no-deps -D warnings`. Set `CLIPPY_ALL_TARGETS=1` to add `--all-targets` and `CLIPPY_TARGET=wasm32-wasip1` to add the WASI target. With no target override, use the normal host configuration. Default `CARGO_BUILD_JOBS` to `2` for the local entrypoint and honor environment and Make command-line overrides. Provide a `CARGO` override for recording the invocation. Reuse the worktree build directory and existing Cargo cache policy, and serialize Cargo runs that share a build directory. Keep `rust-clippy` and `verify-rust` as full-workspace entrypoints.
- (MODIFY) `AGENTS.md` (quality-gate discovery) and `README.md:6–10` (verification entrypoints). Link to the quality-gate spec and show examples for `reversi-game`, `aiarena-protocol`, and `reversi-rust-reference-player`. Include a direct Cargo example with `--features` and a WASI example. At handoff, record crates, relevant targets and features, and results.
- (MODIFY) `.github/workflows/rust-ci.yml:4–41,54–95`. Cover each tracked workspace member in both PR and push path filters and in `detect.full`. Add `internal/aiarena-protocol/**` to all three and include any new lint configuration in those filters. Preserve workspace Clippy, format, tests, both WASI builds, runner E2E, and verification of release artifacts against the pinned AI Arena checkout.

The package list follows `Cargo.toml` and each member manifest's `[package].name`.

| Tracked member directory | Package |
| --- | --- |
| `cmd/reversi-kifu-export` | `reversi-kifu-export` |
| `cmd/reversi-gamemaster` | `reversi-gamemaster` |
| `e2e/reversi-runner` | `reversi-runner-e2e` |
| `games/reversi` | `reversi-game` |
| `internal/aiarena-protocol` | `aiarena-protocol` |
| `players/fixture-bots` | `reversi-fixture-bots` |
| `players/rust-reference` | `reversi-rust-reference-player` |
| `tools/release-packager` | `reversi-release-packager` |

## Sequence and verification

1. After this plan merges, update the specs first. Then add the local command and documentation and update the workflow. The recording-Cargo and workflow-filter checks can run independently.
2. Use a recording Cargo substitute to check one and multiple packages, empty and unknown names, wildcard/version/path forms, default and explicit targets, default and overridden job counts, and propagation of Cargo failure. Verify invalid selections never invoke Cargo.
3. Run real Clippy for `aiarena-protocol` through the local entrypoint and for `reversi-rust-reference-player` with `CLIPPY_TARGET=wasm32-wasip1`. Run workflow lint and `actionlint`. Exercise the PR/push path patterns and `detect.full` with the single changed path `internal/aiarena-protocol/Cargo.toml`, checking that it triggers the workflow and selects `workspace-verify` without selecting the game or player filters.
4. Verify the latest implementation PR commit ran full-workspace all-target Clippy and the existing WASM, E2E and release gates. Record the evidence, delete this completed plan in the implementation PR, and return its link to workspace plan 0048.

For the plan PR, check Markdown structure, exact references, `git diff --check`, and workflow lint.
