use downloader::{Download, Downloader};
use std::env;
use std::fs;
use std::path::PathBuf;

const WEBGPU_H_COMMIT_ID: &str = "673658bc2bd70ec39fc55ebe6bb0173cf6d0a603";
const WEBGPU_H_SHA256_SUM: &str =
    "a483031c3fed05ea5dd1c74082a71676c46c5b2b820ccca10da515c033efc997";

fn main() {
    println!("cargo:rerun-if-changed=wgpu.h");
    println!("cargo:rerun-if-changed=dep/webgpu-headers/webgpu.h");
    println!("cargo:rerun-if-env-changed=BINDGEN_EXTRA_CLANG_ARGS");

    let webgpu_h_download_dir = download_webgpu_header();

    #[rustfmt::skip]
    let types_to_rename = vec![
        ("WGPUAdapter", "WGPUAdapterImpl"),
        ("WGPUBindGroup", "WGPUBindGroupImpl"),
        ("WGPUBindGroupLayout", "WGPUBindGroupLayoutImpl"),
        ("WGPUBuffer", "WGPUBufferImpl"),
        ("WGPUCommandBuffer", "WGPUCommandBufferImpl"),
        ("WGPUCommandEncoder", "WGPUCommandEncoderImpl"),
        ("WGPUComputePassEncoder", "WGPUComputePassEncoderImpl"),
        ("WGPUComputePipeline", "WGPUComputePipelineImpl"),
        ("WGPUDevice", "WGPUDeviceImpl"),
        ("WGPUInstance", "WGPUInstanceImpl"),
        ("WGPUPipelineLayout", "WGPUPipelineLayoutImpl"),
        ("WGPUQuerySet", "WGPUQuerySetImpl"),
        ("WGPUQueue", "WGPUQueueImpl"),
        ("WGPURenderBundle", "WGPURenderBundleImpl"),
        ("WGPURenderBundleEncoder", "WGPURenderBundleEncoderImpl"),
        ("WGPURenderPassEncoder", "WGPURenderPassEncoderImpl"),
        ("WGPURenderPipeline", "WGPURenderPipelineImpl"),
        ("WGPUSampler", "WGPUSamplerImpl"),
        ("WGPUShaderModule", "WGPUShaderModuleImpl"),
        ("WGPUSurface", "WGPUSurfaceImpl"),
        ("WGPUTexture", "WGPUTextureImpl"),
        ("WGPUTextureView", "WGPUTextureViewImpl"),
    ];

    let mut builder = bindgen::Builder::default()
        .header("wgpu.h")
        .clang_arg(format!("-I{}", webgpu_h_download_dir))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_item("WGPU.*")
        .allowlist_item("wgpu.*")
        .prepend_enum_name(false)
        .size_t_is_usize(true)
        .ignore_functions()
        .layout_tests(true)
        .clang_macro_fallback()
        // bindgen's automatic include-path detection shells out to a `clang`
        // binary and appends its reported system include paths as redundant
        // `-isystem` flags on top of what libclang already resolves via
        // LIBCLANG_PATH. For cross-compiled (Android) targets those duplicate
        // paths corrupt stdint.h/stddef.h parsing (e.g. "unknown type name
        // 'uint32_t'"/'size_t'"), so rely solely on the explicit --sysroot/
        // --target flags below instead.
        .detect_include_paths(false);

    // Add extra clang arguments for cross-compilation (e.g., Android)
    // Try common Android targets first since TARGET in build scripts is the host
    let possible_targets = [
        "aarch64_linux_android",
        "aarch64-linux-android",
        "armv7_linux_androideabi",
        "armv7-linux-androideabi",
        "i686_linux_android",
        "i686-linux-android",
        "x86_64_linux_android",
        "x86_64-linux-android",
    ];

    let mut found = false;
    for target in possible_targets.iter() {
        let clang_args_env = format!("BINDGEN_EXTRA_CLANG_ARGS_{}", target);
        println!("cargo:rerun-if-env-changed={}", clang_args_env);
        if let Ok(extra_args) = env::var(&clang_args_env) {
            for arg in extra_args.split_whitespace() {
                builder = builder.clang_arg(arg);
            }
            found = true;
            break;
        }
    }

    // Fall back to generic BINDGEN_EXTRA_CLANG_ARGS
    if !found {
        println!("cargo:rerun-if-env-changed=BINDGEN_EXTRA_CLANG_ARGS");
        if let Ok(extra_args) = env::var("BINDGEN_EXTRA_CLANG_ARGS") {
            for arg in extra_args.split_whitespace() {
                builder = builder.clang_arg(arg);
            }
        }
    }

    // When cross-compiling, libclang's own resource-dir auto-detection
    // (relied on for stdint.h/stddef.h builtins) has proven unreliable
    // depending on which build unit (host vs. target) this build script runs
    // as. Since LIBCLANG_PATH points at the toolchain's libclang, derive its
    // sibling `clang/<version>` resource directory explicitly instead of
    // trusting auto-detection.
    println!("cargo:rerun-if-env-changed=LIBCLANG_PATH");
    if found {
        if let Ok(libclang_path) = env::var("LIBCLANG_PATH") {
            let clang_dir = PathBuf::from(&libclang_path).join("clang");
            if let Ok(entries) = fs::read_dir(&clang_dir) {
                if let Some(version_dir) = entries.filter_map(|e| e.ok()).map(|e| e.path()).find(|p| p.is_dir()) {
                    builder = builder.clang_arg(format!("-resource-dir={}", version_dir.display()));
                }
            }
        }
    }

    for (old_name, new_name) in types_to_rename {
        let line = format!("pub type {old_name} = *const crate::{new_name};");
        builder = builder
            .blocklist_type(old_name)
            .blocklist_type(format!("{old_name}Impl"))
            .raw_line(line);
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs")).expect("Couldn't write bindings!");
}

fn download_webgpu_header() -> String {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let file = out_path.join("webgpu.h");
    if !sha256::try_digest(&file).is_ok_and(|sum| sum == WEBGPU_H_SHA256_SUM) {
        fs::create_dir_all(out_path.clone()).expect("Failed to create output directory");
        let mut downloader = Downloader::builder()
            .download_folder(&out_path)
            .build()
            .expect("Unable to build download builder");
        let downloads = downloader
            .download(&[Download::new(&format!(
                "https://github.com/webgpu-native/webgpu-headers/raw/{WEBGPU_H_COMMIT_ID}/webgpu.h"
            ))])
            .expect("Failed to download maplibre-native static lib")
            .into_iter();
        for download in downloads {
            if let Err(err) = download {
                panic!("Unexpected error from downloader: {err}");
            }
        }
        if let Err(e) = sha256::try_digest(file) {
            panic!("Unable to validate webgpu.h: {e}");
        }
    }

    let out_path = out_path.as_os_str().to_str().expect("Failed to resolve webgpu include dir");
    println!("cargo:rustc-env=WEBGPU_SHIM_WEBGPU_HEADER_INCLUDE_DIR={}", out_path);
    out_path.to_owned()
}
