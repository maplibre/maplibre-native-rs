use super::MapObserver;
use crate::bridge::ffi;
use crate::bridge::ffi::BridgeImage;
use crate::renderer::map_observer::MapObserverCallbacks;
use crate::renderer::{MapDebugOptions, MapLoadError};
use crate::RunLoopHandle;
use crate::{CameraUpdate, EdgeInsets, LatLng, LatLngBounds, ScreenCoordinate, Size, StyleRef};
use cxx::UniquePtr;
use image::{ImageBuffer, Rgba};
use std::cell::Cell;
use std::f64::consts::PI;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::Path;
use std::rc::Rc;

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
    pub(crate) observer_callbacks: Rc<MapObserverCallbacks>,
    pub(crate) _marker: PhantomData<S>,
    // Makes this type !Send and !Sync: the underlying run loop is thread-affine.
    pub(crate) _not_send: PhantomData<*mut ()>,
    pub(crate) style_specified: bool,
}

/// In-flight render request.
///
/// Tick the current thread's run loop via [`RunLoopHandle::tick`] until
/// [`is_ready`](Self::is_ready), then call [`finish`](Self::finish), or call
/// [`wait`](Self::wait) to block.
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

#[derive(Clone, Copy)]
enum StyleLoadState {
    Pending,
    Loaded,
    Failed(MapLoadError),
}

/// In-flight style load request.
///
/// Keep the request only when you need to wait for completion or observe the load result.
pub struct StyleLoadRequest<'a, S> {
    state: Rc<Cell<StyleLoadState>>,
    _renderer: PhantomData<&'a mut ImageRenderer<S>>,
    // Makes this type !Send and !Sync: the underlying run loop is thread-affine.
    _not_send: PhantomData<*mut ()>,
}

impl<S> Debug for StyleLoadRequest<'_, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StyleLoadRequest").field("ready", &self.is_ready()).finish_non_exhaustive()
    }
}

impl<S> StyleLoadRequest<'_, S> {
    fn new(state: Rc<Cell<StyleLoadState>>) -> Self {
        Self { state, _renderer: PhantomData, _not_send: PhantomData }
    }

    /// Returns whether the style load has completed (successfully or not).
    #[must_use]
    pub fn is_ready(&self) -> bool {
        !matches!(self.state.get(), StyleLoadState::Pending)
    }

    /// Consumes the request and returns the load result.
    ///
    /// # Panics
    ///
    /// If [`is_ready`](Self::is_ready) returns `false`.
    ///
    /// # Errors
    ///
    /// Returns the [`MapLoadError`] reported by MapLibre Native if the style
    /// failed to load.
    pub fn finish(self) -> Result<(), MapLoadError> {
        match self.state.replace(StyleLoadState::Pending) {
            StyleLoadState::Loaded => Ok(()),
            StyleLoadState::Failed(e) => Err(e),
            StyleLoadState::Pending => panic!("style load request is not ready"),
        }
    }

    /// Blocks on the current thread by ticking the run loop until ready, then
    /// calls [`finish`](Self::finish).
    ///
    /// # Errors
    ///
    /// Returns the [`MapLoadError`] reported by MapLibre Native if the style
    /// failed to load.
    pub fn wait(self) -> Result<(), MapLoadError> {
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
    /// Starts loading the style from a URL.
    ///
    /// Wait for the returned request before rendering if you need the load result
    /// or want to add sources or layers.
    pub fn load_style_from_url(&mut self, url: &url::Url) -> StyleLoadRequest<'_, S> {
        let state = self.begin_style_load();
        self.instance.pin_mut().style_load_from_url(url.as_str());
        StyleLoadRequest::new(state)
    }

    /// Starts loading the style from a JSON string.
    ///
    /// Wait for the returned request before rendering if you need the load result
    /// or want to add sources or layers.
    pub fn load_style_from_json_str(&mut self, json: impl AsRef<str>) -> StyleLoadRequest<'_, S> {
        let state = self.begin_style_load();
        self.instance.pin_mut().style_load_from_json(json.as_ref());
        StyleLoadRequest::new(state)
    }

    /// Starts loading the style from a JSON value.
    ///
    /// Wait for the returned request before rendering if you need the load result
    /// or want to add sources or layers.
    ///
    /// # Errors
    /// Returns an error if the value cannot be serialized to a JSON string.
    #[cfg(feature = "json")]
    pub fn load_style_from_json_value(
        &mut self,
        value: &serde_json::Value,
    ) -> Result<StyleLoadRequest<'_, S>, serde_json::Error> {
        let json = serde_json::to_string(value)?;
        Ok(self.load_style_from_json_str(json))
    }

    /// Starts loading the style from a filesystem path.
    ///
    /// The style will be loaded from the path, but won't be refreshed automatically if the file changes.
    ///
    /// Wait for the returned request before rendering if you need the load result
    /// or want to add sources or layers.
    ///
    /// # Errors
    /// Returns an error if the path is not a valid file. MapLibre Native's own
    /// load errors are surfaced through [`StyleLoadRequest::finish`] /
    /// [`StyleLoadRequest::wait`].
    pub fn load_style_from_path(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<StyleLoadRequest<'_, S>, std::io::Error> {
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
        let state = self.begin_style_load();
        self.instance.pin_mut().style_load_from_url(&format!("file://{path}"));
        Ok(StyleLoadRequest::new(state))
    }

    fn begin_style_load(&mut self) -> Rc<Cell<StyleLoadState>> {
        self.style_specified = true;
        let state = Rc::new(Cell::new(StyleLoadState::Pending));
        self.map_observer().set_style_load_request_callbacks(
            {
                let weak = Rc::downgrade(&state);
                move || {
                    if let Some(s) = weak.upgrade() {
                        s.set(StyleLoadState::Loaded);
                    }
                }
            },
            {
                let weak = Rc::downgrade(&state);
                // TODO: the `what` string carries the underlying parse/validation/
                // network message. `StyleLoadState`/`MapLoadError` only keep the
                // error kind, so that detail is dropped. Surfacing it would require
                // a richer error type (e.g. `{ kind, message }`).
                move |error, _what| {
                    if let Some(s) = weak.upgrade() {
                        s.set(StyleLoadState::Failed(error));
                    }
                }
            },
        );
        state
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
        MapObserver::new(self.instance.pin_mut().observer(), Rc::clone(&self.observer_callbacks))
    }

    /// Gets a mutable reference to the current map style.
    pub fn style(&mut self) -> StyleRef<'_, S> {
        StyleRef::new(self)
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
