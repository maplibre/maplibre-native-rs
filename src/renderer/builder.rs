//! Image renderer configuration and builder

use crate::renderer::bridge::ffi;
use crate::renderer::{Continuous, ImageRenderer, MapMode, Static, Tile};
use crate::ResourceOptions;
use std::marker::PhantomData;
use std::num::NonZeroU32;

/// Run loop configuration for an [`ImageRenderer`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RunLoopMode {
    /// Use a dedicated run loop for this renderer.
    ///
    /// This is the default. Suitable for worker-thread pools where each worker
    /// owns its renderer. The renderer should be constructed and used on the
    /// same thread.
    #[default]
    Dedicated,
    /// Use MapLibre Native's process-wide default run loop.
    ///
    /// Suitable for single-threaded usage where all renderers are constructed and
    /// used from the same thread. Constructing or using renderers in this mode
    /// from multiple threads can hang or abort because the default loop is
    /// process-shared.
    Shared,
}

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
    /// Image height in pixels
    height: NonZeroU32,
    /// Pixel ratio for high-DPI displays
    pixel_ratio: f32,
    /// Run loop mode for this renderer
    run_loop_mode: RunLoopMode,

    resource_options: Option<ResourceOptions>,
}

impl Default for ImageRendererBuilder {
    #[allow(clippy::missing_panics_doc, reason = "infallible")]
    fn default() -> Self {
        Self {
            width: NonZeroU32::new(512).unwrap(),
            height: NonZeroU32::new(512).unwrap(),
            pixel_ratio: 1.0,
            run_loop_mode: RunLoopMode::default(),
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

    /// Sets the run loop mode for the renderer.
    ///
    /// Default: [`RunLoopMode::Dedicated`]
    #[must_use]
    pub fn with_run_loop_mode(mut self, run_loop_mode: RunLoopMode) -> Self {
        self.run_loop_mode = run_loop_mode;
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
            opts.run_loop_mode == RunLoopMode::Dedicated,
        );

        Self { instance: map, style_specified: false, _marker: PhantomData }
    }
}

#[cfg(test)]
mod tests {
    use super::{ImageRendererBuilder, RunLoopMode};
    use std::{num::NonZeroU32, path::PathBuf, thread};

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(name)
    }

    #[test]
    fn dedicated_run_loop_supports_worker_threads() {
        let handles = (0..2).map(|_| {
            thread::spawn(|| {
                let mut renderer = ImageRendererBuilder::new()
                    .with_size(NonZeroU32::new(128).unwrap(), NonZeroU32::new(128).unwrap())
                    .with_pixel_ratio(1.0)
                    .build_static_renderer();

                renderer
                    .load_style_from_path(fixture_path("test-style.json"))
                    .expect("test style should load");
                let image = renderer
                    .render_static(0.0, 0.0, 0.0, 0.0, 0.0)
                    .expect("dedicated run loop should render");

                assert_eq!(image.as_image().width(), 128);
                assert_eq!(image.as_image().height(), 128);
            })
        });

        for handle in handles {
            handle.join().expect("render thread should not panic");
        }
    }

    #[test]
    fn shared_run_loop_renders_on_single_thread() {
        let mut renderer = ImageRendererBuilder::new()
            .with_size(NonZeroU32::new(128).unwrap(), NonZeroU32::new(128).unwrap())
            .with_pixel_ratio(1.0)
            .with_run_loop_mode(RunLoopMode::Shared)
            .build_static_renderer();

        renderer
            .load_style_from_path(fixture_path("test-style.json"))
            .expect("test style should load");
        let image =
            renderer.render_static(0.0, 0.0, 0.0, 0.0, 0.0).expect("shared run loop should render");

        assert_eq!(image.as_image().width(), 128);
        assert_eq!(image.as_image().height(), 128);
    }
}
