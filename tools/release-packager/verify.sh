#!/bin/bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
release_version="${1:-dev}"
output_dir="${2:-${repo_root}/dist}"
ai_arena_dir="${AI_ARENA_DIR:-$(cd "${repo_root}/../.." && pwd)/ai-arena}"

"${repo_root}/tools/release-packager/package.sh" "${release_version}" "${output_dir}"
output_dir="$(cd "${output_dir}" && pwd)"
repeat_dir="$(mktemp -d)"
trap 'rm -rf "${repeat_dir}"' EXIT
"${repo_root}/tools/release-packager/package.sh" "${release_version}" "${repeat_dir}"
for bundle in "${output_dir}"/*.arena.zip; do
    cmp "${bundle}" "${repeat_dir}/$(basename "${bundle}")"
done
(
    cd "${output_dir}"
    sha256sum --check SHA256SUMS
)
for bundle in "${output_dir}"/*.arena.zip; do
    expected="sha256:$(sha256sum "${bundle}" | awk '{print $1}')"
    actual="$(cd "${ai_arena_dir}" && GOCACHE="${GOCACHE:-/tmp/reversi-ai-arena-gocache}" go run ./cmd/arena-artifact validate "${bundle}")"
    test "${actual}" = "${expected}"
done
