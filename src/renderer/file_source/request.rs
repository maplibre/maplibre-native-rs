use std::{
    ops::RangeInclusive,
    time::{Duration, SystemTime},
};

use crate::bridge::file_source::{RawResourceRequest, ResourceKind};

use super::from_epoch;

/// Resource metadata passed to [`FileSource`](super::FileSource) implementations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ResourceRequest {
    /// Resolved resource URL.
    pub url: String,
    /// Resource kind.
    pub kind: ResourceKind,
    /// Allowed loading methods.
    pub loading_methods: LoadingMethods,
    /// Storage policy requested by MapLibre Native.
    pub storage_policy: StoragePolicy,
    /// Tile metadata, present for tile requests.
    pub tile: Option<TileRequest>,
    /// Requested inclusive byte range, present for partial reads.
    pub data_range: Option<RangeInclusive<u64>>,
    /// Previous `Last-Modified` value from cache, if known.
    pub prior_modified: Option<SystemTime>,
    /// Previous `Expires` value from cache, if known.
    pub prior_expires: Option<SystemTime>,
    /// Previous `ETag`, if known.
    pub prior_etag: Option<String>,
    /// Previous body bytes from cache, if needed for a 304 response.
    pub prior_data: Option<Vec<u8>>,
    /// Minimum interval before refreshing this resource.
    pub minimum_update_interval: Duration,
}

impl ResourceRequest {
    pub(super) fn from_ffi(raw: &RawResourceRequest) -> Self {
        Self {
            url: raw.url.clone(),
            kind: raw.kind,
            loading_methods: LoadingMethods::from_bits(raw.loading_methods),
            storage_policy: if raw.is_volatile {
                StoragePolicy::Volatile
            } else {
                StoragePolicy::Permanent
            },
            tile: raw.has_tile.then(|| TileRequest {
                url_template: raw.tile_url_template.clone(),
                pixel_ratio: raw.tile_pixel_ratio,
                x: raw.tile_x,
                y: raw.tile_y,
                z: raw.tile_z,
            }),
            data_range: raw.has_data_range.then_some(raw.data_range_start..=raw.data_range_end),
            prior_modified: from_epoch(raw.has_prior_modified, raw.prior_modified_epoch_s),
            prior_expires: from_epoch(raw.has_prior_expires, raw.prior_expires_epoch_s),
            prior_etag: raw.has_prior_etag.then(|| raw.prior_etag.clone()),
            prior_data: raw.has_prior_data.then(|| raw.prior_data.clone()),
            minimum_update_interval: Duration::from_millis(
                u64::try_from(raw.minimum_update_interval_ms).unwrap_or(0),
            ),
        }
    }
}

/// MapLibre Native loading-method flags for a resource request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadingMethods(u8);

impl LoadingMethods {
    const CACHE: u8 = 0b01;
    const NETWORK: u8 = 0b10;

    const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    /// Whether cache loading is allowed.
    #[must_use]
    pub const fn has_cache(self) -> bool {
        self.0 & Self::CACHE != 0
    }

    /// Whether network loading is allowed.
    #[must_use]
    pub const fn has_network(self) -> bool {
        self.0 & Self::NETWORK != 0
    }
}

/// MapLibre Native storage policy for a resource request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StoragePolicy {
    /// The response may be stored persistently.
    Permanent,
    /// The response should be treated as volatile.
    Volatile,
}

/// Tile metadata attached to tile resource requests.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TileRequest {
    /// Tile URL template before coordinate substitution.
    pub url_template: String,
    /// Tile pixel ratio.
    pub pixel_ratio: u8,
    /// Tile x coordinate.
    pub x: i32,
    /// Tile y coordinate.
    pub y: i32,
    /// Tile z coordinate.
    pub z: i8,
}
