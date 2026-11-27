#!/bin/bash
set -euo pipefail

export CARGO_HOME="${CARGO_HOME:-/tmp/reversi-ai-arena-cargo-home}"
mkdir -p "${CARGO_HOME}"

usage() {
    echo "Usage: $0 {fmt|clippy|clippy-local|test|wasm|e2e|verify}" >&2
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
    clippy-local)
        # Read all lines and treat spaces, tabs, and newlines as delimiters.
        IFS=$' \t\n' read -r -d '' -a packages <<< "${CLIPPY_PACKAGES:-}" || true
        if [[ -z "${packages[0]:-}" ]]; then
            echo "Set CLIPPY_PACKAGES to one or more exact workspace package names." >&2
            exit 1
        fi

        args=(clippy)
        for package in "${packages[@]}"; do
            # Match the [package].name of each member in the root Cargo.toml.
            case "$package" in
                reversi-kifu-export|reversi-gamemaster|reversi-runner-e2e|reversi-game|\
                aiarena-protocol|reversi-fixture-bots|reversi-rust-reference-player|reversi-release-packager)
                    args+=(-p "$package")
                    ;;
                *)
                    printf 'Unknown workspace package: %s\n' "$package" >&2
                    exit 1
                    ;;
            esac
        done
        if [[ "${CLIPPY_ALL_TARGETS:-}" == 1 ]]; then
            args+=(--all-targets)
        fi
        if [[ -n "${CLIPPY_TARGET:-}" ]]; then
            args+=(--target "$CLIPPY_TARGET")
        fi
        "${CARGO:-cargo}" "${args[@]}" -- --no-deps -D warnings
        ;;
    test)
        cargo test --workspace
        ;;
    wasm)
        cargo build --release --target wasm32-wasip1 -p reversi-gamemaster --bin reversi-gamemaster
        cargo build --release --target wasm32-wasip1 -p reversi-rust-reference-player --bin reversi-rust-reference-player
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
