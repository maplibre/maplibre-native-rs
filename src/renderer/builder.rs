//! Image renderer configuration and builder

use std::ffi::OsString;
use std::marker::PhantomData;
use std::num::NonZeroU32;
use std::path::PathBuf;

use crate::renderer::bridge::ffi;
use crate::renderer::{ImageRenderer, MapMode, Static, Tile};

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
#[derive(Debug, Clone, PartialEq)]
pub struct ImageRendererBuilder {
    /// Image width in pixels
    width: u32,
    /// Image height in pixelsHash
    height: u32,
    /// Pixel ratio for high-DPI displays
    pixel_ratio: f32,

    /// Cache database file path
    cache_path: Option<PathBuf>,
    /// Assets root directory
    asset_root: Option<PathBuf>,

    /// Base URL for tile server
    base_url: url::Url,
    /// Custom URI scheme alias
    uri_scheme_alias: String,

    /// Source JSON URL template
    source_template: String,
    /// Style JSON URL template
    style_template: String,
    /// Sprite URL template
    sprites_template: String,
    /// Glyph URL template
    glyphs_template: String,
    /// Tile URL template
    tile_template: String,

    /// API key for tile sources
    // TODO: remove?
    api_key: String,
    /// API key parameter name
    api_key_parameter_name: String,
    /// Whether API key is required
    requires_api_key: bool,
}

impl Default for ImageRendererBuilder {
    #[allow(clippy::missing_panics_doc, reason = "infallible")]
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            pixel_ratio: 1.0,

            cache_path: None,
            asset_root: std::env::current_dir().ok(),

            base_url: "https://demotiles.maplibre.org"
                .parse()
                .expect("is a valid url"),
            uri_scheme_alias: "maplibre".to_string(),

            source_template: "/tiles/{domain}.json".to_string(),
            style_template: "{path}.json".to_string(),
            sprites_template: "/{path}/sprite{scale}.{format}".to_string(),
            glyphs_template: "/font/{fontstack}/{start}-{end}.pbf".to_string(),
            tile_template: "/{path}".to_string(),

            api_key_parameter_name: String::new(),
            api_key: String::new(),
            requires_api_key: false,
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

    /// Sets tile server base URL
    ///
    /// Default: <https://demotiles.maplibre.org>
    #[must_use]
    #[allow(clippy::needless_pass_by_value, reason = "false positive")]
    pub fn with_base_url(mut self, base_url: url::Url) -> Self {
        self.base_url = base_url;
        self
    }

    /// Sets custom URI scheme alias
    ///
    /// Default: "maplibre"
    #[must_use]
    #[allow(clippy::needless_pass_by_value, reason = "false positive")]
    pub fn with_uri_scheme_alias(mut self, uri_scheme_alias: impl ToString) -> Self {
        self.uri_scheme_alias = uri_scheme_alias.to_string();
        self
    }

    /// Sets source JSON URL template
    ///
    /// Default: "/tiles/{domain}.json"
    #[must_use]
    #[allow(clippy::needless_pass_by_value, reason = "false positive")]
    pub fn with_source_template(mut self, source_template: impl ToString) -> Self {
        self.source_template = source_template.to_string();
        self
    }
    /// Sets style JSON URL template
    ///
    /// Default: "{path}.json"
    #[must_use]
    #[allow(clippy::needless_pass_by_value, reason = "false positive")]
    pub fn with_style_template(mut self, style_template: impl ToString) -> Self {
        self.style_template = style_template.to_string();
        self
    }

    /// Sets sprite URL template
    ///
    /// Default: "/{path}/sprite{scale}.{format}"
    #[must_use]
    #[allow(clippy::needless_pass_by_value, reason = "false positive")]
    pub fn with_sprites_template(mut self, sprites_template: impl ToString) -> Self {
        self.sprites_template = sprites_template.to_string();
        self
    }

    /// Sets glyph URL template
    ///
    /// Default: "/font/{fontstack}/{start}-{end}.pbf"
    #[must_use]
    #[allow(clippy::needless_pass_by_value, reason = "false positive")]
    pub fn with_glyphs_template(mut self, glyphs_template: impl ToString) -> Self {
        self.glyphs_template = glyphs_template.to_string();
        self
    }

    /// Sets tile URL template
    ///
    /// Default: "/{path}"
    #[must_use]
    #[allow(clippy::needless_pass_by_value, reason = "false positive")]
    pub fn with_tile_template(mut self, tile_template: impl ToString) -> Self {
        self.tile_template = tile_template.to_string();
        self
    }

    /// Sets API key parameter name
    ///
    /// Default: ""
    #[must_use]
    #[allow(clippy::needless_pass_by_value, reason = "false positive")]
    pub fn with_api_key_parameter_name(mut self, api_key_parameter_name: impl ToString) -> Self {
        self.api_key_parameter_name = api_key_parameter_name.to_string();
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

    /// Sets whether API key is required
    ///
    /// Default: `false`
    #[must_use]
    pub fn set_requires_api_key(mut self, requires_api_key: impl Into<bool>) -> Self {
        self.requires_api_key = requires_api_key.into();
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
}

impl<S> ImageRenderer<S> {
    /// Creates a new renderer instance
    fn new(map_mode: MapMode, opts: ImageRendererBuilder) -> Self {
        let map = ffi::MapRenderer_new(
            map_mode,
            opts.width,
            opts.height,
            opts.pixel_ratio,
            // cxx.rs does not support OsString, but going via &[u8] is close enough
            opts.cache_path
                .map_or(OsString::new(), PathBuf::into_os_string)
                .as_encoded_bytes(),
            opts.asset_root
                .map_or(OsString::new(), PathBuf::into_os_string)
                .as_encoded_bytes(),
            &opts.api_key,
            opts.base_url.as_ref(),
            &opts.uri_scheme_alias,
            &opts.api_key_parameter_name,
            &opts.source_template,
            &opts.style_template,
            &opts.sprites_template,
            &opts.glyphs_template,
            &opts.tile_template,
            opts.requires_api_key,
        );

        Self {
            instance: map,
            style_specified: false,
            _marker: PhantomData,
        }
    }
}
