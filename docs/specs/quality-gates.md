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
  `wasm32-unknown-unknown`
- `make verify-rust`: run the required Rust verification path for this phase
- `make verify-workflows`: run repository-local workflow lint checks

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

## WASM Readiness

- `wasm32-unknown-unknown` is part of the repository toolchain contract.
- The Rust reference player must stay buildable for that target even before the
  real AI implementation lands.
- A failing WASM build is a verification failure for the player surface, not an
  optional follow-up task.
