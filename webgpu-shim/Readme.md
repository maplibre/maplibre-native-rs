# WebGPU Shim

A thin shim layer that provides **WebGPU-compatible C ABI interfaces** on top of [wgpu-native](https://github.com/gfx-rs/wgpu-native).

## Problem

`wgpu-native` uses `wgpu-core` as its foundation, which means **Textures and other resources cannot be directly shared** between:
- A Rust application using the `wgpu` crate
- A C++ library expecting the standard WebGPU interface

This crate solves that interoperability problem.

## Overview

This library:
- Downloads the official [`webgpu.h`](https://github.com/webgpu-native/webgpu-headers) header
- Generates Rust bindings using `bindgen`
- Wraps wgpu-native types to expose them with WebGPU-compatible ABIs
- Adds a thin conversion layer between wgpu and WebGPU types

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
webgpu-shim = { version = "0.1.0", path = "path/to/webgpu-shim" }
```

The crate exposes:
- WebGPU-compatible type definitions (e.g., `WGPUDevice`, `WGPUTexture`)
- Function bindings matching the WebGPU C API

## Build Requirements

- Rust 2024 edition
- `bindgen` (build dependency)
- Clang/LLVM (for bindgen)
- Network access (to download `webgpu.h` on first build)

## License

Dual-licensed under:
- Apache License, Version 2.0
- MIT License

See the [wgpu-native license](https://github.com/gfx-rs/wgpu-native) for the base code this is derived from.

## Related Projects

- [wgpu-native](https://github.com/gfx-rs/wgpu-native) - Native WebGPU implementation
- [webgpu-headers](https://github.com/webgpu-native/webgpu-headers) - Official WebGPU C API headers
- [wgpu](https://crates.io/crates/wgpu) - Rust WebGPU implementation
