# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.2](https://github.com/maplibre/maplibre-native-rs/compare/v0.4.1...v0.4.2) - 2025-12-08

### Other

- *(deps)* bump the all-actions-version-updates group with 4 updates ([#113](https://github.com/maplibre/maplibre-native-rs/pull/113))

## [0.4.1](https://github.com/maplibre/maplibre-native-rs/compare/v0.4.0...v0.4.1) - 2025-10-06

### Other

- do a docs pass around the readme and doc comments ([#87](https://github.com/maplibre/maplibre-native-rs/pull/87))
- *(deps)* bump taiki-e/install-action from 2.62.11 to 2.62.20 in the all-actions-version-updates group ([#89](https://github.com/maplibre/maplibre-native-rs/pull/89))

## [0.4.0](https://github.com/maplibre/maplibre-native-rs/compare/v0.3.1...v0.4.0) - 2025-10-02

### Added

- *(pool)* add an example how to use the pool ([#88](https://github.com/maplibre/maplibre-native-rs/pull/88))
- *(renderer)* [**breaking**] Rework the builder interface and document it ([#85](https://github.com/maplibre/maplibre-native-rs/pull/85))
- *(renderer)* Rework the image rendering API ([#82](https://github.com/maplibre/maplibre-native-rs/pull/82))

### Other

- *(lints)* Remove unused variables ([#84](https://github.com/maplibre/maplibre-native-rs/pull/84))
- *(deps)* update image requirement from 0.24 to 0.25 in the all-cargo-version-updates group ([#86](https://github.com/maplibre/maplibre-native-rs/pull/86))
- add MapLibre Contributors to the authors ([#81](https://github.com/maplibre/maplibre-native-rs/pull/81))
- *(deps)* bump taiki-e/install-action from 2.62.0 to 2.62.11 in the all-actions-version-updates group ([#79](https://github.com/maplibre/maplibre-native-rs/pull/79))
- *(ci)* trigger PR title workflow on synchronize event ([#77](https://github.com/maplibre/maplibre-native-rs/pull/77))

## [0.3.1](https://github.com/maplibre/maplibre-native-rs/compare/v0.3.0...v0.3.1) - 2025-09-26

### Added

- *(pool)* Add a single-threaded image rendering pool ([#68](https://github.com/maplibre/maplibre-native-rs/pull/68))

### Fixed

- *(release)* Revert permission change for CI ([#74](https://github.com/maplibre/maplibre-native-rs/pull/74))

### Other

- *(release)* another test if we can reduce our permissions for releases ([#75](https://github.com/maplibre/maplibre-native-rs/pull/75))

## [0.3.0](https://github.com/maplibre/maplibre-native-rs/compare/v0.2.0...v0.3.0) - 2025-09-25

### Added

- *(renderer)* refactored parts of the public API to be nicer to work with and less error prawn ([#66](https://github.com/maplibre/maplibre-native-rs/pull/66))
- *(renderer)* logging via `log` from maplibre-native ([#71](https://github.com/maplibre/maplibre-native-rs/pull/71))

### Other

- *(renderer)* Add more derives and rename conversation function on `Image` ([#73](https://github.com/maplibre/maplibre-native-rs/pull/73))
- *(ci)* change to `secrets.GITHUB_TOKEN` instead of our manual secret ([#67](https://github.com/maplibre/maplibre-native-rs/pull/67))
- sort toml files ([#70](https://github.com/maplibre/maplibre-native-rs/pull/70))

## [0.2.0](https://github.com/maplibre/maplibre-native-rs/compare/v0.1.2...v0.2.0) - 2025-09-23

### Fixed

- *(api)* clearer style rendering semantics ([#63](https://github.com/maplibre/maplibre-native-rs/pull/63))

### Other

- *(deps)* bump the all-actions-version-updates group with 2 updates ([#62](https://github.com/maplibre/maplibre-native-rs/pull/62))

## [0.1.2](https://github.com/maplibre/maplibre-native-rs/compare/v0.1.1...v0.1.2) - 2025-09-18

### Added

- remove `persist-credentials` ([#60](https://github.com/maplibre/maplibre-native-rs/pull/60))
- `crates.io` trusted publishing ([#58](https://github.com/maplibre/maplibre-native-rs/pull/58))
- use dependencies MapLibre Native amalgamation instead of system libraries ([#56](https://github.com/maplibre/maplibre-native-rs/pull/56))
- add metal rendering backend support ([#55](https://github.com/maplibre/maplibre-native-rs/pull/55))
- Make opengl testcases pass and mark opengl as supported ([#41](https://github.com/maplibre/maplibre-native-rs/pull/41))
- implemented the cli example ([#11](https://github.com/maplibre/maplibre-native-rs/pull/11))

### Other

- *(sec)* pin github deps to shas ([#59](https://github.com/maplibre/maplibre-native-rs/pull/59))
- *(deps)* bump actions/checkout from 4 to 5 in the all-actions-version-updates group ([#57](https://github.com/maplibre/maplibre-native-rs/pull/57))
- migrate the version update to store metadata in the `metadata` section of `Cargo.toml` ([#54](https://github.com/maplibre/maplibre-native-rs/pull/54))
- *(ci)* cleanup CI and justfile ([#43](https://github.com/maplibre/maplibre-native-rs/pull/43))
- remove the option to `git checkout` to simplify our tool chain ([#53](https://github.com/maplibre/maplibre-native-rs/pull/53))
- finalise the onboarding into maplibre for badges ([#49](https://github.com/maplibre/maplibre-native-rs/pull/49))
- *(deps)* bump amannn/action-semantic-pull-request from 5 to 6 ([#52](https://github.com/maplibre/maplibre-native-rs/pull/52))
- *(deps)* bump actions/checkout from 4 to 5 ([#51](https://github.com/maplibre/maplibre-native-rs/pull/51))
- *(deps)* pre-commit autoupdate ([#50](https://github.com/maplibre/maplibre-native-rs/pull/50))
- Document planned windows x86/arm support ([#46](https://github.com/maplibre/maplibre-native-rs/pull/46))
- update readme for macos support ([#44](https://github.com/maplibre/maplibre-native-rs/pull/44))
- *(ci)* Add a `release-plz` workflow ([#39](https://github.com/maplibre/maplibre-native-rs/pull/39))
- *(ci)* make sure that msrv is a separate step in CI ([#42](https://github.com/maplibre/maplibre-native-rs/pull/42))
- *(ci)* add a step to require conventional pr titles ([#40](https://github.com/maplibre/maplibre-native-rs/pull/40))
- revised wording ([#38](https://github.com/maplibre/maplibre-native-rs/pull/38))
- *(deps)* bump peter-evans/create-pull-request from 6 to 7 ([#36](https://github.com/maplibre/maplibre-native-rs/pull/36))
- Document inclusion of the Maplibre Native license. (fixes #5) ([#37](https://github.com/maplibre/maplibre-native-rs/pull/37))
- *(ci)* Make sure to test all platforms in CI ([#32](https://github.com/maplibre/maplibre-native-rs/pull/32))
- migrate us to the amalgamation ([#30](https://github.com/maplibre/maplibre-native-rs/pull/30))
- update readme with platform support ([#33](https://github.com/maplibre/maplibre-native-rs/pull/33))
- *(deps)* Made sure that MLN is automatically updated once every month ([#29](https://github.com/maplibre/maplibre-native-rs/pull/29))
- prevent/autofix tabs in text ([#23](https://github.com/maplibre/maplibre-native-rs/pull/23))
- Sort justfiles alphabetically ([#22](https://github.com/maplibre/maplibre-native-rs/pull/22))
- Some initial ideas of reworking pre-compiled code download. ([#17](https://github.com/maplibre/maplibre-native-rs/pull/17))
- Minor refactor ([#21](https://github.com/maplibre/maplibre-native-rs/pull/21))
- Bump dependabot/fetch-metadata from 2.3.0 to 2.4.0 ([#20](https://github.com/maplibre/maplibre-native-rs/pull/20))
- lints ([#19](https://github.com/maplibre/maplibre-native-rs/pull/19))
- extract the `GraphicsRenderingAPI` part of #17 ([#18](https://github.com/maplibre/maplibre-native-rs/pull/18))
- add docs required by maplibre ([#15](https://github.com/maplibre/maplibre-native-rs/pull/15))
- fmt
- Refactor build.rs, bump dep ([#14](https://github.com/maplibre/maplibre-native-rs/pull/14))
- Update submodule
- reword the doccomments for the debug options ([#13](https://github.com/maplibre/maplibre-native-rs/pull/13))
- remove duplicate `MLN_WITH_OPENGL` definition ([#9](https://github.com/maplibre/maplibre-native-rs/pull/9))
- reword slightly confusing warning in build script ([#10](https://github.com/maplibre/maplibre-native-rs/pull/10))
- add more docstrings ([#12](https://github.com/maplibre/maplibre-native-rs/pull/12))
