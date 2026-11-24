#!/bin/bash
set -euo pipefail

output_dir="${1:?usage: fetch-xot.sh <output-dir>}"
mkdir -p "${output_dir}"
stage="$(mktemp -d "${output_dir}/.xot.XXXXXX")"
trap 'rm -rf "${stage}"' EXIT
base_url=https://berg.earthlingz.de/xot/downloads

for list in openingslarge.txt openingssmall.txt; do
    curl --fail --silent --show-error --location --proto '=https' \
        --retry 2 --max-time 60 "${base_url}/${list}" -o "${stage}/${list}"
done
for list in openingslarge.txt openingssmall.txt; do
    mv "${stage}/${list}" "${output_dir}/${list}"
done
