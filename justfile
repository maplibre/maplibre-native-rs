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
    @cargo metadata --format-version 1 | jq -e -r '.packages | map(select(.name == "{{package}}")) | first | .{{field}} // error("Field \"{{field}}\" is missing in Cargo.toml for package {{package}}")'

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
      libjpeg-dev \
      libpng-dev \
      libsqlite3-dev \
      libuv1-dev \
      libwebp-dev \
      libz-dev

# Install macOS dependencies via Homebrew
[macos]
install-dependencies backend='vulkan':
    brew install \
        {{if backend == 'vulkan' {'molten-vk vulkan-headers'} else {''} }} \
        curl \
        glfw \
        jpeg \
        libpng \
        sqlite \
        libuv \
        webp \
        zlib \
        icu4c
    brew link icu4c --force

# Show current maplibre-native dependency information
maplibre-native-info: (assert-cmd "curl") (assert-cmd "jq")
    #!/usr/bin/env bash
    set -euo pipefail

    export MLN_REPO="`{{just_executable()}} get-crate-field 'metadata.mln.repo'`"
    export MLN_CORE_RELEASE_SHA="`{{just_executable()}} get-crate-field 'metadata.mln.release'`"

    echo "Github Repo: ${MLN_REPO}"
    echo "Release: ${MLN_CORE_RELEASE_SHA}"

    COMMIT_INFO=$(curl -s "https://api.github.com/repos/$MLN_REPO/commits/$MLN_CORE_RELEASE_SHA" 2>/dev/null)
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

    export MLN_REPO="$({{just_executable()}} get-crate-field 'metadata.mln.repo')"
    export MLN_CORE_RELEASE_SHA="$({{just_executable()}} get-crate-field 'metadata.mln.release')"

    # Hit the GitHub releases API for maplibre-native and pull the latest
    # releases, avoiding drafts and prereleases.
    RELEASES_URL="https://api.github.com/repos/$MLN_REPO/releases?per_page=200"

    MLN_RELEASES=$(mktemp)
    trap 'rm -f "$MLN_RELEASES"' EXIT

    curl -s "$RELEASES_URL" | jq 'map(select((.draft | not) and (.prerelease | not))) | sort_by(.published_at) | reverse' > "$MLN_RELEASES"

    if [[ $(jq 'length' "$MLN_RELEASES") -eq 0 ]]; then
        echo "ERROR: No releases found for GitHub repo $MLN_REPO"
        exit 1
    fi

    LATEST_MLN_CORE_RELEASE_SHA=$(jq -r --arg prefix "core-" 'map(select(.tag_name | startswith($prefix))) | .[0].tag_name' "$MLN_RELEASES")

    if [[ -z "$LATEST_MLN_CORE_RELEASE_SHA" || "$LATEST_MLN_CORE_RELEASE_SHA" == "null" ]]; then
        echo "ERROR: no Maplibre Native Core release found"
        echo "Release tags found:"
        jq -r '.[].tag_name' "$MLN_RELEASES"
        exit 1
    fi

    if [[ "$MLN_CORE_RELEASE_SHA" == "$LATEST_MLN_CORE_RELEASE_SHA" ]]; then
        echo "Already up to date: $LATEST_MLN_CORE_RELEASE_SHA"
    else
        echo "Updating Maplibre Native Core from $MLN_CORE_RELEASE_SHA to $LATEST_MLN_CORE_RELEASE_SHA"
        sed -i.tmp -E "/\[package\.metadata\.mln\]/,/^\[/{s/release\s*=\s*\"[^\"]+\"/release = \"$LATEST_MLN_CORE_RELEASE_SHA\"/}" Cargo.toml && \
        rm -f Cargo.toml.tmp
        sed -i.tmp "s/const MLN_REVISION: &str = \"[^\"]*\"/const MLN_REVISION: \&str = \"$LATEST_MLN_CORE_RELEASE_SHA\"/" build.rs
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
