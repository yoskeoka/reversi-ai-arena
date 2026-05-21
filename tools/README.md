# Tooling

`tools/` owns repository-local helpers that support verification, generation,
and workflow checks.

`workflow-lint.sh` remains the baseline pre-push workflow check.
`rust-ci.sh` owns the Rust formatting, lint, test, and WASM build wrappers used
by both local commands and GitHub Actions.
