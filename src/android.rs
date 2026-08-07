#[cfg(feature = "wgpu")]
unsafe extern "C" {
    fn mbgl_android_set_jvm(vm: *mut core::ffi::c_void);
}

/// Initializes MapLibre Android JNI with the process JavaVM pointer.
///
/// Call this once during Android app startup before creating map objects.
#[cfg(feature = "wgpu")]
pub fn init_android(android_java_vm: *mut core::ffi::c_void) {
    unsafe {
        mbgl_android_set_jvm(android_java_vm);
    }
    crate::init_rust_http_bridge();
}

/// No-op on Android builds without the wgpu feature.
#[cfg(not(feature = "wgpu"))]
pub fn init_android(_vm: *mut core::ffi::c_void) {}
