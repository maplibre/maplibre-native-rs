use crate::renderer::bridge::resource_options;
use crate::renderer::tile_server_options::TileServerOptions;
use cxx::UniquePtr;
use std::{fmt::Debug, path::PathBuf};

/// Resource Options
pub struct ResourceOptions {
    ptr: UniquePtr<resource_options::ResourceOptions>,
}

impl Debug for ResourceOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Resource Options")
    }
}

impl Default for ResourceOptions {
    /// Create new resource options object
    #[must_use]
    fn default() -> Self {
        Self { ptr: resource_options::new() }
    }
}

impl ResourceOptions {
    /// Set api key
    #[must_use]
    pub fn with_api_key(mut self, key: &str) -> Self {
        resource_options::withApiKey(self.ptr.pin_mut(), key);
        self
    }

    /// Sets cache database file path
    #[must_use]
    pub fn with_cache_path(mut self, path: PathBuf) -> Self {
        // cxx.rs does not support OsString, but going via &[u8] is close enough
        let os_string = path.into_os_string();
        resource_options::withCachePath(self.ptr.pin_mut(), os_string.as_encoded_bytes());
        self
    }

    /// Sets assets root directory
    #[must_use]
    pub fn with_asset_path(mut self, path: PathBuf) -> Self {
        let os_string = path.into_os_string();
        resource_options::withAssetPath(self.ptr.pin_mut(), os_string.as_encoded_bytes());
        self
    }

    /// Set maximum cache size in bytes
    #[must_use]
    pub fn with_maximum_cache_size(mut self, max_cache_size: u64) -> Self {
        resource_options::withMaximumCacheSize(self.ptr.pin_mut(), max_cache_size);
        self
    }

    /// Set tile server options
    #[must_use]
    pub fn with_tile_server_options(mut self, tile_server_options: TileServerOptions) -> Self {
        resource_options::withTileServerOptions(self.ptr.pin_mut(), tile_server_options.into_ptr());
        self
    }

    /// Get internal pointer by consuming this object
    #[must_use]
    pub(crate) fn into_ptr(self) -> UniquePtr<resource_options::ResourceOptions> {
        self.ptr
    }
}
