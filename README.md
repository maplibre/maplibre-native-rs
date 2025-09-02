# MapLibre-native-rs

[![GitHub](https://img.shields.io/badge/github-maplibre/maplibre--native--rs-8da0cb?logo=github)](https://github.com/maplibre/maplibre-native-rs)
[![crates.io version](https://img.shields.io/crates/v/maplibre_native)](https://crates.io/crates/maplibre_native)
[![docs.rs](https://img.shields.io/docsrs/maplibre_native)](https://docs.rs/maplibre_native)
[![crates.io license](https://img.shields.io/crates/l/maplibre_native)](https://github.com/maplibre/maplibre-native-rs/blob/main/LICENSE-APACHE)
[![CI build](https://github.com/maplibre/maplibre-native-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/maplibre/maplibre-native-rs/actions)

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

The following platform and rendering-API combinations are supported and tested in CI:

| Platform    | Metal | Vulkan | OpenGL |
|-------------|-------|--------|--------|
| Linux x86   | ❌    | ✅     | ✅     |
| Linux ARM   | ❌    | ✅     | ✅     |
| Windows x86 | ❌    | 🟨     | 🟨     |
| Windows ARM | ❌    | 🟨     | 🟨     |
| macOS ARM   | 🟨    | 🟨[^1] | ❌     |

<sub>
✅ = IS supported and tested in CI
🟨 = SHOULD be supported, but currently is not
❌ = Not possible
</sub>

[^1]: Vulcan support on macos is provided via MoltenVK. There is a slight performance overhead for this with little upsides. Both Metal and Vulcan run through the same extensive test suite upstream. You can use Vulcan if you find a bug in the Metal implementation until we have fixed it upstream.

### Dependencies

This command will install the required dependencies on Linux or macOS for the `vulkan` backend.
You may also use it with `opengl` parameter on Linux.
It is OK to run this command multiple times for each backend.

```shell
just install-dependencies vulkan
```

### Gettting MapLibre Native Core

Since we wrap the Maplibre native library, we need this and its headers to be included.

We can get the library and headers from two places:
- <details><summary>default: downloaded from the releases page</summary>

  The specific version of [MapLibre Native](https://github.com/maplibre/maplibre-native) used is controlled by `package.metadata.mln.release` in `Cargo.toml`.
  This dependency is automatically updated via a GitHub workflow on the 1st of each month repository.
  A pull request is created if an update is available.

  </details>
- <details><summary>if the env vars cMLN_CORE_LIBRARY_PATH</code> and <code>MLN_CORE_HEADERS_PATH</code> are set: from local disk via the environment variables</summary>

  If you don't want to allow network access during buildscript execution, we allow you to download the release and tell us where you have downloaded the contents.
  You can also build from source by following the steps that maplibre-native does in CI to produce the artefacts.

  </details>

## Development

- This project is easier to develop with [just](https://github.com/casey/just#readme), a modern alternative to `make`.
  Install it with `cargo install just`.
- To get a list of available commands, run `just`.
- To run tests, use `just test`.

## Getting Involved

Join the `#maplibre-martin` slack channel at OSMUS -- automatic invite is at <https://slack.openstreetmap.us/>

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)
  at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the
Apache-2.0 license, shall be dual-licensed as above, without any
additional terms or conditions.

### MapLibre Native Licence

This crate incorporates MapLibre Native assets during compilation by downloading and statically linking them. As a result, any project using this crate must comply with the [MapLibre Native License](https://github.com/maplibre/maplibre-native/blob/main/LICENSE.md) (BSD 2-Clause) requirements for binary distribution. This includes providing proper attribution and including the license text with your distributed binaries or source code.
