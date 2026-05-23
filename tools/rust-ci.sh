#!/bin/bash
set -euo pipefail

export CARGO_HOME="${CARGO_HOME:-/tmp/reversi-ai-arena-cargo-home}"
mkdir -p "${CARGO_HOME}"

usage() {
    echo "Usage: $0 {fmt|clippy|test|wasm|e2e|verify}" >&2
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
    e2e)
        cache_root=/tmp/reversi-ai-arena-go
        gobin="${cache_root}/bin"
        gopath="${cache_root}/go"
        gomodcache="${gopath}/pkg/mod"
        gocache="${cache_root}/go-build"
        home_dir="${cache_root}/home"
        xdg_cache_home="${cache_root}/xdg-cache"
        mkdir -p "${gobin}" "${gopath}" "${gomodcache}" "${gocache}" "${home_dir}" "${xdg_cache_home}"
        env \
            GOBIN="${gobin}" \
            GOPATH="${gopath}" \
            GOMODCACHE="${gomodcache}" \
            GOCACHE="${gocache}" \
            HOME="${home_dir}" \
            XDG_CACHE_HOME="${xdg_cache_home}" \
            GOWORK=off \
            go install github.com/yoskeoka/ai-arena/cmd/arena-runner@v0.2.0
        ARENA_RUNNER_BIN="${gobin}/arena-runner" cargo test -p reversi-runner-e2e -- --ignored
        ;;
    verify)
        "$0" fmt
        "$0" clippy
        "$0" test
        "$0" wasm
        "$0" e2e
        ;;
    *)
        usage
        ;;
esac
