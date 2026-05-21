# Phase 1/2 01 Lint and Toolchain Foundation
**Execution**: Use `/execute-task` to implement this plan.

## Objective

Establish the Rust-first quality gates and repository-local developer commands
that must exist before the Reversi game master and WASM AI-player work can
proceed safely.

This plan is the prerequisite for Phase 1 and Phase 2 in
[docs/project-plan.md](../../project-plan.md). It locks the lint/build/test
surface early so later plans can add code without first debating formatting,
Clippy policy, CI ownership, or WASM build prerequisites.

## Confirmed Direction

- Rust is the mainline implementation lane for both the game master and the
  primary AI-player path.
- Go remains a support lane only for fixture helpers or reference-bot work.
- WASM is a first-class verification target for AI players, so the repository
  must expose repeatable Rust and WASM checks from the start.
- The quality-gate surface should be lightweight but strict: `fmt`, `clippy`,
  targeted tests, and workflow checks first; broader CI expansion can follow in
  later plans when more code exists.

## Code Changes

- Add repository-local Rust developer commands and scripts, likely under:
  - `Makefile`
  - `tools/`
- Add lint and formatting configuration needed by the Rust workspace, such as:
  - `clippy.toml` if required
  - rustfmt configuration only if the default style is insufficient
- Add CI workflow scaffolding for the initial Rust verification lane under
  `.github/workflows/`.
- Add any minimal verification helpers needed to keep future WASM fixture
  builds and tagged-runner e2e commands reproducible.

## Spec Changes

- Add a quality-gates spec for this repository that records:
  - required local commands
  - minimum Rust lint/test expectations
  - which checks are mandatory before PR creation
- Update [docs/specs/language-toolchain-policy.md](../../specs/language-toolchain-policy.md)
  to reflect the concrete Rust verification entrypoints and the fact that WASM
  verification is an expected consumer path, not an optional afterthought.
- If CI scope rules need clarification, update
  [docs/specs/repository-structure.md](../../specs/repository-structure.md) or
  add a narrower development-focused spec instead of leaving the policy in
  workflow YAML only.

## Design Decisions

- Keep the initial quality-gate surface Rust-centric and cheap by default.
- Treat WASM build readiness as part of the toolchain contract even before the
  main AI-player implementation lands.
- Avoid introducing repo-wide Go or browser gates for this phase; those lanes
  should stay targeted to their owning surfaces.

## Sub-tasks

- [ ] Define the quality-gate contract in specs before adding workflow files or
      helper scripts.
- [ ] [parallel] Add local developer entrypoints for Rust formatting, linting,
      testing, and targeted WASM-related checks.
- [ ] [parallel] Add the initial CI workflow for the Rust quality gate.
- [ ] [depends on: local developer entrypoints, CI workflow] Verify that the
      repository can run the same Rust checks locally and in CI without hidden
      workspace assumptions.
- [ ] [depends on: verified checks] Document the expected command sequence in
      repo specs and README-level guidance where needed.

## Parallelism

- The local command surface and the CI workflow can be prepared in parallel once
  the quality-gates spec is fixed.
- Final command naming and required-check policy must converge before this plan
  is considered complete because later plans depend on a stable verification
  baseline.

## Verification

- Rust formatting and lint commands run from the repo root.
- The initial CI workflow executes the same Rust verification path.
- The repository documents how later plans are expected to verify Rust and WASM
  code before PR creation.

## Out of Scope

- Implementing the Reversi rules engine itself
- Implementing the game-master protocol bridge
- Running `arena-runner` matches
- Implementing the Rust WASM AI player
