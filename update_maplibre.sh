#!/usr/bin/env bash
set -euo pipefail

CORE_RELEASE_PFX="core-"

# Get maplibre-native <owner>/<repo> from Cargo metadata
MLN_REPO=$(cargo metadata --format-version 1 --no-deps | jq -e -r '
  .packages[] |
  select(.name == "maplibre_native") |
  .metadata.mln.repo // error("MLN repo missing from Cargo metadata")
')

# Get currently-tracked maplibre-native lib-core releasefrom Cargo metadata
CURRENT_MLN_CORE_RELEASE=$(cargo metadata --format-version 1 --no-deps | jq -e -r '
  .packages[] |
  select(.name == "maplibre_native") |
  .metadata.mln.release // error("MLN release missing from Cargo metadata")
')

# Hit the GitHub releases API for maplibre-native and pull the latest
# releases, avoiding drafts and prereleases.
RELEASES_URL="https://api.github.com/repos/$MLN_REPO/releases?per_page=200"

MLN_RELEASES=$(mktemp)
trap 'rm -f "$MLN_RELEASES"' EXIT

curl -s "$RELEASES_URL" | jq '
  map(select((.draft | not) and (.prerelease | not))) |
  sort_by(.published_at) | reverse
' > "$MLN_RELEASES"

if [[ $(jq 'length' "$MLN_RELEASES") -eq 0 ]]; then
  echo "ERROR: No releases found for $MLN_REPO"
  exit 1
fi

LATEST_MLN_CORE_RELEASE=$(jq -r --arg prefix "$CORE_RELEASE_PFX" '
  map(select(.tag_name | startswith($prefix))) |
  .[0].tag_name
' "$MLN_RELEASES")

if [[ -z "$LATEST_MLN_CORE_RELEASE" || "$LATEST_MLN_CORE_RELEASE" == "null" ]]; then
  echo "ERROR: no Maplibre Native Core release found"
  echo "Release tags found:"
  jq -r '.[].tag_name' "$MLN_RELEASES"
  exit 1
fi

if [[ "$LATEST_MLN_CORE_RELEASE" != "$CURRENT_MLN_CORE_RELEASE" ]]; then
  echo "Updating Maplibre Native Core from $CURRENT_MLN_CORE_RELEASE to $LATEST_MLN_CORE_RELEASE"
  sed -i.tmp -E "/\[package\.metadata\.mln\]/,/^\[/{s/release\s*=\s*\"[^\"]+\"/release = \"$LATEST_MLN_CORE_RELEASE\"/}" Cargo.toml && \
  rm -f Cargo.toml.tmp
else
  echo "Maplibre Native Core is current: $CURRENT_MLN_CORE_RELEASE"
fi
