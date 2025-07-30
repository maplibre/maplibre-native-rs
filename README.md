# MapLibre-native-rs

[![GitHub](https://img.shields.io/badge/github-nyurik/maplibre--native--rs-8da0cb?logo=github)](https://github.com/nyurik/maplibre-native-rs)
[![crates.io version](https://img.shields.io/crates/v/maplibre_native)](https://crates.io/crates/maplibre_native)
[![docs.rs](https://img.shields.io/docsrs/maplibre_native)](https://docs.rs/maplibre_native)
[![crates.io license](https://img.shields.io/crates/l/maplibre_native)](https://github.com/nyurik/maplibre-native-rs/blob/main/LICENSE-APACHE)
[![CI build](https://github.com/nyurik/maplibre-native-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/nyurik/maplibre-native-rs/actions)

Rust bindings to the [MapLibre Native](https://github.com/maplibre/maplibre-native) map rendering engine.

## Usage

We use `maplibre-native`s' core build, a static, pre-compiled library.
We also allow you to compile this yourself. Instructions for this are below.

### Backend Features

This crate supports multiple rendering backends:

- `vulkan` (default on Linux/Windows): `cargo build --features vulkan`
- `opengl` (cross-platform): `cargo build --features opengl`
- `metal` (default on macOS/iOS): `cargo build --features metal`

If no feature is specified, the crate will automatically select the platform-appropriate default backend.

### Platform Support

The following platform and backend combinations are supported and tested in CI:

| Platform  | Vulkan | OpenGL | Metal |
|-----------|--------|--------|-------|
| Linux x86 | ✅     | ✅     | ❌    |
| Linux ARM | ✅     | ✅     | ❌    |
| macOS ARM | 🟨 (Molten VK)    | ❌     | 🟨    |

<sub>
✅ = IS supported and tested in CI
🟨 = SHOULD be supported, but currently is not
❌ = Not possible
</sub>

### Apt Packages

> [!NOTE]
> The version of `libicu` is quite specific.
> There [is some work ongoing upstream](https://github.com/maplibre/maplibre-native/issues/3483) to build this into the static library we pull.

```shell
sudo apt-get install -y \
  build-essential \
  libcurl4-openssl-dev \
  libglfw3-dev \
  libjpeg-dev \
  libpng-dev \
  libsqlite3-dev \
  libuv1-dev \
  libwebp-dev \
  libz-dev \
  libicu-dev

# OpenGL
sudo apt-get install -y libopengl0

# Vulkan
sudo apt-get install -y mesa-vulkan-drivers glslang-dev
```

## Development

* This project is easier to develop with [just](https://github.com/casey/just#readme), a modern alternative to `make`.
  Install it with `cargo install just`.
* To get a list of available commands, run `just`.
* To run tests, use `just test`.

### Compiling MapLibre Native

This crate relies on the MapLibre Native library, which is compiled as part of the build process:
* if the `MLN_FROM_SOURCE` environment variable is set, the build script will compile the native library from that dir.
* if this repo has been cloned, it will contain the `/maplibre-native` submodule, which will be used to compile the library. Make sure to run `git submodule update --init --recursive` to fetch the submodule.
* if there is no `/maplibre-native` submodule, the build script assumes it is being run as a dependency, and will try to download the source into the `OUT_DIR`.  Note that the first might take significant time to download and compile.

### MapLibre Native Dependency Management

The specific version of [MapLibre Native](https://github.com/maplibre/maplibre-native) used is controlled by the `MLN_REVISION` constant in `build.rs`.
This dependency is automatically updated via a GitHub workflow on the 1st of each month repository.
A pull request is created if an update is available.

## Getting Involved

Join the `#maplibre-martin` slack channel at OSMUS -- automatic invite is at <https://slack.openstreetmap.us/>

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
  at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the
Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

### MapLibre Native Licence

This crate incorporates MapLibre Native assets during compilation by downloading and statically linking them. As a result, any project using this crate must comply with the [MapLibre Native License](https://github.com/maplibre/maplibre-native/blob/main/LICENSE.md) (BSD 2-Clause) requirements for binary distribution. This includes providing proper attribution and including the license text with your distributed binaries or source code.
