use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use walkdir::WalkDir;

// This allows build support to be unit-tested as well as packaged with the crate.
#[path = "build_helper.rs"]
mod build_helper;
use build_helper::parse_deps;

#[path = "build_github.rs"]
mod build_github;
use build_github::GithubRelease;

const MLN_GIT_REPO: &str = "https://github.com/maplibre/maplibre-native.git";
const MLN_REVISION: &str = "12e0922fc4cadcd88808830e697cfb1d5206c8c9";
const MLN_RELEASE_TAG: &str = "core-12e0922fc4cadcd88808830e697cfb1d5206c8c9";

trait CfgBool {
    fn define_bool(&mut self, key: &str, value: bool);
}

impl CfgBool for cmake::Config {
    fn define_bool(&mut self, key: &str, value: bool) {
        self.define(key, if value { "ON" } else { "OFF" });
    }
}

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

/// Helper that returns a new [`cmake::Config`] with common settings.
/// It selects the renderer based on Cargo features: the user must enable exactly one of:
/// "metal", "opengl", or "vulkan". If none are explicitly enabled, on iOS/macOS the default is metal,
/// and on all other platforms the default is vulkan.
fn create_cmake_config(cpp_root: &Path) -> cmake::Config {
    let mut cfg = cmake::Config::new(cpp_root);
    cfg.generator("Ninja");
    cfg.define("CMAKE_C_COMPILER_LAUNCHER", "ccache");
    cfg.define("CMAKE_CXX_COMPILER_LAUNCHER", "ccache");
    cfg.define_bool("MLN_DRAWABLE_RENDERER", true);

    let rendering_backend = GraphicsRenderingAPI::from_selected_features();
    cfg.define_bool(
        "MLN_WITH_OPENGL",
        rendering_backend == GraphicsRenderingAPI::OpenGL,
    );
    cfg.define_bool(
        "MLN_WITH_METAL",
        rendering_backend == GraphicsRenderingAPI::Metal,
    );
    cfg.define_bool(
        "MLN_WITH_VULKAN",
        rendering_backend == GraphicsRenderingAPI::Vulkan,
    );
    cfg.define_bool("MLN_WITH_WERROR", false);

    // The default profile should be release even in a debug mode, otherwise it gets huge
    println!("cargo:rerun-if-env-changed=MLN_BUILD_PROFILE");
    cfg.profile(
        env::var("MLN_BUILD_PROFILE")
            .as_deref()
            .unwrap_or("Release"),
    );

    cfg
}

/// Check that a given dir contains valid maplibre-native code
fn validate_mln(dir: &Path, revision: &str) {
    let dir_disp = dir.display();
    assert!(
        dir.read_dir().expect("Can't read dir").next().is_some(),
        r"
MapLibre-native source directory is empty: {dir_disp}
For local development, make sure to use
    git submodule update --init --recursive
You may also set MLN_FROM_SOURCE to the path of the maplibre-native directory.
"
    );

    let rev = Command::new("git")
        .current_dir(dir)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("Failed to get git revision");
    assert!(rev.status.success(), "Failed to validate git repo");
    let rev = String::from_utf8(rev.stdout).expect("Failed to parse git rev response");
    assert_eq!(
        rev.trim_ascii(),
        revision,
        "Unexpected git revision in {dir_disp}, please update the build.rs with the new value '{rev}'",
    );
}

/// Download static libraries from mln for linking into the vinding.
fn download_static(out_dir: &Path, release_tag: &str) -> (PathBuf, PathBuf) {
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

    // Store release assets in target/mlb_downloads
    let assets_dir =
        PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string()))
            .join("mlb_downloads")
            .join(release_tag);

    // Fetch release assets from GitHub and symlink or copy into out_dir
    let mln_release = GithubRelease::from_repo("maplibre/maplibre-native", release_tag);

    // TODO: values From Cargo.yaml
    let lib_mln_core = mln_release
        .fetch_asset(
            &format!("libmaplibre-native-core-{target}-{graphics_api}.a"),
            &assets_dir,
        )
        .symlink_or_copy_to(out_dir);
    let mln_headers = mln_release
        .fetch_asset("maplibre-native-headers.tar.gz", &assets_dir)
        .symlink_or_copy_to(out_dir);

    (lib_mln_core, mln_headers)
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

fn clone_mln(dir: &Path, repo: &str, revision: &str) {
    let dir_disp = dir.display();
    println!("cargo:warning=Cloning {repo} to {dir_disp} for rev {revision}");

    // git(
    //     dir,
    //     [
    //         "clone",
    //         "--depth=40",
    //         "--recurse-submodules",
    //         "--shallow-submodules",
    //         repo,
    //         dir.to_str().unwrap(),
    //     ],
    // );
    // git(dir, ["reset", "--hard", revision]);

    // Ideally we want this method as it will only fetch the commit of interest.

    // Adapted from https://stackoverflow.com/a/3489576/177275
    // # make a new blank repository in the current directory
    git(dir, ["init"]);
    // # add a remote
    git(dir, ["remote", "add", "origin", repo]);
    // # fetch a commit (or branch or tag) of interest
    // # Note: the full history up to this commit will be retrieved unless
    // #       you limit it with '--depth=...' or '--shallow-since=...'
    git(dir, ["fetch", "origin", revision, "--depth=1"]);
    // # reset this repository's master branch to the commit of interest
    git(dir, ["reset", "--hard", "FETCH_HEAD"]);
    // # fetch submodules
    git(
        dir,
        [
            "submodule",
            "update",
            "--init",
            "--recursive",
            "--depth=1",
            "--jobs=8",
        ],
    );
}

fn git<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(dir: &Path, args: I) {
    fs::create_dir_all(dir).unwrap_or_else(|e| panic!("Failed to create {}: {e}", dir.display()));

    let args = args
        .into_iter()
        .map(|v| v.as_ref().to_os_string())
        .collect::<Vec<_>>();

    let mut cmd = Command::new("git");

    // let git_dir = dir.join(".git");
    // if git_dir.exists() {
    //     eprintln!(
    //         "Running git {args:?} in {} with GIT_DIR={}",
    //         dir.display(),
    //         dir.display()
    //     );
    //     cmd.env("GIT_DIR", dir);
    // } else {
    //     eprintln!("Running git {args:?} in {} without GIT_DIR", dir.display());
    // }

    cmd.current_dir(dir)
        .args(args.clone())
        .status()
        .map_err(|e| e.to_string())
        .and_then(|v| {
            if v.success() {
                Ok(())
            } else {
                Err(v.to_string())
            }
        })
        .unwrap_or_else(|e| panic!("Failed to run git {args:?}: {e}"));
}

/// Clone or download maplibre-native into the `OUT_DIR`
///
/// Returns the path to the maplibre-native directory and an optional path to an include directorys.
fn clone_or_download(root: &Path) -> (PathBuf, Vec<PathBuf>) {
    println!("cargo:rerun-if-env-changed=MLN_CLONE_REPO");
    println!("cargo:rerun-if-env-changed=MLN_FROM_SOURCE");
    println!("cargo:rerun-if-env-changed=MLN_RELEASE_TAG");
    let cpp_root = env::var_os("MLN_FROM_SOURCE").map(PathBuf::from);
    let sub_module = root.join("maplibre-native");
    let mut out_dir: PathBuf = env::var_os("OUT_DIR").expect("OUT_DIR is not set").into();
    out_dir.push("maplibre-native");

    let cpp_root = if let Some(cpp_root) = cpp_root {
        // User specified MLN_FROM_SOURCE - use that if it has CMakeLists.txt
        let cpp_disp = cpp_root.display();
        assert!(
            cpp_root.join("CMakeLists.txt").exists(),
            "Directory {cpp_disp} does not contain maplibre-native"
        );
        println!("cargo:warning=Using maplibre-native at {cpp_disp}");
        cpp_root
    } else if env::var_os("MLN_CLONE_REPO").is_some() {
        // Clone the repo into OUT_DIR - probably because this is part of dependency build
        // Warnings shouldn't show up in the final build output unless there's an error
        clone_mln(&out_dir, MLN_GIT_REPO, MLN_REVISION);
        out_dir
    } else if sub_module.is_dir() {
        // this is a local development that should have the submodule checked out.
        // Do not print any warnings - using the submodule directly
        validate_mln(&sub_module, MLN_REVISION);
        sub_module
    } else {
        // Defaults to downloading the static library
        let release_tag =
            env::var("MLN_RELEASE_TAG").unwrap_or_else(|_| MLN_RELEASE_TAG.to_string());
        let (library_file, headers) = download_static(&out_dir, &release_tag);
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
        return (library_file, include_dirs);
    };

    let check_cmake_list = cpp_root.join("CMakeLists.txt");
    assert!(
        check_cmake_list.exists(),
        "{} does not exist",
        check_cmake_list.display(),
    );

    // TODO: This is a temporary solution. We should get this list from CMake as well.
    let mut include_dirs = vec![
        root.join("include"),
        cpp_root.join("include"),
        cpp_root.join("platform/default/include"),
    ];
    if cpp_root.is_dir() {
        for entry in WalkDir::new(cpp_root.join("vendor")) {
            let entry = entry.expect("Failed reading maplibre-native/vendor directory");
            if entry.file_type().is_dir()
                && !entry.path_is_symlink()
                && entry.file_name() == "include"
            {
                include_dirs.push(entry.path().to_path_buf());
            }
        }
    }

    (cpp_root, include_dirs)
}

/// Build the "mbgl-core-deps" target first so that mbgl-core-deps.txt is generated.
fn add_link_targets(cpp_root: &Path) {
    let deps_build_dir = create_cmake_config(cpp_root)
        .build_target("mbgl-core-deps")
        .build();
    let deps_file = deps_build_dir.join("build").join("mbgl-core-deps.txt");
    let deps_contents = fs::read_to_string(&deps_file)
        .unwrap_or_else(|_| panic!("Failed to read {}", deps_file.display()));

    // Parse the deps file into a list of Cargo instructions.
    for instr in parse_deps(&deps_contents, &deps_build_dir.join("build"), true) {
        println!("{instr}");
    }

    // FIXME:  These should not be manually set like this here
    println!("cargo:rustc-link-lib=icuuc");
    println!("cargo:rustc-link-lib=icui18n");
    println!("cargo:rustc-link-lib=jpeg");
    println!("cargo:rustc-link-lib=png");
    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=curl");
}

/// Build the actual "mbgl-core" static library target.
fn build_static_lib(cpp_root: &Path) {
    let core_build_dir = create_cmake_config(cpp_root)
        .build_target("mbgl-core")
        .build()
        .join("build");
    let static_lib_base = core_build_dir.to_str().unwrap();
    println!("cargo:rustc-link-search=native={static_lib_base}");
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
    let (cpp_root, include_dirs) = clone_or_download(&root);
    let lib_name = if cpp_root.is_dir() {
        add_link_targets(&cpp_root);
        build_static_lib(&cpp_root);
        "mbgl-core".to_string()
    } else {
        println!(
            "cargo:warning=Using precompiled maplibre-native static library from {}",
            cpp_root.display()
        );
        println!(
            "cargo:rustc-link-search=native={}",
            cpp_root.parent().unwrap().display()
        );

        // Add system library search paths for macOS
        if cfg!(target_os = "macos") {
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
                println!("cargo:rustc-link-lib=X11");
            }
            GraphicsRenderingAPI::Metal => {
                // macOS Metal framework dependencies
                if cfg!(target_os = "macos") {
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
        // For prebuilt static library, extract the correct library name
        let lib_filename = cpp_root
            .file_name()
            .expect("static library base has a file name")
            .to_string_lossy();
        if lib_filename.starts_with("lib") && lib_filename.ends_with(".a") {
            lib_filename[3..lib_filename.len() - 2].to_string()
        } else {
            lib_filename.to_string()
        }
    };
    build_bridge(&lib_name, &include_dirs);
}

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    if env::var("DOCS_RS").is_ok() {
        println!("cargo:warning=Skipping build.rs when building for docs.rs");
        println!("cargo::rustc-cfg=docsrs");
        println!("cargo:rustc-check-cfg=cfg(docsrs)");
    } else {
        build_mln();
    }
}
