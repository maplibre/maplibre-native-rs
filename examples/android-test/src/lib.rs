//! Minimal Android JNI test for MapLibre Native Rust bindings.
//!
//! This library provides a simple JNI interface to verify that:
//! 1. The Rust library builds correctly as a cdylib for Android
//! 2. The MapLibre Native core library links properly via maplibre_native crate
//! 3. Basic functionality is accessible from Java/Kotlin
//! 4. MapLibre Native C++ functions can be called successfully through the Rust bindings

use jni::objects::JClass;
use jni::sys::jstring;
use jni::JNIEnv;

/// JNI entry point to get the MapLibre version string.
///
/// This function is called from Java as: `MapLibreNative.getVersion()`
#[no_mangle]
pub extern "system" fn Java_org_maplibre_test_MapLibreNative_getVersion(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let version = format!(
        "MapLibre Native Rust {}.{}.{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH")
    );

    env.new_string(version)
        .expect("Failed to create Java string")
        .into_raw()
}

/// JNI entry point to test that MapLibre Native core is linked and functional.
///
/// This function is called from Java as: `MapLibreNative.testCore()`
/// Returns true if the core library is accessible and C++ functions execute successfully.
#[no_mangle]
pub extern "system" fn Java_org_maplibre_test_MapLibreNative_testCore(
    _env: JNIEnv,
    _class: JClass,
) -> bool {
    // Call MapLibre Native function through the Rust bindings to verify:
    // 1. The Rust cdylib compiled correctly
    // 2. The maplibre_native crate links successfully
    // 3. All required Android system libraries are found
    // 4. C++ FFI calls work at runtime through the cxx bridge

    // Call a simple MapLibre Native function that toggles a global setting
    maplibre_native::set_log_thread_enabled(true);
    maplibre_native::set_log_thread_enabled(false);

    // If we got here without crashing, the C++ library is working correctly through the Rust bindings
    true
}
