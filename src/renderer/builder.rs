//! Image renderer configuration and builder

use crate::renderer::bridge::{ffi, resource_options};
use crate::renderer::{Continuous, ImageRenderer, MapMode, Static, Tile};
use crate::ResourceOptions;
use std::ffi::OsString;
use std::marker::PhantomData;
use std::num::NonZeroU32;
use std::path::PathBuf;

/// Builder for configuring [`ImageRenderer`] instances
///
/// # Examples
///
/// ```
/// let renderer = ImageRendererBuilder::new()
///     .with_size(1024, 768)
///     .with_pixel_ratio(2.0)
///     .build_static_renderer();
/// ```
#[derive(Debug)]
pub struct ImageRendererBuilder {
    /// Image width in pixels
    width: u32,
    /// Image height in pixelsHash
    height: u32,
    /// Pixel ratio for high-DPI displays
    pixel_ratio: f32,

    /// The maximum cache size in bytes
    maximum_cache_size: u64,

    /// Cache database file path
    cache_path: Option<PathBuf>,
    /// Assets root directory
    asset_root: Option<PathBuf>,

    /// API key for tile sources
    // TODO: remove?
    api_key: String,

    resource_options: Option<ResourceOptions>,
}

impl Default for ImageRendererBuilder {
    #[allow(clippy::missing_panics_doc, reason = "infallible")]
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            pixel_ratio: 1.0,

            maximum_cache_size: 5000000,
            cache_path: None,
            asset_root: std::env::current_dir().ok(),

            // base_url: "https://demotiles.maplibre.org"
            //     .parse()
            //     .expect("is a valid url"),
            // uri_scheme_alias: "maplibre".to_string(),

            // source_template: "/tiles/{domain}.json".to_string(),
            // style_template: "{path}.json".to_string(),
            // sprites_template: "/{path}/sprite{scale}.{format}".to_string(),
            // glyphs_template: "/font/{fontstack}/{start}-{end}.pbf".to_string(),
            // tile_template: "/{path}".to_string(),

            // api_key_parameter_name: String::new(),
            api_key: String::new(),
            // requires_api_key: false,
            resource_options: None,
        }
    }
}

impl ImageRendererBuilder {
    /// Creates a new builder with default values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets image dimensions
    ///
    /// Default: `512` x `512`
    #[must_use]
    #[allow(clippy::needless_pass_by_value, reason = "false positive")]
    pub fn with_size(mut self, width: NonZeroU32, height: NonZeroU32) -> Self {
        self.width = width.get();
        self.height = height.get();
        self
    }

    /// Sets pixel ratio for high-DPI displays
    ///
    /// Default: `1.0`
    #[must_use]
    #[allow(clippy::needless_pass_by_value, reason = "false positive")]
    pub fn with_pixel_ratio(mut self, pixel_ratio: impl Into<f32>) -> Self {
        self.pixel_ratio = pixel_ratio.into();
        self
    }

    /// Sets the maximum cache size in bytes
    #[must_use]
    #[allow(clippy::needless_pass_by_value, reason = "false positive")]
    pub fn with_maximum_cache_size(mut self, maximum_cache_size: u64) -> Self {
        self.maximum_cache_size = maximum_cache_size;
        self
    }

    /// Sets cache database file path
    ///
    /// Default: no cache
    #[must_use]
    #[allow(clippy::needless_pass_by_value, reason = "false positive")]
    pub fn with_cache_path(mut self, cache_path: impl Into<PathBuf>) -> Self {
        self.cache_path = Some(cache_path.into());
        self
    }

    /// Sets assets root directory
    ///
    /// Default: current working directory
    #[must_use]
    #[allow(clippy::needless_pass_by_value, reason = "false positive")]
    pub fn with_asset_root(mut self, asset_root: impl Into<PathBuf>) -> Self {
        self.asset_root = Some(asset_root.into());
        self
    }

    /// Sets API key
    ///
    /// Default: ""
    #[must_use]
    #[allow(clippy::needless_pass_by_value, reason = "false positive")]
    pub fn with_api_key(mut self, api_key: impl ToString) -> Self {
        self.api_key = api_key.to_string();
        self
    }

    /// Builds a static image renderer
    #[must_use]
    pub fn build_static_renderer(self) -> ImageRenderer<Static> {
        // TODO: Should the width/height be passed in here, or have another `build_static_with_size` method?
        ImageRenderer::new(MapMode::Static, self)
    }

    /// Builds a tile renderer
    #[must_use]
    pub fn build_tile_renderer(self) -> ImageRenderer<Tile> {
        // TODO: Is the width/height used for this mode?
        ImageRenderer::new(MapMode::Tile, self)
    }

    /// Builds a continuous renderer
    /// Using the `MapObserver` it is possible to react on signals from the Map
    #[must_use]
    pub fn build_continuous_renderer(self) -> ImageRenderer<Continuous> {
        ImageRenderer::new(MapMode::Continuous, self)
    }
}

impl<S> ImageRenderer<S> {
    /// Creates a new renderer instance
    fn new(map_mode: MapMode, opts: ImageRendererBuilder) -> Self {
        let resource_options = opts.resource_options.unwrap_or(ResourceOptions::new());
        let map = ffi::MapRenderer_new(
            map_mode,
            opts.width,
            opts.height,
            opts.pixel_ratio,
            resource_options.into_ptr(),
        );

        Self {
            instance: map,
            style_specified: false,
            _marker: PhantomData,
        }
    }
}
