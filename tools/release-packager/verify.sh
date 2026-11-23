#!/bin/bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
release_version="${1:-dev}"
output_dir="${2:-${repo_root}/dist}"
ai_arena_dir="${AI_ARENA_DIR:?set AI_ARENA_DIR to the pinned ai-arena checkout}"
ai_arena_pin=212839f537274d8874a2e29afc2bb292cd842dd0
if [[ "$(git -C "${ai_arena_dir}" rev-parse HEAD)" != "${ai_arena_pin}" ]]; then
    echo "AI_ARENA_DIR must point at ai-arena v0.3.0 (${ai_arena_pin})" >&2
    exit 2
fi
export GOWORK=off
export GOCACHE="${GOCACHE:-/tmp/reversi-ai-arena-gocache}"

"${repo_root}/tools/release-packager/package.sh" "${release_version}" "${output_dir}"
output_dir="$(cd "${output_dir}" && pwd)"
game_manifest="$(unzip -p "${output_dir}/reversi-game-${release_version}.arena.zip" manifest.json)"
ai_manifest="$(unzip -p "${output_dir}/reversi-rust-reference-ai-${release_version}.arena.zip" manifest.json)"
test "$(jq -r '.game_version' <<<"${game_manifest}")" = "$(jq -r '.game_version' <<<"${ai_manifest}")"
if [[ "${release_version}" != "dev" ]]; then
    test "$(jq -r '.game_version' <<<"${game_manifest}")" = "${release_version#v}"
fi
verification_dir="$(mktemp -d)"
trap 'rm -rf "${verification_dir}"' EXIT
repeat_dir="${verification_dir}/repeat"
"${repo_root}/tools/release-packager/package.sh" "${release_version}" "${repeat_dir}"

game_bundle="${output_dir}/reversi-game-${release_version}.arena.zip"
ai_bundle="${output_dir}/reversi-rust-reference-ai-${release_version}.arena.zip"
go -C "${ai_arena_dir}" build -o "${verification_dir}/arena-artifact" ./cmd/arena-artifact
go -C "${ai_arena_dir}" build -o "${verification_dir}/arena-runner" ./cmd/arena-runner
for bundle in "${game_bundle}" "${ai_bundle}"; do
    cmp "${bundle}" "${repeat_dir}/$(basename "${bundle}")"
    expected="sha256:$(sha256sum "${bundle}" | awk '{print $1}')"
    actual="$("${verification_dir}/arena-artifact" validate "${bundle}")"
    test "${actual}" = "${expected}"
done
(
    cd "${output_dir}"
    sha256sum --check SHA256SUMS
)

run_match() {
    "${verification_dir}/arena-runner" \
        --game-master-bundle "${game_bundle}" \
        --player-bundle "black=${ai_bundle}" \
        --player-bundle "white=${ai_bundle}" \
        --output-dir "${verification_dir}" \
        --log-output none \
        --match-timeout 1m "$@"
}

for ruleset in standard standard-xot-large standard-xot-small; do
    run_match --match-id "${ruleset}" --ruleset "${ruleset}" --rng-seed reversi-release
    match_dir="${verification_dir}/${ruleset}"
    jq -e --arg ruleset "${ruleset}" \
        '.status == "completed" and .ruleset_version == $ruleset' \
        "${match_dir}/result-summary.json" "${match_dir}/exported-snapshot.json" >/dev/null
    jq -e --arg ruleset "${ruleset}" '
        .game.ruleset_version == $ruleset and
        (.snapshot.game_state.public_replay_turns | length) > 0 and
        ((.snapshot.game_state.opening_moves | length) == (if $ruleset == "standard" then 0 else 8 end))
    ' "${match_dir}/record.json" >/dev/null
done

run_match --match-id repeat-seed --ruleset standard-xot-small --rng-seed reversi-release
jq -e --slurp '.[0].game_state.opening_moves == .[1].game_state.opening_moves' \
    "${verification_dir}/standard-xot-small/snapshot.json" "${verification_dir}/repeat-seed/snapshot.json" >/dev/null

# v0.3.0 CLI bundle wrappers omit the optional public replay RPC.
# Confirm persisted turn and opening continuity through a WASI snapshot resume.
run_match --match-id resume --snapshot-input "${verification_dir}/standard-xot-small/snapshot.json"
jq -e --slurp '.[0].game_state == .[1].game_state' \
    "${verification_dir}/standard-xot-small/snapshot.json" "${verification_dir}/resume/snapshot.json" >/dev/null

# Keep the pinned checkout unchanged while testing its real operator services.
jq -n --arg target "$(cd "${ai_arena_dir}" && pwd)/internal/platform/service/reversi_release_test.go" \
    --arg source "${repo_root}/tools/release-packager/operator_test.go" \
    '{Replace: {($target): $source}}' > "${verification_dir}/overlay.json"
REVERSI_GAME_BUNDLE="${game_bundle}" REVERSI_AI_BUNDLE="${ai_bundle}" \
    go -C "${ai_arena_dir}" test -overlay "${verification_dir}/overlay.json" \
    -count=1 -v -run '^TestReversiReleaseCompetitions$' ./internal/platform/service
