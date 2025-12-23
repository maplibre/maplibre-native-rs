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
NDK_TOOLCHAIN="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64"
export CC_aarch64_linux_android="$NDK_TOOLCHAIN/bin/aarch64-linux-android23-clang"
export CXX_aarch64_linux_android="$NDK_TOOLCHAIN/bin/aarch64-linux-android23-clang++"
export AR_aarch64_linux_android="$NDK_TOOLCHAIN/bin/llvm-ar"

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
