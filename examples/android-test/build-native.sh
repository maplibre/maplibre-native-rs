#!/usr/bin/env bash
set -euo pipefail

# Build script for Android native library
# This script builds the Rust cdylib and copies it to the Android jniLibs directory

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/../.."

echo "Building Rust library for Android arm64-v8a..."

# Ensure Android NDK is set up
if [ -z "${ANDROID_NDK_ROOT:-}" ]; then
    echo "Error: ANDROID_NDK_ROOT not set"
    echo "Please set it to your Android NDK installation path"
    echo "  export ANDROID_NDK_ROOT=/path/to/android-ndk"
    exit 1
fi

# Set up NDK toolchain environment
NDK_TOOLCHAIN="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/darwin-x86_64"
export CC_aarch64_linux_android="$NDK_TOOLCHAIN/bin/aarch64-linux-android23-clang"
export CXX_aarch64_linux_android="$NDK_TOOLCHAIN/bin/aarch64-linux-android23-clang++"
export AR_aarch64_linux_android="$NDK_TOOLCHAIN/bin/llvm-ar"

# Use locally built Android artifacts from maplibre-native
MAPLIBRE_NATIVE_DIR="$HOME/Code/maplibre-native"
MLN_CORE_LIB="$MAPLIBRE_NATIVE_DIR/build-android-arm64-v8a-opengl/libmbgl-core-amalgam.a"
MLN_HEADERS="$MAPLIBRE_NATIVE_DIR/build-android-arm64-v8a-opengl/maplibre-native-headers.tar.gz"

if [ ! -f "$MLN_CORE_LIB" ]; then
    echo "Error: Local Android core library not found at $MLN_CORE_LIB"
    echo "Please build it first in maplibre-native:"
    echo "  cd $MAPLIBRE_NATIVE_DIR"
    echo "  cmake --preset android-arm64-v8a-opengl"
    echo "  cmake --build build-android-arm64-v8a-opengl --target mbgl-core"
    exit 1
fi

# Create a temporary headers tarball if it doesn't exist
if [ ! -f "$MLN_HEADERS" ]; then
    echo "Creating headers tarball..."
    cd "$MAPLIBRE_NATIVE_DIR"
    tar czf build-android-arm64-v8a-opengl/maplibre-native-headers.tar.gz \
        include \
        vendor/maplibre-native-base/include \
        vendor/maplibre-native-base/deps/variant/include \
        vendor/maplibre-native-base/deps/geometry.hpp/include \
        vendor/expected-lite/include
fi

export MLN_CORE_LIBRARY_PATH="$MLN_CORE_LIB"
export MLN_CORE_LIBRARY_HEADERS_PATH="$MLN_HEADERS"

# Build for Android arm64-v8a
cd "$PROJECT_ROOT"
cargo build --target aarch64-linux-android --release -p maplibre-android-test

# Copy the built library to jniLibs
LIB_SRC="target/aarch64-linux-android/release/libmaplibre_android_test.so"
LIB_DST="$SCRIPT_DIR/app/src/main/jniLibs/arm64-v8a/libmaplibre_android_test.so"

if [ -f "$LIB_SRC" ]; then
    echo "Copying library to $LIB_DST"
    cp "$LIB_SRC" "$LIB_DST"
    echo "✓ Build successful!"
else
    echo "Error: Library not found at $LIB_SRC"
    exit 1
fi

echo ""
echo "Now you can build the Android app with Android Studio or:"
echo "  cd $SCRIPT_DIR"
echo "  ./gradlew assembleDebug"
