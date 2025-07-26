#!/usr/bin/env just --justfile

@_default:
    just --list

# Build the library
build backend="vulkan":
    RUSTFLAGS='-D warnings' cargo build --workspace --features {{backend}} --all-targets

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

# Lint the project
ci-lint: rust-info test-fmt clippy

# Run all tests as expected by CI
ci-test backend: rust-info test-fmt clippy (build backend) (test backend) (test-doc backend)

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

    CURRENT_SHA=$(grep -o 'const MLN_REVISION: &str = "[^"]*"' build.rs | grep -o '"[^"]*"' | tr -d '"')
    echo "Current SHA: $CURRENT_SHA"

    COMMIT_INFO=$(curl -s "https://api.github.com/repos/maplibre/maplibre-native/commits/$CURRENT_SHA" 2>/dev/null)
    if [[ "$COMMIT_INFO" != "null" ]]; then
        echo "Message: $(echo "$COMMIT_INFO" | jq -r '.commit.message' | head -n1)"
        echo "Date: $(echo "$COMMIT_INFO" | jq -r '.commit.author.date')"
    fi

# Run all tests
test-all:
    cargo test --all-targets --workspace

# Run testcases against a specific backend
test backend="vulkan":
    cargo test --all-targets --features {{backend}} --workspace

# Run all tests and accept the changes. Requires cargo-insta to be installed.
test-accept:
    cargo insta test --accept

# Test documentation
test-doc backed="vulcan":
    RUSTDOCFLAGS="-D warnings" cargo test --doc --features {{backend}}
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --features {{backend}}

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

    # Get all core tags and find the one with latest commit date
    TAGS_RESPONSE=$(curl -s "https://api.github.com/repos/maplibre/maplibre-native/tags?per_page=200")
    if echo "$TAGS_RESPONSE" | jq -e '.message' >/dev/null 2>&1; then
        echo "GitHub API error: $(echo "$TAGS_RESPONSE" | jq -r '.message')"
        exit 1
    fi

    # GitHubs ordering is publish based, not commit date
    CORE_TAGS=$(echo "$TAGS_RESPONSE" | jq -r '.[] | select(.name | startswith("core-")) | .name')
    if [[ -z "$CORE_TAGS" ]]; then
        echo "No core releases found"
        exit 1
    fi

    LATEST_COMMIT_DATE=""
    TARGET_SHA=""

    for tag in $CORE_TAGS; do
        sha=$(echo "$tag" | sed 's/^core-//')
        commit_date=$(curl -s "https://api.github.com/repos/maplibre/maplibre-native/commits/$sha" | jq -r '.commit.author.date')

        if [[ -z "$LATEST_COMMIT_DATE" ]] || [[ "$commit_date" > "$LATEST_COMMIT_DATE" ]]; then
            LATEST_COMMIT_DATE="$commit_date"
            TARGET_SHA="$sha"
        fi
    done

    CURRENT_SHA=$(grep -o 'const MLN_REVISION: &str = "[^"]*"' build.rs | grep -o '"[^"]*"' | tr -d '"')

    if [[ "$CURRENT_SHA" == "$TARGET_SHA" ]]; then
        echo "Already up to date: $TARGET_SHA"
    else
        echo "Updating from $CURRENT_SHA to $TARGET_SHA"
        sed -i.tmp "s/const MLN_REVISION: &str = \"[^\"]*\"/const MLN_REVISION: \&str = \"$TARGET_SHA\"/" build.rs
        rm -f build.rs.tmp
    fi

# Ensure that a certain command is available
[private]
assert $COMMAND:
    @if ! type "{{COMMAND}}" > /dev/null; then \
        echo "Command '{{COMMAND}}' could not be found. Please make sure it has been installed on your computer." ;\
        exit 1 ;\
    fi
