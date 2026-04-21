//! Image renderer configuration and builder

use std::marker::PhantomData;
use std::num::NonZeroU32;

use crate::renderer::bridge::ffi;
use crate::renderer::{Continuous, ImageRenderer, MapMode, Static, Tile};
use crate::ResourceOptions;

/// Builder for configuring [`ImageRenderer`] instances
///
/// # Examples
///
/// ```
/// use maplibre_native::ImageRendererBuilder;
/// use std::num::NonZeroU32;
///
/// let renderer = ImageRendererBuilder::new()
///     .with_size(NonZeroU32::new(1024).unwrap(), NonZeroU32::new(768).unwrap())
///     .with_pixel_ratio(2.0)
///     .build_static_renderer();
/// ```
#[derive(Debug)]
pub struct ImageRendererBuilder {
    /// Image width in pixels
    width: NonZeroU32,
    /// Image height in pixelsHash
    height: NonZeroU32,
    /// Pixel ratio for high-DPI displays
    pixel_ratio: f32,

    resource_options: Option<ResourceOptions>,
}

impl Default for ImageRendererBuilder {
    #[allow(clippy::missing_panics_doc, reason = "infallible")]
    fn default() -> Self {
        Self {
            width: NonZeroU32::new(512).unwrap(),
            height: NonZeroU32::new(512).unwrap(),
            pixel_ratio: 1.0,
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
        self.width = width;
        self.height = height;
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

    /// Set Resource Options
    #[must_use]
    #[allow(clippy::needless_pass_by_value, reason = "false positive")]
    pub fn with_resource_options(mut self, resource_options: ResourceOptions) -> Self {
        self.resource_options = Some(resource_options);
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
        let resource_options = opts.resource_options.unwrap_or_default();
        let map = ffi::MapRenderer_new(
            map_mode,
            opts.width.get(),
            opts.height.get(),
            opts.pixel_ratio,
            resource_options.as_ref(),
        );

        Self { instance: map, style_specified: false, _marker: PhantomData }
    }
}
