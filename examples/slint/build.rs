fn main() {
    slint_build::compile("main.slint").expect("Slint build failed");

    // Must be in the main binary build.rs
    println!("cargo:rustc-link-arg=-Wl,-rpath,/home/martin/GIT/maplibre-native/vendor/wgpu-native/target/x86_64-unknown-linux-gnu/release");
}
