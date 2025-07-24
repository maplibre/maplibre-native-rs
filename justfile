#!/usr/bin/env just --justfile

@_default:
    just --list

# Build the library
build:
    RUSTFLAGS='-D warnings' cargo build --workspace --all-targets

# Quick compile without building a binary
check:
    RUSTFLAGS='-D warnings' cargo check --workspace --all-targets

# Verify that the current version of the crate is not the same as the one published on crates.io
check-if-published: (assert "jq")
    #!/usr/bin/env bash
    LOCAL_VERSION="$(cargo metadata --format-version 1 | jq -r '.resolve.root | sub(".*@"; "")')"
    echo "Detected crate version:  $LOCAL_VERSION"
    CRATE_NAME="$(cargo metadata --format-version 1 | jq -r '.resolve.root | sub(".*#"; "") | sub("@.*"; "")')"
    echo "Detected crate name:     $CRATE_NAME"
    PUBLISHED_VERSION="$(cargo search ${CRATE_NAME} | grep "^${CRATE_NAME} =" | sed -E 's/.* = "(.*)".*/\1/')"
    echo "Published crate version: $PUBLISHED_VERSION"
    if [ "$LOCAL_VERSION" = "$PUBLISHED_VERSION" ]; then
        echo "ERROR: The current crate version has already been published."
        exit 1
    else
        echo "The current crate version has not yet been published."
    fi

# Run all tests as expected by CI
ci-test: rust-info test-fmt clippy build test test-doc

# Run minimal subset of tests to ensure compatibility with MSRV (Minimum Supported Rust Version). This assumes the default toolchain is already set to MSRV.
ci-test-msrv: rust-info build test

# Clean all build artifacts
clean:
    cargo clean
    rm -f Cargo.lock

# Run cargo clippy to lint the code
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# Build and open code documentation
docs:
    cargo doc --no-deps --open

# Reformat all code `cargo fmt`. If nightly is available, use it for better results
fmt:
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v cargo +nightly &> /dev/null; then
        echo 'Reformatting Rust code using nightly Rust fmt to sort imports'
        cargo +nightly fmt --all -- --config imports_granularity=Module,group_imports=StdExternalCrate
    else
        echo 'Reformatting Rust with the stable cargo fmt.  Install nightly with `rustup install nightly` for better results'
        cargo fmt --all
    fi

# Find the minimum supported Rust version (MSRV) using cargo-msrv extension, and update Cargo.toml
msrv:
    cargo msrv find --write-msrv --ignore-lockfile

package:
    cargo package

# Run the demo binary
run *ARGS:
    cargo run -p render -- {{ARGS}}

# Print Rust version information
@rust-info:
    rustc --version
    cargo --version
    echo "PWD $(pwd)"

# Show current maplibre-native dependency information
maplibre-native-info: (assert "curl") (assert "jq")
    #!/usr/bin/env bash
    set -euo pipefail

    MLN_REPO=$(cargo metadata --format-version 1 --no-deps | jq -e -r '
        .packages[] |
        select(.name == "maplibre_native") |
        .metadata.mln.repo // error("MLN repo missing from Cargo metadata")
    ')

    MLN_CORE_RELEASE=$(cargo metadata --format-version 1 --no-deps | jq -e -r '
        .packages[] |
        select(.name == "maplibre_native") |
        .metadata.mln.release // error("MLN release missing from Cargo metadata")
    ')

    echo "Github Repo: ${MLN_REPO}"
    echo "Release: ${MLN_CORE_RELEASE}"

# Run all tests
test:
    cargo test --all-targets --workspace

# Run all tests and accept the changes. Requires cargo-insta to be installed.
test-accept:
    cargo insta test --accept

# Test documentation
test-doc:
    RUSTDOCFLAGS="-D warnings" cargo test --doc
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

# Test code formatting
test-fmt:
    cargo fmt --all -- --check

test-publishing:
    cargo publish --dry-run

# Update all dependencies, including the breaking changes. Requires nightly toolchain (install with `rustup install nightly`)
update:
    cargo +nightly -Z unstable-options update --breaking
    cargo update

# Update maplibre-native dependency to latest core release
update-maplibre-native: (assert "curl") (assert "jq")
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
        echo "ERROR: No releases found for GitHub repo $MLN_REPO"
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

# Ensure that a certain command is available
[private]
assert $COMMAND:
    @if ! type "{{COMMAND}}" > /dev/null; then \
        echo "Command '{{COMMAND}}' could not be found. Please make sure it has been installed on your computer." ;\
        exit 1 ;\
    fi
