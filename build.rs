//! File for defining how we download and link against `MapLibre Native`.
//! Set `MLN_CORE_LIBRARY_PATH` and `MLN_CORE_LIBRARY_HEADERS_PATH` environment variables to use a local version of maplibre
//! 
//! If you don't use the AMALGAM library define the env variable `MLN_CORE_LIBRARY_NO_AMALGAM` (value does not matter).
//! In this case all dependend libraries get linked manually
//!
//! IMPORTANT: The library path must point to the amalgan library which contains all the dependent libraries if `MLN_CORE_LIBRARY_NO_AMALGAM` is not set!
//!
//! Required libraries:
//! Fedora:
//!     - `sudo dnf install libicu-devel libglslang-devel spirv-tools-devel libpng-devel libjpeg-turbo-devel libuv-devel libwebp-devel`

use std::path::{Path, PathBuf};
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
                println!("cargo::warning=Features 'metal', 'opengl', and 'vulkan' are mutually exclusive.");

                let default_choice = if with_opengl {
                    Self::OpenGL
                } else if is_macos {
                    Self::Metal
                } else {
                    Self::Vulkan
                };
                println!("cargo::warning=Using only '{default_choice}', but this default selection may change in future releases.");
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

fn download_static(out_dir: &Path, revision: &str) -> (PathBuf, PathBuf) {
    let graphics_api = GraphicsRenderingAPI::from_selected_features();

    let target = if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        "amalgam-linux-arm64"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "amalgam-linux-x64"
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "amalgam-macos-arm64"
    } else {
        panic!(
            "unsupported target: only linux and macos are currently supported by maplibre-native"
        );
    };

    let mut tasks = Vec::new();
    let lib_filename = format!("libmaplibre-native-core-{target}-{graphics_api}.a");
    let library_file = out_dir.join(&lib_filename);
    if !library_file.is_file() {
        let static_url = format!("https://github.com/maplibre/maplibre-native/releases/download/{revision}/{lib_filename}");
        println!("cargo:warning=Downloading precompiled maplibre-native core library from {static_url} into {}", out_dir.display());
        tasks.push(Download::new(&static_url));
    }

    let headers_file = out_dir.join("maplibre-native-headers.tar.gz");
    if !headers_file.is_file() {
        let headers_url = format!("https://github.com/maplibre/maplibre-native/releases/download/{revision}/maplibre-native-headers.tar.gz");
        println!("cargo:warning=Downloading headers for maplibre-native core library from {headers_url} into {}", out_dir.display());
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
    let (library_file, headers) =match (env::var_os("MLN_CORE_LIBRARY_PATH"), env::var_os("MLN_CORE_LIBRARY_HEADERS_PATH")) {
      (Some(library_path),Some(headers_path)) => (PathBuf::from(library_path), PathBuf::from(headers_path)),
      (Some(_), None) => panic!("MLN_CORE_LIBRARY_HEADERS_PATH is not set. To compile from a local library/headers, both MLN_CORE_LIBRARY_PATH and MLN_CORE_LIBRARY_HEADERS_PATH must be set."),
      (None, Some(_)) => panic!("MLN_CORE_LIBRARY_PATH is not set. To compile from a local library/headers, both MLN_CORE_LIBRARY_PATH and MLN_CORE_LIBRARY_HEADERS_PATH must be set."),
      // Default => to downloading the static library
      (None, None) => download_static(&out_dir, MLN_REVISION),
     };
    assert!(
        library_file.is_file(),
        "The MLN library at {} must be a file. When building locally on Linux it is called libmbgl-core.a",
        library_file.display()
    );
    if let Some(_) = env::var_os("MLN_CORE_LIBRARY_HEADERS_PATH") {
        assert!(
            headers.is_file(),
            "The MLN headers at {} must be a gzip (tar.gz) file containing the headers. When building locally checkout <maplibre-native repository>/.github/workflows/core-release.yml commands how to create the header archive",
            headers.display()
        );
    } else {
        assert!(
            headers.is_file(),
            "The MLN headers at {} must be a zip file containing the headers.",
            headers.display()
        );
    }

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
    println!("cargo:rerun-if-changed=include/renderer_observer.h");
    println!("cargo:rerun-if-changed=include/map_observer.h");
    println!("cargo:rerun-if-changed=include/rust_log_observer.h");
    cxx_build::bridge("src/renderer/bridge.rs")
        .includes(include_dirs)
        .file("src/renderer/bridge.cpp")
        .flag_if_supported("-std=c++20")
        .compile("maplibre_rust_map_renderer_bindings");

    // Link mbgl-core after the bridge - or else `cargo test` won't be able to find the symbols.
    println!("cargo:rustc-link-lib=static={lib_name}");
}

fn build_mln() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let (cpp_root, include_dirs) = resolve_mln_core(&root);

    println!("cargo:rerun-if-env-changed=MLN_CORE_LIBRARY_NO_AMALGAM");
    let no_amalgam_lib = env::var_os("MLN_CORE_LIBRARY_NO_AMALGAM").is_some();

    println!(
        "cargo:warning=Using precompiled maplibre-native static library from {}",
        cpp_root.display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        cpp_root.parent().unwrap().display()
    );

    // Add system library search paths for macOS
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");
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
    if no_amalgam_lib {
        // The dependent libs are not bundled in the core lib, so we have to link manually
        // Required for mlt-cpp. Cpp root link search was already added above
        println!(
            "cargo:rustc-link-search=native={}",
            cpp_root.parent().unwrap().join("vendor").join("maplibre-tile-spec").join("cpp").display()
        );
        println!("cargo:rustc-link-lib=mbgl-harfbuzz");
        println!("cargo:rustc-link-lib=mbgl-freetype");
        println!("cargo:rustc-link-lib=mbgl-vendor-nunicode");
        println!("cargo:rustc-link-lib=mbgl-vendor-parsedate");
        println!("cargo:rustc-link-lib=mbgl-vendor-sqlite");
        println!("cargo:rustc-link-lib=mbgl-vendor-csscolorparser");
        println!("cargo:rustc-link-lib=mlt-cpp"); // provided with matlibre-native
        // println!("cargo:rustc-link-lib=utf8proc"); // sudo dnf install utf8proc-devel
        println!("cargo:rustc-link-lib=icuuc"); //sudo dnf install libicu-devel
        println!("cargo:rustc-link-lib=icudata"); //sudo dnf install libicu-devel
        println!("cargo:rustc-link-lib=icui18n"); //sudo dnf install libicu-devel
        println!("cargo:rustc-link-lib=glslang"); //sudo dnf install libglslang-devel
        println!("cargo:rustc-link-lib=glslang-default-resource-limits"); //sudo dnf install libglslang-devel
        println!("cargo:rustc-link-lib=SPIRV-Tools"); //sudo dnf install  spirv-tools-devel // Required by glslang spirv-tools-devel
        println!("cargo:rustc-link-lib=SPIRV-Tools-opt"); //sudo dnf install  spirv-tools-devel // Required by glslang spirv-tools-devel
        println!("cargo:rustc-link-lib=png"); // sudo dnf install libpng-devel
        println!("cargo:rustc-link-lib=jpeg");// sudo dnf install libjpeg-turbo-devel
        println!("cargo:rustc-link-lib=uv"); // sudo dnf install libuv-devel
        println!("cargo:rustc-link-lib=webp"); // sudo dnf install libwebp-devel
    }
    println!("cargo:rustc-link-lib=curl");
    println!("cargo:rustc-link-lib=z");
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
