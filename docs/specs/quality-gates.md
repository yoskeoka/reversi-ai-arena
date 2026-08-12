# Quality Gates

## Purpose

This repository uses a Rust-first verification baseline for the game master and
mainline AI-player surfaces. The baseline must stay cheap enough for routine
execution while still proving that repository-local commands and CI cover the
same contract.

## Required Local Commands

Run these commands from the repository root:

- `make rust-fmt`: check Rust formatting across the workspace
- `make rust-clippy`: run Clippy for all workspace targets with warnings denied
- `make rust-test`: run the workspace Rust test suite
- `make wasm-check`: build the Rust reference player for
  `wasm32-wasip1`
- `make runner-e2e`: install the pinned tagged `arena-runner` and run the
  manifest-backed Reversi end-to-end verification path
- `make verify-rust`: run the required Rust verification path for this phase
- `make verify-workflows`: run repository-local workflow lint checks

## Workflow Artifact Retention

Workflow checks expose only active work in the checkout:

- Active execution plans remain under `docs/exec-plan/todo/`, and unresolved
  local issues remain under `docs/issues/`.
- A matching `feat/<name>` or `fix/<name>` branch may close its plan by deleting
  the matching file. The workflow linter reads that deleted plan from the
  merge-base side of the branch diff to validate its completion metadata.
- Local issues explicitly listed by the deleted plan must be deleted in the
  same branch unless the PR body explains why an issue remains open.
- External GitHub issues listed by the deleted plan must have matching PR-body
  closing metadata, unless the PR body explains why an issue remains open.
- Completed plans and resolved local issues are retrieved from the plan PR,
  implementation PR, or Git history rather than a checked-out `done/`
  directory.

## Minimum Pre-PR Verification

- PRs that touch `games/reversi/**`, `players/rust-reference/**`, `cmd/**`,
  `Makefile`, `tools/rust-ci.sh`, root Rust-toolchain files, or the Rust CI
  workflow must run `make verify-rust`.
- PRs that change workflow files should also run `make verify-workflows`.
- Go and browser verification stay targeted to later phases and are not part of
  the Phase 1/2 baseline unless those owning surfaces change.

## CI Contract

- GitHub Actions must execute the same Rust verification entrypoints used
  locally instead of inventing a separate CI-only command sequence.
- Full Rust-surface changes run the complete `make verify-rust` path.
- Surface-local Rust changes may run narrower crate checks, but the Rust
  reference player lane must still include the WASM build check.
- The tagged-runner Reversi E2E path must use the same pinned external runner
  version in local verification and CI.

## WASM Readiness

- `wasm32-wasip1` is part of the repository toolchain contract.
- The Rust reference player must stay buildable for that target as a runnable
  `wasm-wasi` program.
- A failing WASM build is a verification failure for the player surface, not an
  optional follow-up task.
