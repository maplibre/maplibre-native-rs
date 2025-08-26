#!/usr/bin/env just --justfile

main_crate := 'maplibre_native'

# if running in CI, treat warnings as errors by setting RUSTFLAGS and RUSTDOCFLAGS to '-D warnings' unless they are already set
# Use `CI=true just ci-test` to run the same tests as in GitHub CI.
# Use `just env-info` to see the current values of RUSTFLAGS and RUSTDOCFLAGS
ci_mode := if env('CI', '') != '' {'1'} else {''}
export RUSTFLAGS := env('RUSTFLAGS', if ci_mode == '1' {'-D warnings'} else {''})
export RUSTDOCFLAGS := env('RUSTDOCFLAGS', if ci_mode == '1' {'-D warnings'} else {''})
export RUST_BACKTRACE := env('RUST_BACKTRACE', if ci_mode == '1' {'1'} else {''})

@_default:
    {{just_executable()}} --list

# Build the project
build backend='vulkan':
    cargo build --workspace --features {{backend}} --all-targets

# Quick compile without building a binary
check:
    cargo check --workspace --all-targets

# Lint the project
ci-lint: env-info test-fmt clippy

# Run all tests as expected by CI
ci-test backend: env-info (build backend) (test backend) (test-doc backend) && assert-git-is-clean

# Run minimal subset of tests to ensure compatibility with MSRV
ci-test-msrv backend: (ci-test backend)  # for now, same as ci-test

# Clean all build artifacts
clean:
    cargo clean
    rm -f Cargo.lock

# Run cargo clippy to lint the code
clippy *args:
    cargo clippy --workspace --all-targets {{args}}

# Build and open code documentation
docs backend *args='--open':
    DOCS_RS=1 cargo doc --no-deps {{args}} --workspace --features {{backend}}

# Print environment info
env-info:
    @echo "Running {{if ci_mode == '1' {'in CI mode'} else {'in dev mode'} }} on {{os()}} / {{arch()}}"
    echo "PWD $(pwd)"
    {{just_executable()}} --version
    rustc --version
    cargo --version
    rustup --version
    @echo "RUSTFLAGS='$RUSTFLAGS'"
    @echo "RUSTDOCFLAGS='$RUSTDOCFLAGS'"
    @echo "RUST_BACKTRACE='$RUST_BACKTRACE'"

# Reformat all code `cargo fmt`. If nightly is available, use it for better results
fmt:
    #!/usr/bin/env bash
    set -euo pipefail
    if (rustup toolchain list | grep nightly && rustup component list --toolchain nightly | grep rustfmt) &> /dev/null; then
        echo 'Reformatting Rust code using nightly Rust fmt to sort imports'
        cargo +nightly fmt --all -- --config imports_granularity=Module,group_imports=StdExternalCrate
    else
        echo 'Reformatting Rust with the stable cargo fmt.  Install nightly with `rustup install nightly` for better results'
        cargo fmt --all
    fi

# Get any package's field from the metadata
get-crate-field field package=main_crate:  (assert-cmd 'jq')
    cargo metadata --format-version 1 | jq -e -r '.packages | map(select(.name == "{{package}}")) | first | .{{field}} | select(. != null)'

# Get the minimum supported Rust version (MSRV) for the crate
get-msrv package=main_crate:  (get-crate-field 'rust_version' package)

# Install Linux dependencies (Ubuntu/Debian). Supports 'vulkan' and 'opengl' backends.
[linux]
install-dependencies backend='vulkan':
    sudo apt-get update
    sudo apt-get install -y \
      {{if backend == 'opengl' {'libgl1-mesa-dev libglu1-mesa-dev'} else if backend == 'vulkan' {'mesa-vulkan-drivers glslang-dev'} else {''} }} \
      build-essential \
      libcurl4-openssl-dev \
      libglfw3-dev \
      libsqlite3-dev \
      libuv1-dev \
      libz-dev

# Install macOS dependencies via Homebrew
[macos]
install-dependencies backend='vulkan':
    brew install \
        {{if backend == 'vulkan' {'molten-vk vulkan-headers'} else {''} }} \
        curl \
        glfw \
        sqlite \
        libuv \
        zlib

# Show current maplibre-native dependency information
maplibre-native-info: (assert-cmd "curl") (assert-cmd "jq")
    #!/usr/bin/env bash
    set -euo pipefail

    CURRENT_SHA=$(grep -o 'const MLN_REVISION: &str = "[^"]*"' build.rs | grep -o '"[^"]*"' | tr -d '"')
    echo "Current SHA: $CURRENT_SHA"

    COMMIT_INFO=$(curl -s "https://api.github.com/repos/maplibre/maplibre-native/commits/$CURRENT_SHA" 2>/dev/null)
    if [[ "$COMMIT_INFO" != "null" ]]; then
        echo "Message: $(echo "$COMMIT_INFO" | jq -r '.commit.message' | head -n1)"
        echo "Date: $(echo "$COMMIT_INFO" | jq -r '.commit.author.date')"
    fi

# Find the minimum supported Rust version (MSRV) using cargo-msrv extension, and update Cargo.toml
msrv:  (cargo-install 'cargo-msrv')
    cargo msrv find --write-msrv --ignore-lockfile

package:
    cargo package

# Run cargo-release
release *args='':  (cargo-install 'release-plz')
    release-plz {{args}}

# Run the demo binary
run *ARGS:
    cargo run -p render -- {{ARGS}}

# Check semver compatibility with prior published version. Install it with `cargo install cargo-semver-checks`
semver *args:  (cargo-install 'cargo-semver-checks')
    cargo semver-checks {{args}}

# Run testcases against a specific backend
test backend='vulkan':
    cargo test --all-targets --features {{backend}} --workspace

# Run all tests and accept the changes. Requires cargo-insta to be installed.
test-accept:
    cargo insta test --accept

# Run all tests
test-all:
    cargo test --all-targets --workspace

# Test documentation generation
test-doc backend: (docs backend '')

# Test code formatting
test-fmt:
    cargo fmt --all -- --check

# Run testcases against a specific backend
test-miri backend='vulkan':
    MIRIFLAGS="" cargo miri test --all-targets --features {{backend}} --workspace

test-publishing:
    cargo publish --dry-run

# Find unused dependencies. Install it with `cargo install cargo-udeps`
udeps:  (cargo-install 'cargo-udeps')
    cargo +nightly udeps --workspace --all-targets

# Update all dependencies, including breaking changes. Requires nightly toolchain (install with `rustup install nightly`)
update:
    cargo +nightly -Z unstable-options update --breaking
    cargo update

# Update maplibre-native dependency to latest core release
update-maplibre-native: (assert-cmd "curl") (assert-cmd "jq")
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
assert-cmd command:
    @if ! type {{command}} > /dev/null; then \
        echo "Command '{{command}}' could not be found. Please make sure it has been installed on your computer." ;\
        exit 1 ;\
    fi

# Make sure the git repo has no uncommitted changes
[private]
assert-git-is-clean:
    @if [ -n "$(git status --untracked-files --porcelain)" ]; then \
      >&2 echo "ERROR: git repo is no longer clean. Make sure compilation and tests artifacts are in the .gitignore, and no repo files are modified." ;\
      >&2 echo "######### git status ##########" ;\
      git status ;\
      git --no-pager diff ;\
      exit 1 ;\
    fi

# Check if a certain Cargo command is installed, and install it if needed
[private]
cargo-install $COMMAND $INSTALL_CMD='' *args='':
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v $COMMAND > /dev/null; then
        if ! command -v cargo-binstall > /dev/null; then
            echo "$COMMAND could not be found. Installing it with    cargo install ${INSTALL_CMD:-$COMMAND} --locked {{args}}"
            cargo install ${INSTALL_CMD:-$COMMAND} --locked {{args}}
        else
            echo "$COMMAND could not be found. Installing it with    cargo binstall ${INSTALL_CMD:-$COMMAND} --locked {{args}}"
            cargo binstall ${INSTALL_CMD:-$COMMAND} --locked {{args}}
        fi
    fi
