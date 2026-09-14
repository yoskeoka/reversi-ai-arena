#!/bin/bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
release_version="${1:-dev}"
output_dir="${2:-${repo_root}/dist}"
ai_arena_dir="${AI_ARENA_DIR:-$(cd "${repo_root}/../.." && pwd)/ai-arena}"

"${repo_root}/tools/release-packager/package.sh" "${release_version}" "${output_dir}"
output_dir="$(cd "${output_dir}" && pwd)"
game_manifest="$(unzip -p "${output_dir}/reversi-game-${release_version}.arena.zip" manifest.json)"
ai_manifest="$(unzip -p "${output_dir}/reversi-rust-reference-ai-${release_version}.arena.zip" manifest.json)"
test "$(jq -r '.game_version' <<<"${game_manifest}")" = "$(jq -r '.game_version' <<<"${ai_manifest}")"
if [[ "${release_version}" != "dev" ]]; then
    test "$(jq -r '.game_version' <<<"${game_manifest}")" = "${release_version#v}"
fi
repeat_dir="$(mktemp -d)"
runner_output_dir="$(mktemp -d)"
trap 'rm -rf "${repeat_dir}" "${runner_output_dir}"' EXIT
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

game_bundle="${output_dir}/reversi-game-${release_version}.arena.zip"
ai_bundle="${output_dir}/reversi-rust-reference-ai-${release_version}.arena.zip"
match_id="release-artifact-bundles"
(
    cd "${ai_arena_dir}"
    GOCACHE="${GOCACHE:-/tmp/reversi-ai-arena-gocache}" go run ./cmd/arena-runner \
        --game-master-bundle "${game_bundle}" \
        --player-bundle "black=${ai_bundle}" \
        --player-bundle "white=${ai_bundle}" \
        --match-id "${match_id}" \
        --output-dir "${runner_output_dir}" \
        --log-output none \
        --match-timeout 1m
)
jq -e '.status == "completed"' "${runner_output_dir}/${match_id}/result-summary.json" >/dev/null
jq -e '.status == "completed"' "${runner_output_dir}/${match_id}/exported-snapshot.json" >/dev/null
