use crate::renderer::bridge::sources;
use cxx::UniquePtr;
use std::fmt;

/// Latitude coordinate value.
#[derive(Debug, Clone, Copy)]
pub struct Latitude(pub f64);

/// Longitude coordinate value.
#[derive(Debug, Clone, Copy)]
pub struct Longitude(pub f64);

/// Options for configuring a GeoJSON source.
///
/// Contains settings that control how GeoJSON data is processed and rendered,
/// including tiling parameters and clustering configuration.
///
/// Note: Currently, only default options are supported when creating a source.
/// Custom options support will be added in a future release when MapLibre's
/// Immutable<T> FFI support is improved.
#[derive(Debug, Clone)]
pub struct GeoJSONOptions {
    /// Minimum zoom level. Default: 0
    pub minzoom: u8,
    /// Maximum zoom level. Default: 18
    pub maxzoom: u8,
    /// Tile size for GeoJSON-VT discretization. Default: 512
    pub tile_size: u16,
    /// Buffer size in pixels. Default: 128
    pub buffer: u16,
    /// Tolerance for line simplification. Default: 0.375
    pub tolerance: f64,
    /// Whether to compute line metrics. Default: false
    pub line_metrics: bool,
    /// Enable clustering. Default: false
    pub cluster: bool,
    /// Cluster radius in pixels. Default: 50
    pub cluster_radius: u16,
    /// Maximum zoom level for clustering. Default: 17
    pub cluster_max_zoom: u8,
    /// Minimum points for a cluster. Default: 2
    pub cluster_min_points: usize,
    /// Whether to update synchronously. Default: false
    pub synchronous_update: bool,
}

impl Default for GeoJSONOptions {
    fn default() -> Self {
        Self {
            minzoom: 0,
            maxzoom: 18,
            tile_size: 512,
            buffer: 128,
            tolerance: 0.375,
            line_metrics: false,
            cluster: false,
            cluster_radius: 50,
            cluster_max_zoom: 17,
            cluster_min_points: 2,
            synchronous_update: false,
        }
    }
}

/// A `GeoJSON` source for rendering geographic data.
pub struct GeoJsonSource {
    source_id: String,
    source: UniquePtr<sources::GeoJSONSource>,
}

impl GeoJsonSource {
    /// Create a new `GeoJSON` source with default options
    #[must_use]
    pub fn new(id: &str) -> Self {
        Self { source_id: id.to_owned(), source: sources::createWithDefaultOptions(id) }
    }

    /// Create a new `GeoJSON` source with custom options.
    ///
    /// Note: Currently maps to default options. Custom option support
    /// is pending resolution of MapLibre's Immutable<T> FFI constraints.
    #[must_use]
    pub fn with_options(id: &str, _options: &GeoJSONOptions) -> Self {
        // TODO: Pass options to C++ when Immutable<T> FFI support improves
        Self { source_id: id.to_owned(), source: sources::createWithDefaultOptions(id) }
    }

    /// Sets the URL for loading GeoJSON data.
    pub fn set_url(&mut self, url: &str) {
        sources::setURL(&self.source, url);
    }

    /// Sets the point for this source.
    pub fn set_point(&mut self, latitude: Latitude, longitude: Longitude) {
        sources::setPoint(&self.source, latitude.0, longitude.0);
    }

    pub(crate) fn into_inner(self) -> UniquePtr<sources::GeoJSONSource> {
        self.source
    }
}

impl fmt::Debug for GeoJsonSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeoJsonSource")
            .field("source_id", &self.source_id)
            .field("Pointer", &self.source.as_ptr())
            .finish()
    }
}

impl super::StyleSourceRef for GeoJsonSource {
    fn source_id(&self) -> &str {
        &self.source_id
    }
}

impl From<GeoJsonSource> for super::StyleSource {
    fn from(value: GeoJsonSource) -> Self {
        Self::GeoJson(value)
    }
}
