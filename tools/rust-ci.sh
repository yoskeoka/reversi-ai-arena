#!/bin/bash
set -euo pipefail

usage() {
    echo "Usage: $0 {fmt|clippy|test|wasm|verify}" >&2
    exit 1
}

cmd="${1:-}"

case "$cmd" in
    fmt)
        cargo fmt --all --check
        ;;
    clippy)
        cargo clippy --workspace --all-targets -- -D warnings
        ;;
    test)
        cargo test --workspace
        ;;
    wasm)
        cargo build --target wasm32-unknown-unknown -p reversi-rust-reference-player
        ;;
    verify)
        "$0" fmt
        "$0" clippy
        "$0" test
        "$0" wasm
        ;;
    *)
        usage
        ;;
esac
