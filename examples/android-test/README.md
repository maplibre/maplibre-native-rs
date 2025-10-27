# Android Test App for MapLibre Native Rust Bindings

This is a minimal Android application to validate that the MapLibre Native Rust bindings work correctly on Android with proper linking to the native core library.

## Purpose

This test app verifies:

1. **Rust Library Builds**: The Rust bindings compile as a cdylib (`.so`) for Android
2. **Native Library Linking**: The MapLibre Native core static library links correctly
3. **System Library Resolution**: Required Android system libraries are found:
   - `libandroid.so` - Android system library
   - `liblog.so` - Android logging
   - `libEGL.so` - EGL context management
   - `libGLESv3.so` - OpenGL ES 3.x (for OpenGL backend)
4. **JNI Integration**: The native library loads and is callable from Java/Kotlin

## Prerequisites

- **Android NDK**: r28c or later
  ```bash
  # On macOS with Homebrew:
  brew install android-ndk
  export ANDROID_NDK_ROOT="/opt/homebrew/share/android-ndk"
  ```

- **Rust with Android targets**:
  ```bash
  rustup target add aarch64-linux-android
  ```

- **Android SDK**: API level 23 (Android 6.0) or higher

- **Android Studio**: For building and running the app (optional - can use Gradle directly)

## Building

### Step 1: Build the Native Library

Run the build script to compile the Rust cdylib for Android:

```bash
cd examples/android-test
./build-native.sh
```

This will:
1. Build the Rust library for `aarch64-linux-android` (arm64-v8a)
2. Download the MapLibre Native core library from GitHub releases
3. Link against Android system libraries
4. Copy the resulting `.so` to `app/src/main/jniLibs/arm64-v8a/`

The build downloads a ~400MB static library and may take several minutes on first run.

### Step 2: Build the Android App

**Option A: Using Android Studio**

1. Open `examples/android-test/` in Android Studio
2. Build and run the app on a device or emulator

**Option B: Using Gradle**

```bash
cd examples/android-test
./gradlew assembleDebug
```

Install on a connected device:
```bash
./gradlew installDebug
```

## Expected Behavior

When the app launches, it should display:

- **Status**: ✓ MapLibre Native loaded successfully!
- **Version**: MapLibre Native Rust 0.4.1 (or current version)

If the library fails to load or link, the app will display an error message.

## Project Structure

```
android-test/
├── app/
│   ├── build.gradle.kts          # App Gradle configuration
│   └── src/
│       └── main/
│           ├── AndroidManifest.xml
│           ├── java/org/maplibre/test/
│           │   ├── MainActivity.kt      # Main activity
│           │   └── MapLibreNative.kt    # JNI wrapper
│           ├── jniLibs/
│           │   └── arm64-v8a/
│           │       └── libmaplibre_android_test.so  # Built by build-native.sh
│           └── res/
│               ├── layout/activity_main.xml
│               └── values/
│                   ├── strings.xml
│                   └── styles.xml
├── src/
│   └── lib.rs                    # Rust JNI bindings
├── Cargo.toml                    # Rust library configuration
├── build-native.sh               # Build script for native library
├── build.gradle.kts              # Root Gradle configuration
├── settings.gradle.kts           # Gradle settings
└── README.md                     # This file
```

## Supported ABIs

Currently only arm64-v8a (aarch64) is supported. Future support planned for:

- armeabi-v7a (armv7)
- x86_64
- x86

## Rendering Backend

This test app uses the **OpenGL ES** rendering backend. To test with Vulkan:

1. Edit `Cargo.toml` and change the dependency:
   ```toml
   maplibre_native = { workspace = true, features = ["vulkan"] }
   ```

2. Rebuild the native library:
   ```bash
   ./build-native.sh
   ```

## Troubleshooting

### Build Fails: "unsupported target"

Ensure you're building for a supported Android architecture:
```bash
cargo build --target aarch64-linux-android --release
```

### Build Fails: "cannot find -landroid"

Make sure `ANDROID_NDK_ROOT` is set correctly:
```bash
export ANDROID_NDK_ROOT="/opt/homebrew/share/android-ndk"
```

### App Crashes on Launch: "UnsatisfiedLinkError"

The native library wasn't copied correctly. Verify:
```bash
ls -la app/src/main/jniLibs/arm64-v8a/libmaplibre_android_test.so
```

### Linking Errors During Build

Check that all required system libraries are being linked. See the main `ANDROID.md` documentation for details about required system libraries.

## Next Steps

This is a minimal validation app. For a full integration:

1. Implement actual MapLibre rendering with `ImageRenderer`
2. Add map controls and interaction
3. Test with real tile data and styles
4. Support additional ABIs
5. Add proper error handling and logging

## References

- [MapLibre Native Rust Android Documentation](../../ANDROID.md)
- [MapLibre Native Android Core Library Build Design Proposal](https://github.com/maplibre/maplibre-native/blob/main/design-proposals/2025-10-26-android-core-library-build.md)
- [JNI Documentation](https://docs.rs/jni/latest/jni/)
