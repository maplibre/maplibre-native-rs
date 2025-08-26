use std::path::{Path, PathBuf};
use std::{env, fs};

use downloader::{Download, Downloader};

const MLN_REVISION: &str = "core-fe158c7e9b0b3f748f88d34ad384a7bcbc2cf903";

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

        let is_macos = cfg!(any(target_os = "ios", target_os = "macos"));

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
        "linux-arm64"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "linux-x64"
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "macos-arm64"
    } else {
        panic!(
            "unsupported target: only linux and macos are currently supported by maplibre-native"
        );
    };

    let mut tasks = Vec::new();
    let lib_filename = format!("libmaplibre-native-core-amalgam-{target}-{graphics_api}.a");
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
    println!(
        "cargo:warning=Using precompiled maplibre-native static library from {}",
        cpp_root.display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        cpp_root.parent().unwrap().display()
    );

    println!("cargo:rustc-link-lib=sqlite3");
    println!("cargo:rustc-link-lib=uv");
    println!("cargo:rustc-link-lib=icuuc");
    println!("cargo:rustc-link-lib=icui18n");
    //println!("cargo:rustc-link-lib=nu"); // todo add to docs => git clone https://bitbucket.org/alekseyt/nunicode.git && cmake .  && make && sudo make install
    println!("cargo:rustc-link-lib=jpeg");
    println!("cargo:rustc-link-lib=png");
    println!("cargo:rustc-link-lib=webp");
    println!("cargo:rustc-link-lib=curl");
    println!("cargo:rustc-link-lib=z");
    match GraphicsRenderingAPI::from_selected_features() {
        GraphicsRenderingAPI::Vulkan => {
            // all libraries below are from glslang-dev despite their names
            println!("cargo:rustc-link-lib=glslang");
            println!("cargo:rustc-link-lib=glslang-default-resource-limits");
            println!("cargo:rustc-link-lib=SPIRV");
            println!("cargo:rustc-link-lib=SPIRV-Tools-opt");
            println!("cargo:rustc-link-lib=SPIRV-Tools");
            println!("cargo:rustc-link-lib=MachineIndependent");
            println!("cargo:rustc-link-lib=GenericCodeGen");
        }
        GraphicsRenderingAPI::OpenGL => {
            println!("cargo:rustc-link-lib=GL");
            println!("cargo:rustc-link-lib=EGL");
        }
        GraphicsRenderingAPI::Metal => {
            // macOS does require dynamic linking against some proprietary system libraries
            // We have not tested this part
        }
    }
    let lib_name = cpp_root
        .file_name()
        .expect("static library base has a file name")
        .to_string_lossy()
        .to_string()
        .replacen("lib", "", 1)
        .replace(".a", "");
    build_bridge(&lib_name, &include_dirs);
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
