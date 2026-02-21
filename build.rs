//! File for defining how we download and link against `MapLibre Native`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use downloader::{Download, Downloader};

const MLN_REVISION: &str = "core-9b6325a14e2cf1cc29ab28c1855ad376f1ba4903";

/// Supported graphics rendering APIs.
#[derive(PartialEq, Eq, Clone, Copy)]
enum GraphicsRenderingAPI {
    /// [Apple's Metal API](https://developer.apple.com/metal/) (macOS/iOS only)
    Metal,
    /// [OpenGL API](https://www.opengl.org/)
    OpenGL,
    /// [Vulkan API](https://www.vulkan.org/)
    Vulkan,
}
impl GraphicsRenderingAPI {
    /// Selects the rendering API based on enabled cargo features and platform.
    ///
    /// - If one feature is enabled, it is used.
    /// - If none are enabled, defaults to Metal on macOS/iOS, Vulkan elsewhere.
    /// - If multiple are enabled, falls back to OpenGL > Metal > Vulkan, with a warning.
    fn from_selected_features() -> Self {
        let with_opengl = env::var("CARGO_FEATURE_OPENGL").is_ok();
        let with_metal = env::var("CARGO_FEATURE_METAL").is_ok();
        let with_vulkan = env::var("CARGO_FEATURE_VULKAN").is_ok();

        let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");
        let is_macos = target_os == "ios" || target_os == "macos";

        match (with_metal, with_vulkan, with_opengl) {
            (true, false, false) => Self::Metal,
            (false, true, false) => Self::Vulkan,
            (false, false, true) => Self::OpenGL,
            (false, false, false) => {
                if is_macos {
                    Self::Metal
                } else {
                    Self::Vulkan
                }
            }
            (_, _, _) => {
                // TODO: modify for better defaults
                // This might not be the best logic, but it can change at any moment because it's a fallback with a warning
                // Current logic: if opengl is enabled, always use that, otherwise pick metal on macOS and vulkan on other platforms
                println!(
                    "cargo::warning=Features 'metal', 'opengl', and 'vulkan' are mutually exclusive."
                );

                let default_choice = if with_opengl {
                    Self::OpenGL
                } else if is_macos {
                    Self::Metal
                } else {
                    Self::Vulkan
                };
                println!(
                    "cargo::warning=Using only '{default_choice}', but this default selection may change in future releases."
                );
                default_choice
            }
        }
    }
}
impl std::fmt::Display for GraphicsRenderingAPI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Metal => f.write_str("metal"),
            Self::OpenGL => f.write_str("opengl"),
            Self::Vulkan => f.write_str("vulkan"),
        }
    }
}

fn is_macos_arm64_target() -> bool {
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");
    target_os == "macos" && target_arch == "aarch64"
}

fn llvm_objcopy_path() -> Option<PathBuf> {
    if let Some(path) = env::var_os("LLVM_OBJCOPY").map(PathBuf::from) {
        if path.is_file() {
            return Some(path);
        }
    }

    if Command::new("llvm-objcopy")
        .arg("--version")
        .output()
        .is_ok()
    {
        return Some(PathBuf::from("llvm-objcopy"));
    }

    if let Ok(output) = Command::new("xcrun").args(["--find", "llvm-objcopy"]).output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            if !path.is_empty() {
                let candidate = PathBuf::from(path);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    [
        "/opt/homebrew/opt/llvm/bin/llvm-objcopy",
        "/usr/local/opt/llvm/bin/llvm-objcopy",
    ]
    .iter()
    .map(PathBuf::from)
    .find(|path| path.is_file())
}

fn collect_bridge_object_files(out_dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(out_dir)
        .expect("Failed to read OUT_DIR")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .is_some_and(|name| {
                        name.ends_with("-bridge.rs.o") || name.ends_with("-bridge.o")
                    })
        })
        .collect()
}

fn collect_required_mbgl_symbols(bridge_objects: &[PathBuf]) -> Vec<String> {
    let nm_output = Command::new("nm")
        .arg("-u")
        .args(bridge_objects)
        .output()
        .expect("Failed to inspect bridge object symbols with nm");
    assert!(
        nm_output.status.success(),
        "nm failed while inspecting bridge object symbols"
    );

    let mut required_symbols = nm_output
        .stdout
        .split(|byte| *byte == b'\n')
        .filter_map(|line| {
            std::str::from_utf8(line)
                .ok()
                .and_then(|value| value.split_whitespace().last())
                .map(ToOwned::to_owned)
        })
        .filter(|symbol| symbol.starts_with("__ZN4mbgl"))
        .collect::<Vec<_>>();
    required_symbols.sort();
    required_symbols.dedup();
    required_symbols
}

fn globalize_macos_amalgam_symbols(library_file: &Path) {
    if !is_macos_arm64_target() {
        return;
    }

    let Some(file_name) = library_file.file_name().and_then(|value| value.to_str()) else {
        return;
    };
    if !file_name.contains("amalgam-macos-arm64") {
        return;
    }

    println!("cargo:rerun-if-env-changed=LLVM_OBJCOPY");
    let Some(objcopy) = llvm_objcopy_path() else {
        panic!(
            "llvm-objcopy is required on macos arm64 to prepare maplibre amalgam symbols; set LLVM_OBJCOPY or install llvm-objcopy"
        );
    };

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is not set"));
    let bridge_objects = collect_bridge_object_files(&out_dir);
    if bridge_objects.is_empty() {
        return;
    }

    let required_symbols = collect_required_mbgl_symbols(&bridge_objects);
    if required_symbols.is_empty() {
        return;
    }

    let mut command = Command::new(objcopy);
    command.arg("--wildcard");
    for symbol in required_symbols {
        command.arg(format!("--globalize-symbol={symbol}"));
    }
    command.arg(library_file);
    let status = command.status().expect("Failed to run llvm-objcopy");
    assert!(
        status.success(),
        "llvm-objcopy failed while preparing maplibre amalgam symbols"
    );
}

fn download_static(out_dir: &Path, revision: &str) -> (PathBuf, PathBuf) {
    let graphics_api = GraphicsRenderingAPI::from_selected_features();

    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");
    let target = match (target_os.as_str(), target_arch.as_str()) {
        ("linux", "aarch64") => "amalgam-linux-arm64",
        ("linux", "x86_64") => "amalgam-linux-x64",
        ("macos", "aarch64") => "amalgam-macos-arm64",
        _ => {
            panic!(
                "unsupported target: only linux and macos are currently supported by maplibre-native"
            );
        }
    };

    let mut tasks = Vec::new();
    let lib_filename = format!("libmaplibre-native-core-{target}-{graphics_api}.a");
    let library_file = out_dir.join(&lib_filename);
    if !library_file.is_file() {
        let static_url = format!(
            "https://github.com/maplibre/maplibre-native/releases/download/{revision}/{lib_filename}"
        );
        println!(
            "cargo:warning=Downloading precompiled maplibre-native core library from {static_url} into {}",
            out_dir.display()
        );
        tasks.push(Download::new(&static_url));
    }

    let headers_file = out_dir.join("maplibre-native-headers.tar.gz");
    if !headers_file.is_file() {
        let headers_url = format!(
            "https://github.com/maplibre/maplibre-native/releases/download/{revision}/maplibre-native-headers.tar.gz"
        );
        println!(
            "cargo:warning=Downloading headers for maplibre-native core library from {headers_url} into {}",
            out_dir.display()
        );
        tasks.push(Download::new(&headers_url));
    }
    fs::create_dir_all(out_dir).expect("Failed to create output directory");
    let mut downloader = Downloader::builder()
        .download_folder(out_dir)
        .parallel_requests(
            u16::try_from(tasks.len()).expect("with the number of tasks, this cannot be exceeded"),
        )
        .build()
        .expect("Failed to create downloader");
    let downloads = downloader
        .download(&tasks)
        .expect("Failed to download maplibre-native static lib")
        .into_iter();
    for download in downloads {
        if let Err(err) = download {
            panic!("Unexpected error from downloader: {err}");
        }
    }

    (library_file, headers_file)
}

/// Extracts the headers from the downloaded tarball
fn extract_headers(headers_from: &Path, headers_to: &Path) {
    println!(
        "cargo:warning=Extracting headers for maplibre-native core library from {} into {}",
        headers_from.display(),
        headers_to.display()
    );
    let headers_file = fs::File::open(headers_from).expect("Failed to open headers file");
    let mut tar = flate2::read::GzDecoder::new(headers_file);

    if !headers_to.is_dir() {
        fs::create_dir_all(headers_to).expect("Failed to create headers directory");
    }
    let mut archive = tar::Archive::new(&mut tar);
    archive.set_overwrite(true);
    archive
        .unpack(headers_to)
        .expect("Failed to extract headers");
}

/// Get local directory or download maplibre-native into the `OUT_DIR`
///
/// Returns the path to the maplibre-native directory and an optional path to an include directorys.
fn resolve_mln_core(root: &Path) -> (PathBuf, Vec<PathBuf>) {
    let out_dir =
        PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is not set")).join("maplibre-native");

    println!("cargo:rerun-if-env-changed=MLN_CORE_LIBRARY_PATH");
    println!("cargo:rerun-if-env-changed=MLN_CORE_LIBRARY_HEADERS_PATH");
    let (library_file, headers) = match (
        env::var_os("MLN_CORE_LIBRARY_PATH"),
        env::var_os("MLN_CORE_LIBRARY_HEADERS_PATH"),
    ) {
        (Some(library_path), Some(headers_path)) => {
            (PathBuf::from(library_path), PathBuf::from(headers_path))
        }
        (Some(_), None) => panic!(
            "MLN_CORE_LIBRARY_HEADERS_PATH is not set. To compile from a local library/headers, both MLN_CORE_LIBRARY_PATH and MLN_CORE_LIBRARY_HEADERS_PATH must be set."
        ),
        (None, Some(_)) => panic!(
            "MLN_CORE_LIBRARY_PATH is not set. To compile from a local library/headers, both MLN_CORE_LIBRARY_PATH and MLN_CORE_LIBRARY_HEADERS_PATH must be set."
        ),
        // Default => to downloading the static library
        (None, None) => download_static(&out_dir, MLN_REVISION),
    };
    assert!(
        library_file.is_file(),
        "The MLN library at {} must be a file",
        library_file.display()
    );
    assert!(
        headers.is_file(),
        "The MLN headers at {} must be a zip file containing the headers",
        headers.display()
    );

    let extracted_path = out_dir.join("headers");
    extract_headers(&headers, &extracted_path);
    // Returning the downloaded file, bypassing CMakeLists.txt check
    let include_dirs = vec![
        root.join("include"),
        extracted_path
            .join("vendor")
            .join("maplibre-native-base")
            .join("include"),
        extracted_path
            .join("vendor")
            .join("maplibre-native-base")
            .join("deps")
            .join("geometry.hpp")
            .join("include"),
        extracted_path
            .join("vendor")
            .join("maplibre-native-base")
            .join("deps")
            .join("variant")
            .join("include"),
        extracted_path.join("include"),
    ];
    (library_file, include_dirs)
}

/// Gather include directories and build the C++ bridge using `cxx_build`.
fn build_bridge(lib_name: &str, include_dirs: &[PathBuf]) {
    println!("cargo:rerun-if-changed=src/renderer/bridge.rs");
    println!("cargo:rerun-if-changed=include/map_renderer.h");
    println!("cargo:rerun-if-changed=include/rust_log_observer.h");

    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");

    let mut bridge = cxx_build::bridge("src/renderer/bridge.rs");
    bridge
        .includes(include_dirs)
        .file("src/renderer/bridge.cpp")
        .flag_if_supported("-std=c++20");
    if target_os == "macos" && target_arch == "aarch64" {
        println!("cargo:rerun-if-changed=src/renderer/icu61_compat.cpp");
        bridge.file("src/renderer/icu61_compat.cpp");
    }
    bridge.compile("maplibre_rust_map_renderer_bindings");

    // Link mbgl-core after the bridge - or else `cargo test` won't be able to find the symbols.
    println!("cargo:rustc-link-lib=static={lib_name}");
}

fn build_mln() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");
    let (cpp_root, include_dirs) = resolve_mln_core(&root);
    println!(
        "cargo:warning=Using precompiled maplibre-native static library from {}",
        cpp_root.display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        cpp_root.parent().unwrap().display()
    );

    // Add system library search paths for macOS
    if target_os == "macos" {
        // Check for Homebrew installation paths
        if let Ok(homebrew_prefix) = env::var("HOMEBREW_PREFIX") {
            println!("cargo:rustc-link-search=native={homebrew_prefix}/lib");
        } else if Path::new("/opt/homebrew").exists() {
            println!("cargo:rustc-link-search=native=/opt/homebrew/lib");
        } else if Path::new("/usr/local").exists() {
            println!("cargo:rustc-link-search=native=/usr/local/lib");
        }

        // macOS system library paths
        println!("cargo:rustc-link-search=native=/usr/lib");
        println!("cargo:rustc-link-search=native=/System/Library/Frameworks");

        // Add pkg-config paths if available
        if let Ok(pkgconfig_path) = env::var("PKG_CONFIG_PATH") {
            for path in pkgconfig_path.split(':') {
                let lib_path = Path::new(path).parent().map(|p| p.join("lib"));
                if let Some(lib_path) = lib_path {
                    if lib_path.exists() {
                        println!("cargo:rustc-link-search=native={}", lib_path.display());
                    }
                }
            }
        }
    }

    // These `cargo:rustc-link-lib` must be done before curl and GL,
    // especially on Linux before 1.90 (1.90 introduced new linker on Linux)
    let lib_name = cpp_root
        .file_name()
        .expect("static library base has a file name")
        .to_string_lossy()
        .to_string()
        .replacen("lib", "", 1)
        .replace(".a", "");
    build_bridge(&lib_name, &include_dirs);
    globalize_macos_amalgam_symbols(&cpp_root);

    println!("cargo:rustc-link-lib=curl");
    println!("cargo:rustc-link-lib=z");
    if target_os == "macos" {
        println!("cargo:rustc-link-lib=sqlite3");
        println!("cargo:rustc-link-lib=icucore");
        println!("cargo:rustc-link-lib=bz2");
    }
    match GraphicsRenderingAPI::from_selected_features() {
        GraphicsRenderingAPI::Vulkan => {}
        GraphicsRenderingAPI::OpenGL => {
            println!("cargo:rustc-link-lib=GL");
            println!("cargo:rustc-link-lib=EGL");
        }
        GraphicsRenderingAPI::Metal => {
            // macOS Metal framework dependencies
            println!("cargo:rustc-link-lib=framework=Metal");
            println!("cargo:rustc-link-lib=framework=MetalKit");
            println!("cargo:rustc-link-lib=framework=QuartzCore");
            println!("cargo:rustc-link-lib=framework=Foundation");
            println!("cargo:rustc-link-lib=framework=CoreGraphics");
            println!("cargo:rustc-link-lib=framework=AppKit");
            println!("cargo:rustc-link-lib=framework=CoreLocation");
        }
    }
}

fn main() {
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    if env::var("DOCS_RS").is_ok() {
        println!("cargo:warning=Skipping build.rs when building for docs.rs");
        println!("cargo::rustc-cfg=docsrs");
        println!("cargo:rustc-check-cfg=cfg(docsrs)");
    } else {
        build_mln();
    }
}
