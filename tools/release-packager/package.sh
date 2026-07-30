#!/bin/bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
release_version="${1:?usage: package.sh <release-version> [output-dir]}"
output_dir="${2:-${repo_root}/dist}"
epoch="${SOURCE_DATE_EPOCH:-315532800}" # 1980-01-01, the ZIP epoch
if [[ "${epoch}" != "315532800" ]]; then
    echo "SOURCE_DATE_EPOCH must be 315532800 for reproducible ZIP timestamps" >&2
    exit 2
fi

mkdir -p "${output_dir}"
output_dir="$(cd "${output_dir}" && pwd)"
stage="$(mktemp -d)"
trap 'rm -rf "${stage}"' EXIT

export CARGO_HOME="${CARGO_HOME:-/tmp/reversi-ai-arena-cargo-home}"
cargo build --release --target wasm32-wasip1 -p reversi-gamemaster --bin reversi-gamemaster
cargo build --release --target wasm32-wasip1 -p reversi-rust-reference-player --bin reversi-rust-reference-player

package_bundle() {
    local artifact_name="$1"
    local manifest_kind="$2"
    local module_source="$3"
    local module_name="$4"
    local bundle_dir="${stage}/${artifact_name}"
    mkdir -p "${bundle_dir}"
    cargo run --quiet -p reversi-release-packager -- "${manifest_kind}" > "${bundle_dir}/manifest.json"
    cp "${module_source}" "${bundle_dir}/${module_name}"
    touch -t 198001010000 "${bundle_dir}/manifest.json" "${bundle_dir}/${module_name}"
    (
        cd "${bundle_dir}"
        TZ=UTC zip -X -q -0 -FS "${output_dir}/${artifact_name}" manifest.json "${module_name}"
    )
}

package_bundle \
    "reversi-game-${release_version}.arena.zip" \
    game-manifest \
    "${repo_root}/target/wasm32-wasip1/release/reversi-gamemaster.wasm" \
    reversi-gamemaster.wasm
package_bundle \
    "reversi-rust-reference-ai-${release_version}.arena.zip" \
    ai-manifest \
    "${repo_root}/target/wasm32-wasip1/release/reversi-rust-reference-player.wasm" \
    rust-reference-ai.wasm

(
    cd "${output_dir}"
    sha256sum "reversi-game-${release_version}.arena.zip" "reversi-rust-reference-ai-${release_version}.arena.zip" > SHA256SUMS
)
