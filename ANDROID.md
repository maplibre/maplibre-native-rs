# Android Support for MapLibre Native Rust

This document describes how to use MapLibre Native with Rust on Android.

## Overview

MapLibre Native Rust now supports Android arm64-v8a (aarch64) targets with both OpenGL ES and Vulkan rendering backends.

## Requirements

### System Requirements
- Android NDK r28c or later
- Rust with Android targets installed
- Android API level 23 (Android 6.0) or higher

### Rust Setup

Install the Android target for Rust:

```bash
rustup target add aarch64-linux-android
```

### NDK Configuration

Set up your NDK path in your environment or cargo config:

```bash
export ANDROID_NDK_ROOT=/path/to/android-ndk
```

Or in `.cargo/config.toml`:

```toml
[env]
ANDROID_NDK_ROOT = "/path/to/android-ndk"
```

## Rendering Backends

MapLibre Native on Android supports two rendering backends:

### OpenGL ES (Default)
Uses OpenGL ES 3.x with EGL for context management.

Enable in `Cargo.toml`:
```toml
maplibre = { version = "...", features = ["opengl"] }
```

**System Libraries Required:**
- `libandroid.so` - Android system library
- `liblog.so` - Android logging
- `libEGL.so` - EGL context management
- `libGLESv3.so` - OpenGL ES 3.x

### Vulkan
Uses Vulkan API for rendering.

Enable in `Cargo.toml`:
```toml
maplibre = { version = "...", features = ["vulkan"] }
```

**System Libraries Required:**
- `libandroid.so` - Android system library
- `liblog.so` - Android logging
- Vulkan support (available on Android 7.0+)

## Core Library Artifacts

The build system automatically downloads pre-compiled MapLibre Native core libraries:

**OpenGL ES:**
- `libmaplibre-native-core-amalgam-android-arm64-v8a-opengl.a`

**Vulkan:**
- `libmaplibre-native-core-amalgam-android-arm64-v8a-vulkan.a`

These are downloaded from MapLibre Native releases and include all dependencies (FreeType, HarfBuzz, SQLite, ICU, etc.) merged into a single static library.

## Building for Android

### Cross-Compilation

To build your Rust project for Android:

```bash
cargo build --target aarch64-linux-android --release
```

### Using a Custom Core Library

If you want to use a locally built core library instead of downloading:

```bash
export MLN_CORE_LIBRARY_PATH=/path/to/libmaplibre-native-core-amalgam-android-arm64-v8a-opengl.a
export MLN_CORE_LIBRARY_HEADERS_PATH=/path/to/maplibre-native-headers.tar.gz
cargo build --target aarch64-linux-android
```

## Integration with Android Apps

### Using with Android NDK

When integrating with an Android app, you'll need to:

1. Build your Rust code as a dynamic library (cdylib)
2. Link against the Android system libraries
3. Load the library from your Android app using JNI

Example `Cargo.toml` for a library:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
maplibre = { version = "...", features = ["opengl"] }
```

### Minimum SDK Version

MapLibre Native requires Android API level 23 (Android 6.0) or higher.

Set this in your `AndroidManifest.xml`:

```xml
<uses-sdk android:minSdkVersion="23" />
```

## System Dependencies

### Required Android System Libraries

The following Android system libraries are automatically linked:

- **libandroid.so** - Android native app glue and system APIs
- **liblog.so** - Android logging functionality
- **libz.so** - zlib compression (provided by Android)

### Rendering Backend Libraries

**For OpenGL ES:**
- **libEGL.so** - EGL context management
- **libGLESv3.so** - OpenGL ES 3.x

**For Vulkan:**
- Vulkan loader (provided by system on Android 7.0+)

### NOT Included (handled by amalgam library)

These dependencies are already included in the amalgam static library:
- FreeType (font rendering)
- HarfBuzz (text shaping)
- SQLite (tile caching)
- ICU (internationalization)
- glslang (SPIR-V compiler, Vulkan only)

## Supported Architectures

Currently supported:
- **arm64-v8a** (aarch64) ✅

Planned for future:
- armeabi-v7a (armv7)
- x86_64
- x86

## Troubleshooting

### "unsupported target" Error

If you see:
```
unsupported target: only linux, macos, and android (arm64-v8a) are currently supported
```

Make sure you're building for a supported target:
```bash
cargo build --target aarch64-linux-android
```

### Linking Errors

If you encounter linking errors, ensure:
1. Android NDK is properly installed and ANDROID_NDK_ROOT is set
2. You're using NDK r28c or later
3. The correct rendering backend feature is enabled

### Missing System Libraries

If you get "cannot find -landroid" or similar errors:
1. Verify your NDK installation
2. Check that you're using the correct target triple
3. Ensure the NDK's toolchain is in your PATH

## References

- [MapLibre Native Android Core Library Build](https://github.com/maplibre/maplibre-native/blob/main/design-proposals/2025-10-26-android-core-library-build.md)
- [MapLibre Native](https://github.com/maplibre/maplibre-native)
- [Android NDK](https://developer.android.com/ndk)
- [Rust Android Target](https://doc.rust-lang.org/rustc/platform-support/android.html)
