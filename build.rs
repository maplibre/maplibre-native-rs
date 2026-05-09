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
//! Ubuntu:
//!     - `sudo apt install glslang-dev glslang-tools libicu-dev libpng-dev libjpeg-turbo8-dev libuv1-dev libwebp-dev libglfw3-dev ccache`
//!
//! To build the amalgam library [armerge](https://github.com/tux3/armerge) is required:
//!     - `cargo install armerge`
//!     - `sudo apt install llvm` llvm-objcopy required
use downloader::{Download, Downloader};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

// Used when building locally
const MLN_REPOSITORY_URL: &str = "https://github.com/Murmele/maplibre-native.git";
const MLN_COMMIT: &str = "6d4ff7ce1c4a6b0425233e3528582f6cab167e6f";

// Files of the bridge
const BRIDGE_FILES: &[&str] = &[
    "src/renderer/bridge.rs",
    "src/cpp/bridge.cpp",
    "src/cpp/util.cpp",
    "src/cpp/resource_options.h",
    "src/cpp/resource_options.cpp",
    "src/cpp/tile_server_options.h",
    "src/cpp/tile_server_options.cpp",
    "src/cpp/map_renderer.h",
    "src/cpp/renderer_observer.h",
    "src/cpp/map_observer.h",
    "src/cpp/rust_file_source.h",
    "src/cpp/rust_file_source.cpp",
    "src/cpp/rust_log_observer.h",
    "src/cpp/sources/sources.h",
    "src/cpp/sources/sources.cpp",
    "src/cpp/layers/layers.h",
    "src/cpp/layers/layers.cpp",
    "src/cpp/texture.h",
    "src/cpp/texture.cpp",
];

const BRIDGE_INCLUDE_DIRS: &[&str] = &[/*"include", */ "src/cpp"];

/// Supported graphics rendering APIs.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum GraphicsRenderingAPI {
    /// [Apple's Metal API](https://developer.apple.com/metal/) (macOS/iOS only)
    Metal,
    /// [OpenGL API](https://www.opengl.org/)
    OpenGL,
    /// [Vulkan API](https://www.vulkan.org/)
    Vulkan,
    /// [WGPU API](https://github.com/gfx-rs/wgpu)
    #[expect(clippy::upper_case_acronyms)]
    WGPU,
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
        let with_wgpu = env::var("CARGO_FEATURE_WGPU").is_ok();

        let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");
        let is_macos = target_os == "ios" || target_os == "macos";

        if with_wgpu {
            Self::WGPU
        } else {
            match (with_metal, with_vulkan, with_opengl) {
                (true, false, false) => Self::Metal,
                (false, true, false) => Self::Vulkan,
                (false, false, true) => Self::OpenGL,
                (false, false, false) => {
                    if is_macos {
                        Self::Metal
                    } else {
                        Self::WGPU
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
                        Self::WGPU
                    };
                    println!("cargo::warning=Using only '{default_choice}', but this default selection may change in future releases.");
                    default_choice
                }
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
            Self::WGPU => f.write_str("webgpu-wgpu"),
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

struct CargoTomlInformation {
    mln_release: String,
}

/// Reads `[package.metadata.mln].release` from the crate's `Cargo.toml`.
fn determine_cargo_toml_information() -> CargoTomlInformation {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set"));
    let manifest_path = manifest_dir.join("Cargo.toml");
    println!("cargo:rerun-if-changed={}", manifest_path.display());

    let manifest_str = fs::read_to_string(&manifest_path).unwrap_or_else(|err| {
        panic!("Failed to read manifest at {}: {err}", manifest_path.display())
    });

    let manifest: toml::Value = manifest_str.parse().unwrap_or_else(|err| {
        panic!("Failed to parse manifest as TOML at {}: {err}", manifest_path.display())
    });

    let mln_release = manifest
        .get("package")
        .and_then(|package| package.get("metadata"))
        .and_then(|metadata| metadata.get("mln"))
        .and_then(|mln| mln.get("release"))
        .and_then(toml::Value::as_str)
        .unwrap_or_else(|| {
            panic!(
                "Missing string key [package.metadata.mln].release in {}",
                manifest_path.display()
            )
        })
        .to_owned();

    CargoTomlInformation { mln_release }
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
    archive.unpack(headers_to).expect("Failed to extract headers");
}

/// Get local directory or download maplibre-native into the `OUT_DIR`
///
/// Returns the path to the maplibre-native directory and the include directories.
fn resolve_mln_core() -> (PathBuf, Vec<PathBuf>) {
    let out_dir =
        PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is not set")).join("maplibre-native");
    let mln_release = determine_cargo_toml_information().mln_release;

    println!("cargo:rerun-if-env-changed=MLN_CORE_LIBRARY_PATH");
    println!("cargo:rerun-if-env-changed=MLN_CORE_LIBRARY_HEADERS_PATH");
    let (library_file, headers) =match (env::var_os("MLN_CORE_LIBRARY_PATH"), env::var_os("MLN_CORE_LIBRARY_HEADERS_PATH")) {
      (Some(library_path),Some(headers_path)) => {
        println!("cargo:warning=Local library and headers will be used");
        let _ = headers_path.clone().into_string().inspect(|s| println!("cargo:rerun-if-changed={s}"));
        let _ = library_path.clone().into_string().inspect(|s| println!("cargo:rerun-if-changed={s}"));
        (PathBuf::from(library_path), PathBuf::from(headers_path))
    },
      (Some(_), None) => panic!("MLN_CORE_LIBRARY_HEADERS_PATH is not set. To compile from a local library/headers, both MLN_CORE_LIBRARY_PATH and MLN_CORE_LIBRARY_HEADERS_PATH must be set."),
      (None, Some(_)) => panic!("MLN_CORE_LIBRARY_PATH is not set. To compile from a local library/headers, both MLN_CORE_LIBRARY_PATH and MLN_CORE_LIBRARY_HEADERS_PATH must be set."),
      // Default => to downloading the static library
    (None, None) => download_static(&out_dir, &mln_release),
     };
    assert!(
        library_file.is_file(),
        "The MLN library at {} must be a file. When building locally on Linux it is called libmbgl-core-amalgam.a",
        library_file.display()
    );
    if env::var_os("MLN_CORE_LIBRARY_HEADERS_PATH").is_some() {
        assert!(
            headers.is_file(),
            "The MLN headers at {} must be a gzip (tar.gz) file containing the headers. When building locally checkout <maplibre-native repository>/.github/workflows/core-release.yml commands how to create the header archive",
            headers.display()
        );
    } else {
        assert!(
            headers.is_file(),
            "The MLN headers at {} must be a gzip (tar.gz) file containing the headers.",
            headers.display()
        );
    }

    let extracted_path = out_dir.join("headers");
    extract_headers(&headers, &extracted_path);
    // Returning the downloaded file, bypassing CMakeLists.txt check
    let base = extracted_path.join("vendor").join("maplibre-native-base");
    let deps = base.join("deps");
    let include_dirs = vec![
        base.join("include"),
        deps.join("geometry.hpp").join("include"),
        deps.join("geojson.hpp").join("include"),
        deps.join("variant").join("include"),
        extracted_path.join("include"),
    ];
    (library_file, include_dirs)
}

/// Gather include directories and build the C++ bridge using `cxx_build`.
fn build_bridge(lib_name: &str, include_dirs: &[PathBuf], api: GraphicsRenderingAPI) {
    // println!("cargo:warning=Include_dirs: {:?}", include_dirs);
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let bridge_include_dirs: Vec<PathBuf> =
        BRIDGE_INCLUDE_DIRS.iter().map(|p| root.join(p)).collect();
    let mut build = cxx_build::bridge("src/renderer/bridge.rs");
    build
        .includes(&bridge_include_dirs)
        .includes(include_dirs)
        .flag_if_supported("-std=c++20")
        .warnings(true)
        .warnings_into_errors(true);

    if matches!(api, GraphicsRenderingAPI::WGPU) {
        build.flag_if_supported("-DMLN_WEBGPU_IMPL_FFI=1");
    }

    for f in BRIDGE_FILES {
        println!("cargo:rerun-if-changed={f}");
        #[allow(clippy::case_sensitive_file_extension_comparisons)]
        if f.ends_with(".cpp") {
            build.file(f);
        }
    }

    build.compile("maplibre_rust_map_renderer_bindings");

    // Link mbgl-core after the bridge - or else `cargo test` won't be able to find the symbols.
    println!("cargo:rustc-link-lib=static={lib_name}");
}

struct Info {
    lib_name: String,
    include_dirs: Vec<PathBuf>,
    cpp_root: PathBuf,
}

fn bundle_precompiled() -> Info {
    let (cpp_root, include_dirs) = resolve_mln_core();

    println!(
        "cargo:warning=Using precompiled maplibre-native static library from {}",
        cpp_root.display()
    );
    println!("cargo:rustc-link-search=native={}", cpp_root.parent().unwrap().display());

    // These `cargo:rustc-link-lib` must be done before curl and GL,
    // especially on Linux before 1.90 (1.90 introduced new linker on Linux)
    let lib_name = cpp_root
        .file_name()
        .expect("static library base has a file name")
        .to_string_lossy()
        .to_string()
        .replacen("lib", "", 1)
        .replace(".a", "");

    Info { lib_name, include_dirs, cpp_root }
}

fn clone_repository<P: AsRef<Path>>(
    clone_dir: P,
    folder_name: &str,
    repository_url: &str,
    commit: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(&clone_dir)?;
    let clone_status = Command::new("git")
        .current_dir(clone_dir)
        .args(["clone", "--depth", "1", "--revision", commit, repository_url, folder_name])
        .status()?;
    if !clone_status.success() {
        return Err(format!("Failed to clone maplibre-native repository: {clone_status}").into());
    }
    Ok(())
}

fn submodule_update<P: AsRef<Path>>(repository: P) -> Result<(), Box<dyn std::error::Error>> {
    let submodule_status = Command::new("git")
        .current_dir(repository)
        .args(["submodule", "update", "--init", "--recursive"])
        .status()?;
    if !submodule_status.success() {
        return Err(
            format!("Failed to initialize maplibre-native submodules: {submodule_status}").into()
        );
    }
    Ok(())
}

fn build_local(
    respository_dir: PathBuf,
    name: &str,
    amalgam_lib: bool,
    target_os: &str,
) -> Result<Info, Box<dyn std::error::Error>> {
    const TARGET_NAME: &str = "mbgl-core";
    let maplibre_native_dir = respository_dir.join(name);

    // Some CI cache restores may leave an incomplete directory tree.
    // Require files that prove this is a usable maplibre-native checkout.
    let has_required_checkout_files = maplibre_native_dir.join("CMakeLists.txt").is_file()
        && maplibre_native_dir.join("include").is_dir();

    if maplibre_native_dir.exists() && !has_required_checkout_files {
        println!(
            "cargo:warning=Removing incomplete cached maplibre-native checkout at {}",
            maplibre_native_dir.display()
        );
        fs::remove_dir_all(&maplibre_native_dir)?;
    }

    // Clone Repository
    if !maplibre_native_dir.exists() {
        println!("cargo:warning=Cloning maplibre-native.");
        clone_repository(respository_dir, name, MLN_REPOSITORY_URL, MLN_COMMIT)?;
    }
    println!("cargo:rerun-if-changed={}", maplibre_native_dir.as_os_str().to_str().unwrap());

    // Update submodules
    submodule_update(&maplibre_native_dir)?;

    let mut config = cmake::Config::new(maplibre_native_dir.clone());
    let webgpu_h_include_dir =
        fs::canonicalize(PathBuf::from("binding-generator").join("dep").join("webgpu-headers"))
            .unwrap();
    config.build_target(TARGET_NAME);
    let api = GraphicsRenderingAPI::from_selected_features();

    // maplibre-native's platform/darwin/darwin.cmake calls enable_language(Swift),
    // which the default "Unix Makefiles" generator does not support. Switch to Ninja.
    if target_os == "macos" || target_os == "ios" {
        config.generator("Ninja");
    }

    match api {
        GraphicsRenderingAPI::Metal => {
            config.configure_arg("-DMLN_WITH_METAL=ON");
        }
        GraphicsRenderingAPI::OpenGL => {
            config.configure_arg("-DMLN_WITH_OPENGL=ON");
        }
        GraphicsRenderingAPI::Vulkan => {
            config.configure_arg("-DMLN_WITH_VULKAN=ON");
        }
        GraphicsRenderingAPI::WGPU => {
            config.configure_arg("-DMLN_WITH_WEBGPU=ON");
            config.configure_arg("-DMLN_WEBGPU_IMPL_FFI=ON");
            config.configure_arg("-DMLN_WEBGPU_IMPL_WGPU=ON");
            config.configure_arg(format!(
                "-DMLN_WEBGPU_IMPL_WEBGPU_HEADER_DIR={}",
                webgpu_h_include_dir.as_path().as_os_str().to_str().unwrap()
            ));
        }
    }
    if amalgam_lib {
        config.configure_arg("-DMLN_CREATE_AMALGAMATION:BOOL=ON");
    }
    if cfg!(target_os = "linux") {
        config.configure_arg("-DMLN_WITH_WAYLAND=OFF");
        config.configure_arg("-DMLN_WITH_X11=ON");
    }
    let dest = config.build();
    println!("cargo:rustc-link-search=native={}", dest.join("build").display());
    println!(
        "cargo:rustc-link-search=native={}",
        dest.join("build").join("vendor").join("maplibre-tile-spec").join("cpp").display()
    );
    // println!("cargo:warning=Building maplibre-native done.");

    // maplibre-native include directories
    let mut include_dirs = Vec::new();
    let mut maplibre_native_include_dirs = vec![
        "include",
        "src", // contains offscreen_texture.hpp
        "platform/default/include",
        "vendor/maplibre-native-base/include",
        "vendor/maplibre-native-base/deps/variant/include",
        "vendor/maplibre-native-base/deps/geometry.hpp/include",
        "vendor/maplibre-native-base/deps/geojson.hpp/include",
        "vendor/metal-cpp",
        "vendor/expected-lite/include",
    ];
    if matches!(api, GraphicsRenderingAPI::WGPU) {
        maplibre_native_include_dirs.push("vendor/wgpu-native/ffi");
        maplibre_native_include_dirs.push("vendor/wgpu-native/ffi/webgpu-headers");
        include_dirs.push(dest.join("build").join("webgpu-cpp"));
    }

    // maplibre-rs include dirs
    for i in BRIDGE_INCLUDE_DIRS {
        include_dirs.push(Path::new(i).to_path_buf());
    }

    // Move maplibre-native include dirs into maplibre-rs include dirs
    include_dirs.append(
        &mut maplibre_native_include_dirs
            .into_iter()
            .map(|path| maplibre_native_dir.clone().join(path))
            .collect::<Vec<PathBuf>>(),
    );

    Ok(Info {
        lib_name: format!("{TARGET_NAME}{}", if amalgam_lib { "-amalgam" } else { "" }),
        include_dirs,
        cpp_root: maplibre_native_dir,
    })
}

#[allow(clippy::too_many_lines)]
fn build_mln() {
    println!("cargo:rerun-if-env-changed=MLN_SYSTEM");
    println!("cargo:rerun-if-env-changed=MLN_PRECOMPILE");
    println!("cargo:rerun-if-env-changed=MLN_CORE_LIBRARY_USE_AMALGAM");
    println!("cargo:rerun-if-env-changed=MLN_LOCAL_REPOSITORY");

    let precompiled = !env::var("MLN_PRECOMPILE").unwrap_or("0".to_string()).eq("0");
    let amalgam_lib =
        precompiled || !env::var("MLN_CORE_LIBRARY_USE_AMALGAM").unwrap_or("0".to_string()).eq("0");
    let system_lib = !env::var("MLN_SYSTEM").unwrap_or("0".to_string()).eq("0");
    let local_repository = env::var("MLN_LOCAL_REPOSITORY").unwrap_or_default();

    if !local_repository.is_empty() {
        println!("cargo:warning=Using local repository from: {local_repository}");
        println!("cargo:rerun-if-env-changed={local_repository}");
    }

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

    let info = if precompiled {
        bundle_precompiled()
    } else if system_lib {
        // Using pkg config
        // let mut cfg = pkg_config::Config::new();
        panic!("Not implemented")
    } else {
        const MAPLIBRE_NATIVE_DIR_NAME: &str = "maplibre-native";
        let respository_dir = if local_repository.is_empty() {
            let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
            root.join("target")
        } else {
            assert!(
                local_repository.ends_with(MAPLIBRE_NATIVE_DIR_NAME),
                "The repository must be called: {MAPLIBRE_NATIVE_DIR_NAME}"
            );
            PathBuf::from(local_repository.clone()).parent().unwrap().to_path_buf()
        };

        match build_local(
            respository_dir.clone(),
            MAPLIBRE_NATIVE_DIR_NAME,
            amalgam_lib,
            &target_os,
        ) {
            Err(e) => {
                if respository_dir.join(MAPLIBRE_NATIVE_DIR_NAME).exists()
                    && local_repository.is_empty()
                {
                    // let _ = fs::remove_dir_all(clone_dir.join(MAPLIBRE_NATIVE_DIR_NAME));
                }
                panic!("Failed to build maplibre native: {e}")
            }
            Ok(info) => info,
        }
    };

    let backend = GraphicsRenderingAPI::from_selected_features();
    build_bridge(&info.lib_name, &info.include_dirs, backend);
    let is_apple = target_os == "macos" || target_os == "ios";
    if !amalgam_lib {
        // The dependent libs are not bundled in the core lib, so we have to link manually
        // Required for mlt-cpp. Cpp root link search was already added above
        println!(
            "cargo:rustc-link-search=native={}",
            info.cpp_root
                .parent()
                .unwrap()
                .join("vendor")
                .join("maplibre-tile-spec")
                .join("cpp")
                .display()
        );
        println!("cargo:rustc-link-lib=mbgl-harfbuzz");
        println!("cargo:rustc-link-lib=mbgl-freetype");
        println!("cargo:rustc-link-lib=mbgl-vendor-parsedate");
        println!("cargo:rustc-link-lib=mbgl-vendor-csscolorparser");
        println!("cargo:rustc-link-lib=mlt-cpp"); // provided with maplibre-native
        if is_apple {
            // darwin builds vendored ICU and uses the system sqlite3
            println!("cargo:rustc-link-lib=mbgl-vendor-icu");
            println!("cargo:rustc-link-lib=sqlite3");
        } else {
            println!("cargo:rustc-link-lib=mbgl-vendor-nunicode");
            println!("cargo:rustc-link-lib=mbgl-vendor-sqlite");
            // println!("cargo:rustc-link-lib=utf8proc"); // sudo dnf install utf8proc-devel
            println!("cargo:rustc-link-lib=icuuc"); //sudo dnf install libicu-devel
            println!("cargo:rustc-link-lib=icudata"); //sudo dnf install libicu-devel
            println!("cargo:rustc-link-lib=icui18n"); //sudo dnf install libicu-devel
        }
        // Vulkan translates GLSL to SPIR-V at runtime via glslang; OpenGL/Metal don't.
        if backend == GraphicsRenderingAPI::Vulkan {
            println!("cargo:rustc-link-lib=glslang"); //sudo dnf install libglslang-devel
            println!("cargo:rustc-link-lib=glslang-default-resource-limits"); //sudo dnf install libglslang-devel

            // `SPIRV-Tools-opt` depends on symbols from `SPIRV-Tools`.
            // Keep this order for static linking (notably on Linux/aarch64).
            println!("cargo:rustc-link-lib=SPIRV-Tools-opt"); //sudo dnf install  spirv-tools-devel // Required by glslang spirv-tools-devel
            println!("cargo:rustc-link-lib=SPIRV-Tools"); //sudo dnf install  spirv-tools-devel // Required by glslang spirv-tools-devel
        }
        println!("cargo:rustc-link-lib=png"); // sudo dnf install libpng-devel
        println!("cargo:rustc-link-lib=jpeg"); // sudo dnf install libjpeg-turbo-devel
        println!("cargo:rustc-link-lib=uv"); // sudo dnf install libuv-devel
        println!("cargo:rustc-link-lib=webp"); // sudo dnf install libwebp-devel
    }
    println!("cargo:rustc-link-lib=curl");
    println!("cargo:rustc-link-lib=z");

    if is_apple {
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=CoreGraphics");
    }
    match backend {
        GraphicsRenderingAPI::Vulkan if is_apple => {
            println!("cargo:rustc-link-lib=framework=CoreText");
            println!("cargo:rustc-link-lib=framework=ImageIO");
        }
        GraphicsRenderingAPI::OpenGL => {
            println!("cargo:rustc-link-lib=GL");
            println!("cargo:rustc-link-lib=EGL");
            if cfg!(target_os = "linux") {
                // GLX backend uses X11 symbols such as XInitThreads.
                println!("cargo:rustc-link-lib=X11");
            }
        }
        GraphicsRenderingAPI::Metal => {
            // macOS Metal framework dependencies
            println!("cargo:rustc-link-lib=framework=Metal");
            println!("cargo:rustc-link-lib=framework=MetalKit");
            println!("cargo:rustc-link-lib=framework=QuartzCore");
            println!("cargo:rustc-link-lib=framework=AppKit");
            println!("cargo:rustc-link-lib=framework=CoreLocation");
        }
        GraphicsRenderingAPI::Vulkan | GraphicsRenderingAPI::WGPU => {}
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
