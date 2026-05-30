use super::MapObserver;
use crate::bridge::ffi;
use crate::bridge::ffi::BridgeImage;
use crate::renderer::MapDebugOptions;
use crate::RunLoopHandle;
use crate::{CameraUpdate, EdgeInsets, LatLng, LatLngBounds, ScreenCoordinate, Size};
use cxx::UniquePtr;
use image::{ImageBuffer, Rgba};
use std::f64::consts::PI;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::Path;

/// A rendered map image.
///
/// The image is stored as RGBA pixel data using the `image` crate.
/// Use [`as_image`](Image::as_image) to access the underlying `ImageBuffer` for all image operations.
///
/// # Example
///
/// ```no_run
/// # fn foo() {
/// use maplibre_native::{CameraUpdate, Image, ImageRendererBuilder, LatLng};
/// use std::num::NonZeroU32;
///
/// let mut renderer = ImageRendererBuilder::new()
///     .with_size(NonZeroU32::new(512).unwrap(), NonZeroU32::new(512).unwrap())
///     .build_static_renderer();
///
/// renderer.load_style_from_url(&"https://demotiles.maplibre.org/style.json".parse().unwrap());
/// let camera = CameraUpdate::new()
///     .center(LatLng { lat: 0.0, lng: 0.0 })
///     .zoom(0.0);
/// let image: Image = renderer.render_static(&camera).unwrap();
///
/// // Access the underlying ImageBuffer for all operations
/// let img_buffer = image.as_image();
/// println!("Image dimensions: {}x{}", img_buffer.width(), img_buffer.height());
/// img_buffer.save("map.png").unwrap();
/// # }
/// ```
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Image(ImageBuffer<Rgba<u8>, Vec<u8>>);

impl Image {
    /// Create an Image from raw RGBA data
    pub(crate) fn from_raw(bytes: &[u8]) -> Option<Self> {
        // Parse dimensions from first 8 bytes
        if bytes.len() < 8 {
            return None;
        }

        let width = u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let height = u32::from_ne_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let data = bytes[8..].to_vec();
        ImageBuffer::from_vec(width, height, data).map(Image)
    }

    /// Get access to the underlying image buffer.
    /// Use this to perform any image operations using the `image` crate.
    #[must_use]
    pub fn as_image(&self) -> &ImageBuffer<Rgba<u8>, Vec<u8>> {
        &self.0
    }
}

/// Internal state type to render a static map image.
#[derive(Debug)]
pub struct Static;
/// Internal state type to render a map tile.
#[derive(Debug)]
pub struct Tile;

/// Internal state type to render continuously
#[derive(Debug)]
pub struct Continuous;

/// Configuration options for a tile server.
pub struct ImageRenderer<S> {
    pub(crate) instance: UniquePtr<ffi::MapRenderer>,
    pub(crate) _marker: PhantomData<S>,
    // Makes this type !Send and !Sync: the underlying run loop is thread-affine.
    pub(crate) _not_send: PhantomData<*mut ()>,
    pub(crate) style_specified: bool,
}

/// In-flight render request.
///
/// Tick the current thread's run loop via [`RunLoopHandle::tick`] until
/// [`is_ready`](Self::is_ready) returns `true`, then call
/// [`finish`](Self::finish).
///
/// This is intentionally a runtime-agnostic primitive rather than a
/// [`std::future::Future`]. No async runtime drives MapLibre Native's libuv
/// run loop, so the caller advances it explicitly via
/// [`RunLoopHandle::tick`]; a bare `Future` would never make progress under
/// e.g. tokio without an explicit ticker.
#[must_use = "render requests must be finished or waited on to complete the render"]
pub struct RenderRequest<'a, S> {
    instance: UniquePtr<ffi::RenderRequest>,
    _renderer: PhantomData<&'a mut ImageRenderer<S>>,
    // Makes this type !Send and !Sync: the underlying run loop is thread-affine.
    _not_send: PhantomData<*mut ()>,
}

impl<S> Debug for RenderRequest<'_, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderRequest").field("ready", &self.is_ready()).finish_non_exhaustive()
    }
}

impl<S> RenderRequest<'_, S> {
    /// Returns whether the render request has completed.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.instance.isReady()
    }

    /// Returns the rendered image.
    ///
    /// # Panics
    ///
    /// If [`is_ready`](Self::is_ready) returns `false`.
    ///
    /// # Errors
    ///
    /// If the underlying render failed or produced invalid image data.
    pub fn finish(mut self) -> Result<Image, RenderingError> {
        assert!(self.is_ready(), "render request is not ready");

        if self.instance.hasError() {
            return Err(RenderingError::Native(self.instance.errorMessage()));
        }

        let data = self.instance.pin_mut().takeImage();
        let bytes = data.as_bytes();
        Image::from_raw(bytes).ok_or(RenderingError::InvalidImageData)
    }

    /// Blocks on the current thread by ticking the run loop until ready, then
    /// calls [`finish`](Self::finish).
    ///
    /// # Errors
    ///
    /// If the underlying render failed or produced invalid image data.
    pub fn wait(self) -> Result<Image, RenderingError> {
        let run_loop = RunLoopHandle::current();
        while !self.is_ready() {
            run_loop.tick();
        }
        self.finish()
    }
}

impl<S> Debug for ImageRenderer<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageRenderer")
            .field("style_specified", &self.style_specified)
            .finish_non_exhaustive()
    }
}

impl<S> ImageRenderer<S> {
    /// Set the style URL for the map.
    pub fn load_style_from_url(&mut self, url: &url::Url) -> &mut Self {
        self.style_specified = true;
        self.instance.pin_mut().style_load_from_url(url.as_str());
        self
    }

    /// Loads the style from a JSON string.
    pub fn load_style_from_json(&mut self, json: impl AsRef<str>) -> &mut Self {
        self.style_specified = true;
        self.instance.pin_mut().style_load_from_json(json.as_ref());
        self
    }

    /// Load the style from the specified path.
    ///
    /// The style will be loaded from the path, but won't be refreshed automatically if the file changes
    ///
    /// # Errors
    /// Returns an error if the path is not a valid file.
    pub fn load_style_from_path(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<&mut Self, std::io::Error> {
        let path = path.as_ref();
        if !path.is_file() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Path {} is not a file", path.display()),
            ));
        }
        let Some(path) = path.to_str() else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Path {} is not valid UTF-8", path.display()),
            ));
        };
        self.style_specified = true;
        self.instance.pin_mut().style_load_from_url(&format!("file://{path}"));
        Ok(self)
    }

    /// Set debug visualization flags for the map renderer.
    pub fn set_debug_flags(&mut self, flags: MapDebugOptions) -> &mut Self {
        self.instance.pin_mut().setDebugFlags(flags);
        self
    }

    /// Set the renderer output size.
    pub fn set_map_size(&mut self, size: Size) {
        self.instance.pin_mut().setSize(&size);
    }

    /// Get access to the map observer to setup callbacks.
    pub fn map_observer(&mut self) -> MapObserver {
        MapObserver::new(self.instance.pin_mut().observer())
    }

    /// Calculates a camera update that fits geographic bounds.
    #[must_use]
    pub fn camera_for_bounds(
        &mut self,
        bounds: LatLngBounds,
        padding: Option<EdgeInsets>,
        bearing: f64,
        pitch: f64,
    ) -> CameraUpdate {
        let padding = padding.unwrap_or_default();
        let camera =
            self.instance.pin_mut().cameraForLatLngBounds(&bounds, &padding, bearing, pitch);
        CameraUpdate::from_camera_options(camera)
    }

    fn submit_with_camera(
        &mut self,
        camera: &CameraUpdate,
    ) -> Result<RenderRequest<'_, S>, RenderingError> {
        if !self.style_specified {
            return Err(RenderingError::StyleNotSpecified);
        }
        self.instance.pin_mut().jumpTo(&camera.to_camera_options());
        let request = self.instance.pin_mut().submitRender();
        Ok(RenderRequest { instance: request, _renderer: PhantomData, _not_send: PhantomData })
    }
}

impl ImageRenderer<Static> {
    /// Render the map as a static [`Image`] using camera options.
    ///
    /// # Errors
    /// If no style has been loaded.
    pub fn render_static(&mut self, camera: &CameraUpdate) -> Result<Image, RenderingError> {
        self.submit_render_static(camera)?.wait()
    }

    /// Submits a static render request using camera options.
    ///
    /// Use this when driving one or more requests manually with
    /// [`RunLoopHandle::tick`]. Use [`render_static`](Self::render_static) for the
    /// blocking convenience API.
    ///
    /// # Errors
    /// If no style has been loaded.
    pub fn submit_render_static(
        &mut self,
        camera: &CameraUpdate,
    ) -> Result<RenderRequest<'_, Static>, RenderingError> {
        self.submit_with_camera(camera)
    }
}

impl ImageRenderer<Tile> {
    /// Render a top-down tile of the map as a static [`Image`].
    ///
    /// # Errors
    /// If no style has been loaded.
    pub fn render_tile(&mut self, zoom: u8, x: u32, y: u32) -> Result<Image, RenderingError> {
        self.submit_render_tile(zoom, x, y)?.wait()
    }

    /// Submits a tile render request without blocking.
    ///
    /// Use this when driving one or more requests manually with
    /// [`RunLoopHandle::tick`]. Use [`render_tile`](Self::render_tile) for the
    /// blocking convenience API.
    ///
    /// # Errors
    /// If no style has been loaded.
    pub fn submit_render_tile(
        &mut self,
        zoom: u8,
        x: u32,
        y: u32,
    ) -> Result<RenderRequest<'_, Tile>, RenderingError> {
        let center = tile_coords_to_latlng(f64::from(zoom), x, y);
        self.submit_with_camera(
            &CameraUpdate::new().center(center).zoom(f64::from(zoom)).bearing(0.0).pitch(0.0),
        )
    }
}

/// Keeps information about an image including a buffer
/// This is used, so no unneccesary copy of the data must be made
pub struct ImagePtr {
    instance: UniquePtr<BridgeImage>,
}

impl Debug for ImagePtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ImagePtr")
    }
}

impl ImagePtr {
    fn new(image: UniquePtr<BridgeImage>) -> Self {
        Self { instance: image }
    }

    pub fn size(&self) -> Size {
        self.instance.size()
    }

    pub fn buffer(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.instance.get(), self.instance.bufferLength()) }
    }
}

impl ImageRenderer<Continuous> {
    /// Applies a partial camera update immediately.
    ///
    /// See [this graphic](https://en.wikipedia.org/wiki/Degrees_of_freedom_(mechanics)#/media/File:Flight_dynamics_with_text.svg)
    /// as a reminder what bearing, pitch (and yaw) is.
    ///
    /// Important: Without setting the camera initially no image will be generated!
    pub fn update_camera(&mut self, camera: &CameraUpdate) {
        self.instance.pin_mut().jumpTo(&camera.to_camera_options());
    }

    /// Move map by
    pub fn move_by(&mut self, delta: ScreenCoordinate) {
        self.instance.pin_mut().moveBy(&delta);
    }

    /// Scale map (zooming)
    pub fn scale_by(&mut self, scale: f64, pos: ScreenCoordinate) {
        self.instance.pin_mut().scaleBy(scale, &pos);
    }

    /// Trigger render loop once (animations)
    pub fn render_once(&mut self) {
        self.instance.pin_mut().render_once();
    }

    /// Reading rendered image
    pub fn read_still_image(&mut self) -> ImagePtr {
        ImagePtr::new(self.instance.pin_mut().readStillImage())
    }
}

#[allow(clippy::cast_precision_loss)]
fn tile_coords_to_latlng(zoom: f64, x: u32, y: u32) -> LatLng {
    // https://github.com/oldmammuth/slippy_map_tilenames/blob/058678480f4b50b622cda7a48b98647292272346/src/lib.rs#L114
    let zz = 2_f64.powf(zoom);
    let lng = (f64::from(x) + 0.5) / zz * 360_f64 - 180_f64;
    let lat = ((PI * (1_f64 - 2_f64 * (f64::from(y) + 0.5) / zz)).sinh()).atan().to_degrees();
    LatLng { lat, lng }
}

/// Errors that can occur during map rendering operations.
#[derive(thiserror::Error, Debug)]
pub enum RenderingError {
    /// Style must be specified before rendering can occur.
    #[error("Style must be specified before rendering")]
    StyleNotSpecified,
    /// The renderer returned invalid or corrupted image data.
    #[error("Invalid image data received from renderer")]
    InvalidImageData,
    /// MapLibre Native returned a rendering error.
    #[error("Native rendering error: {0}")]
    Native(String),
}

#[cfg(test)]
mod tests {
    use super::tile_coords_to_latlng;

    #[test]
    fn converts_tile_zero_to_geographic_center() {
        let center = tile_coords_to_latlng(0.0, 0, 0);
        assert!(center.lat.abs() < f64::EPSILON);
        assert!(center.lng.abs() < f64::EPSILON);
    }

    #[test]
    fn tile_coordinate_conversion_returns_typed_coordinates() {
        let center = tile_coords_to_latlng(1.0, 1, 1);
        assert!((-90.0..=90.0).contains(&center.lat));
        assert!((-180.0..=180.0).contains(&center.lng));
    }
}
