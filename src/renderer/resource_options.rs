use crate::renderer::bridge::resource_options::{self, *};
use cxx::UniquePtr;
use std::ffi::OsString;
use std::{fmt::Debug, path::PathBuf};

pub struct ResourceOptions {
    ptr: UniquePtr<resource_options::ResourceOptions>,
}

impl Debug for ResourceOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Resource Options")
    }
}

impl ResourceOptions {
    pub fn new() -> Self {
        Self { ptr: new() }
    }

    pub fn with_api_key(mut self, key: &str) -> Self {
        withApiKey(self.ptr.pin_mut(), key);
        self
    }

    pub fn with_cache_path(mut self, path: PathBuf) -> Self {
        // cxx.rs does not support OsString, but going via &[u8] is close enough
        let os_string = path.into_os_string();
        withCachePath(self.ptr.pin_mut(), os_string.as_encoded_bytes());
        self
    }

    pub fn with_asset_path(mut self, path: PathBuf) -> Self {
        let os_string = path.into_os_string();
        withAssetPath(self.ptr.pin_mut(), os_string.as_encoded_bytes());
        self
    }

    pub fn with_maximum_cache_size(mut self, max_cache_size: u64) -> Self {
        withMaximumCacheSize(self.ptr.pin_mut(), max_cache_size);
        self
    }

    pub(crate) fn into_ptr(self) -> UniquePtr<resource_options::ResourceOptions> {
        self.ptr
    }
}
