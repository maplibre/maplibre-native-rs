use std::env;
use std::path::PathBuf;

fn main() {
    // Get the path to the MapLibre Native core library
    let lib_path = env::var("MLN_CORE_LIBRARY_PATH")
        .expect("MLN_CORE_LIBRARY_PATH must be set");

    let lib_path_buf = PathBuf::from(&lib_path);
    let lib_dir = lib_path_buf
        .parent()
        .expect("Invalid library path")
        .to_str()
        .expect("Invalid UTF-8 in path");

    // Tell cargo to link against the MapLibre Native core library
    println!("cargo:rustc-link-search=native={}", lib_dir);
    println!("cargo:rustc-link-lib=static=mbgl-core-amalgam");

    // Link Android system libraries
    println!("cargo:rustc-link-lib=dylib=android");
    println!("cargo:rustc-link-lib=dylib=log");
    println!("cargo:rustc-link-lib=dylib=EGL");
    println!("cargo:rustc-link-lib=dylib=GLESv3");
    println!("cargo:rustc-link-lib=dylib=z");  // zlib compression library

    // Link C++ standard library
    println!("cargo:rustc-link-lib=dylib=c++_shared");
}
