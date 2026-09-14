#!/bin/bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
fixture_dir="$(mktemp -d)"
trap 'rm -rf "${fixture_dir}"' EXIT

printf '%s\n' '{"game_version":"1.1.0"}' > "${fixture_dir}/game.json"
printf '%s\n' '{"game_version":"1.1.0"}' > "${fixture_dir}/ai.json"

"${repo_root}/tools/release-packager/check-release-version.sh" \
    v1.1.0 "${fixture_dir}/game.json" "${fixture_dir}/ai.json"
"${repo_root}/tools/release-packager/check-release-version.sh" \
    dev "${fixture_dir}/game.json" "${fixture_dir}/ai.json"

if "${repo_root}/tools/release-packager/check-release-version.sh" \
    v1.2.0 "${fixture_dir}/game.json" "${fixture_dir}/ai.json"; then
    echo "expected mismatched release version to fail" >&2
    exit 1
fi

if "${repo_root}/tools/release-packager/check-release-version.sh" \
    1.1.0 "${fixture_dir}/game.json" "${fixture_dir}/ai.json"; then
    echo "expected unprefixed release version to fail" >&2
    exit 1
fi

printf '%s\n' '{"game_version":"1.2.0"}' > "${fixture_dir}/ai.json"
if "${repo_root}/tools/release-packager/check-release-version.sh" \
    dev "${fixture_dir}/game.json" "${fixture_dir}/ai.json"; then
    echo "expected mismatched manifest versions to fail" >&2
    exit 1
fi
