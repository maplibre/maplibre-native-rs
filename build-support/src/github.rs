use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

pub struct GithubRelease {
    assets: HashMap<String, ReleaseAsset>,
}

/// A type representing a Github release for a given repo.
///
/// Used to fetch release artifacts into a directory for later integration into
/// a build.
impl GithubRelease {
    pub fn from_repo(repo: &str, release_tag: &str) -> Self {
        let api_url = format!("https://api.github.com/repos/{repo}/releases/tags/{release_tag}");
        let json: serde_json::Value = ureq::get(&api_url)
            .call()
            .unwrap()
            .body_mut()
            .read_json()
            .unwrap();

        let assets_vec: Vec<ReleaseAsset> = serde_json::from_value(json["assets"].clone()).unwrap();
        let assets = assets_vec
            .into_iter()
            .map(|asset| (asset.name.clone(), asset))
            .collect();

        Self { assets }
    }

    /// Downloads an asset from a GitHub release if it hasn't been downloaded yet,
    /// and places it into a cache directory.
    ///
    /// Will panic if the asset fails to download or or if the cached asset
    /// does not match the checksum provided in the release metadata.
    pub fn fetch_asset(&self, asset_name: &str, output_dir: &Path) -> AssetPath {
        let asset = self
            .assets
            .get(asset_name)
            .unwrap_or_else(|| panic!("Asset '{}' not found in release", asset_name));
        let asset_checksum = asset.checksum().unwrap();
        assert!(
            !asset_checksum.is_empty(),
            "Asset {asset_name}: empty checksum hash"
        );
        fs::create_dir_all(output_dir).expect(
            "Failed to create destination
   directory",
        );
        let output_path = output_dir.join(&asset.name);
        if output_path.exists() {
            if verify_checksum(&output_path, asset_checksum) {
                return output_path.into();
            };
            println!(
                "cargo:warning=Asset {}: cached file fails checksum, redownloading",
                asset_name
            );
        };
        println!(
                "cargo:warning=Asset {}: downloading asset from github for the first time, this may take a moment...",
                asset_name
            );
        let mut output_file = fs::File::create(&output_path).unwrap();
        std::io::copy(
            &mut ureq::get(&asset.browser_download_url)
                .call()
                .unwrap()
                .into_body()
                .into_reader(),
            &mut output_file,
        )
        .unwrap();
        if !verify_checksum(&output_path, asset_checksum) {
            panic!("Asset {}: checksum mismatch", asset_name)
        };
        output_path.into()
    }
}

/// An individual asset within a release.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ReleaseAsset {
    name: String,
    browser_download_url: String,
    digest: Option<String>,
}

impl ReleaseAsset {
    /// Get the SHA256 checksum for this asset from the release metadata.
    fn checksum(&self) -> Option<&str> {
        self.digest.as_ref()?.strip_prefix("sha256:")
    }
}

/// A lightweight wrapper around PathBuf allowing a release asset to be symlinked
/// into a build directory.
pub struct AssetPath(PathBuf);

impl AssetPath {
    /// Creates a symlink from an asset in a cached download directory into an ephemeral build directory.
    ///
    /// Will fall back to copy if symlinking fails.
    pub fn symlink_or_copy_to(&self, dest_dir: &Path) -> PathBuf {
        fs::create_dir_all(dest_dir).expect(
            "Failed to create destination
   directory",
        );

        let filename = self.0.file_name().expect("Path is a directory");
        let dest_path = dest_dir.join(filename);

        // Remove existing file if it exists
        if dest_path.exists() {
            fs::remove_file(&dest_path).expect(
                "Failed to remove existing
   file",
            );
        }

        // Get absolute path for symlinks to avoid broken relative paths
        let abs_source = self.0.canonicalize().expect(
            "Failed to get
  absolute path",
        );

        // Try to create symlink first, fallback to copy
        // XXX: Would require #[cfg] gate for windows.
        if std::os::unix::fs::symlink(&abs_source, &dest_path).is_err() {
            println!(
                "cargo:warning=symlink to {} failed, copying instead",
                &dest_path.to_string_lossy(),
            );
            fs::copy(&self.0, &dest_path).expect("Failed to copy file");
        };
        dest_path
    }
}

impl From<PathBuf> for AssetPath {
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

impl From<AssetPath> for PathBuf {
    fn from(asset_path: AssetPath) -> Self {
        asset_path.0
    }
}

fn verify_checksum(file_path: &Path, want_hash: &str) -> bool {
    let mut file = fs::File::open(file_path).unwrap();
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).unwrap();
    let got_hash = format!("{:x}", hasher.finalize());
    got_hash == want_hash
}
