#!/bin/bash
set -euo pipefail

release_version="${1:?usage: check-release-version.sh <vX.Y.Z|dev> <game-manifest> <ai-manifest>}"
game_manifest="${2:?usage: check-release-version.sh <vX.Y.Z|dev> <game-manifest> <ai-manifest>}"
ai_manifest="${3:?usage: check-release-version.sh <vX.Y.Z|dev> <game-manifest> <ai-manifest>}"

manifest_version() {
    jq -er '.game_version | strings' "$1"
}

game_version="$(manifest_version "${game_manifest}")"
ai_version="$(manifest_version "${ai_manifest}")"

if [[ "${game_version}" != "${ai_version}" ]]; then
    echo "game and AI manifest game_version values must match" >&2
    exit 1
fi

if [[ "${release_version}" == "dev" ]]; then
    exit 0
fi

if [[ ! "${release_version}" =~ ^v([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
    echo "release version must be dev or a v-prefixed semantic version" >&2
    exit 2
fi

tag_version="${release_version#v}"
if [[ "${tag_version}" != "${game_version}" ]]; then
    echo "release tag ${release_version} does not match manifest game_version ${game_version}" >&2
    exit 1
fi
