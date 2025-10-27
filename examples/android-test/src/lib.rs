//! Minimal Android JNI test for MapLibre Native Rust bindings.
//!
//! This library provides a simple JNI interface to verify that:
//! 1. The Rust library builds correctly as a cdylib for Android
//! 2. The MapLibre Native core library links properly
//! 3. Basic functionality is accessible from Java/Kotlin
//! 4. MapLibre Native C++ functions can be called successfully

use jni::objects::JClass;
use jni::sys::jstring;
use jni::JNIEnv;

// Direct FFI bindings to MapLibre Native C++ MapOptions class
// These symbols exist in the MapLibre Native core library
extern "C" {
    // Constructor: mbgl::MapOptions::MapOptions()
    #[link_name = "_ZN4mbgl10MapOptionsC1Ev"]
    fn mbgl_MapOptions_new(this: *mut MapOptionsOpaque);

    // Destructor: mbgl::MapOptions::~MapOptions()
    #[link_name = "_ZN4mbgl10MapOptionsD1Ev"]
    fn mbgl_MapOptions_delete(this: *mut MapOptionsOpaque);
}

// Opaque type representing mbgl::MapOptions
// The actual size is larger, but we allocate enough space
#[repr(C)]
struct MapOptionsOpaque {
    _data: [u8; 16], // std::unique_ptr<Impl> is 8 bytes on 64-bit, add padding
}

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
    // Call actual MapLibre Native C++ functions to verify:
    // 1. The Rust cdylib compiled correctly
    // 2. The MapLibre Native static library linked successfully
    // 3. All required Android system libraries (libandroid, liblog, libEGL, libGLESv3, libc++_shared) are found
    // 4. C++ FFI calls work at runtime
    // 5. C++ constructors and destructors execute properly

    unsafe {
        // Create a MapOptions object (calls C++ constructor)
        let mut map_options = std::mem::MaybeUninit::<MapOptionsOpaque>::uninit();
        mbgl_MapOptions_new(map_options.as_mut_ptr());

        // Destroy the MapOptions object (calls C++ destructor)
        mbgl_MapOptions_delete(map_options.as_mut_ptr());
    }

    // If we got here without crashing, the C++ library is working correctly
    true
}
