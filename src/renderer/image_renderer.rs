use crate::renderer::bridge::ffi;
use crate::renderer::bridge::ffi::{self, BridgeImage};
use crate::renderer::callbacks::{
    CameraDidChangeCallback, FailingLoadingMapCallback, FinishRenderingFrameCallback, VoidCallback,
};
use crate::renderer::MapDebugOptions;
use crate::{ScreenCoordinate, Size};
use cxx::{CxxString, SharedPtr, UniquePtr};
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
/// use maplibre_native::{ImageRendererBuilder, Image};
///
/// let renderer = ImageRendererBuilder::new()
///     .with_size(512, 512)
///     .build_static_renderer();
///
/// renderer.load_style_from_url(&"https://demotiles.maplibre.org/style.json".parse().unwrap());
/// let image: Image = renderer.render_static(0.0, 0.0, 0.0, 0.0, 0.0).unwrap();
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
    pub(crate) style_specified: bool,
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
        ffi::MapRenderer_getStyle_loadURL(self.instance.pin_mut(), url.as_ref());
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
        ffi::MapRenderer_getStyle_loadURL(self.instance.pin_mut(), &format!("file://{path}"));
        Ok(self)
    }

    /// Set debug visualization flags for the map renderer.
    pub fn set_debug_flags(&mut self, flags: MapDebugOptions) -> &mut Self {
        ffi::MapRenderer_setDebugFlags(self.instance.pin_mut(), flags);
        self
    }
}

impl ImageRenderer<Static> {
    /// Render the map as a static [`Image`] where the camera can be freely controlled.
    ///
    /// # Errors
    /// Returns an error if
    /// - the style has not been specified via either [`load_style_from_path`](Self::load_style_from_path) or [`load_style_from_url`](Self::load_style_from_url).
    pub fn render_static(
        &mut self,
        lat: f64,
        lon: f64,
        zoom: f64,
        bearing: f64,
        pitch: f64,
    ) -> Result<Image, RenderingError> {
        if !self.style_specified {
            return Err(RenderingError::StyleNotSpecified);
        }

        ffi::MapRenderer_setCamera(self.instance.pin_mut(), lat, lon, zoom, bearing, pitch);
        let data = ffi::MapRenderer_render(self.instance.pin_mut());
        let bytes = data.as_bytes();

        let image = Image::from_raw(bytes).ok_or(RenderingError::InvalidImageData)?;
        Ok(image)
    }
}

impl ImageRenderer<Tile> {
    /// Render a top-down tile of the map as a static [`Image`].
    ///
    /// # Errors
    /// Returns an error if
    /// - the style has not been specified via either [`load_style_from_path`](Self::load_style_from_path) or [`load_style_from_url`](Self::load_style_from_url).
    pub fn render_tile(&mut self, zoom: u8, x: u32, y: u32) -> Result<Image, RenderingError> {
        if !self.style_specified {
            return Err(RenderingError::StyleNotSpecified);
        }

        let (lat, lon) = coords_to_lat_lon(f64::from(zoom), x, y);
        ffi::MapRenderer_setCamera(self.instance.pin_mut(), lat, lon, f64::from(zoom), 0.0, 0.0);

        let data = ffi::MapRenderer_render(self.instance.pin_mut());
        let bytes = data.as_bytes();
        let image = Image::from_raw(bytes).ok_or(RenderingError::InvalidImageData)?;
        Ok(image)
    }
}

/// Object to modify the map observer callbacks
pub struct MapObserver {
    instance: SharedPtr<ffi::MapObserver>,
}

impl MapObserver {
    fn new(instance: SharedPtr<ffi::MapObserver>) -> Self {
        Self { instance }
    }

    pub fn set_will_start_loading_map_callback<F: Fn() + 'static>(&mut self, callback: F) {
        // TODO: why is this unsafe and for uniqueptr it is not?
        unsafe {
            self.instance
                .pin_mut_unchecked()
                .setWillStartLoadingMapCallback(Box::new(VoidCallback::new(callback)));
        }
    }

    pub fn set_did_finish_loading_style_callback<F: Fn() + 'static>(&mut self, callback: F) {
        unsafe {
            self.instance
                .pin_mut_unchecked()
                .setFinishLoadingStyleCallback(Box::new(VoidCallback::new(callback)));
        }
    }

    pub fn set_did_become_idle_callback<F: Fn() + 'static>(&mut self, callback: F) {
        unsafe {
            self.instance
                .pin_mut_unchecked()
                .setBecomeIdleCallback(Box::new(VoidCallback::new(callback)));
        }
    }

    pub fn set_did_fail_loading_map_callback<F: Fn(ffi::MapLoadError, &str) + 'static>(
        &mut self,
        callback: F,
    ) {
        unsafe {
            self.instance
                .pin_mut_unchecked()
                .setFailLoadingMapCallback(Box::new(FailingLoadingMapCallback::new(callback)));
        }
    }

    pub fn set_camera_changed_callback<F: Fn(ffi::MapObserverCameraChangeMode) + 'static>(
        &mut self,
        callback: F,
    ) {
        unsafe {
            self.instance
                .pin_mut_unchecked()
                .setCameraDidChangeCallback(Box::new(CameraDidChangeCallback::new(callback)));
        }
    }

    pub fn set_finish_rendering_frame_callback<
        F: Fn(/*needs_repaint:*/ bool, /*placement_changed:*/ bool) + 'static,
    >(
        &mut self,
        callback: F,
    ) {
        unsafe {
            self.instance
                .pin_mut_unchecked()
                .setFinishRenderingFrameCallback(Box::new(FinishRenderingFrameCallback::new(
                    callback,
                )));
        }
    }
}

pub struct ImagePtr {
    instance: UniquePtr<BridgeImage>,
}

impl ImagePtr {
    fn new(image: UniquePtr<BridgeImage>) -> Self {
        Self { instance: image }
    }

    pub fn size(&self) -> Size {
        self.instance.size()
    }

    pub fn buffer<'a>(&'a self) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts(self.instance.get(), self.instance.bufferLength()) }
    }
}

impl ImageRenderer<Continuous> {
    /// Set the camera
    /// Important: Without setting the camera initially no image will be generated!
    pub fn set_camera(&mut self, x: u32, y: u32, zoom: u8, bearing: f64, pitch: f64) {
        let (lat, lon) = coords_to_lat_lon(f64::from(zoom), x, y);
        ffi::MapRenderer_setCamera(
            self.instance.pin_mut(),
            lat,
            lon,
            f64::from(zoom),
            bearing,
            pitch,
        );
    }

    /// Get access to the map observer to setup callbacks
    pub fn map_observer(&mut self) -> MapObserver {
        MapObserver::new(self.instance.pin_mut().observer())
    }

    pub fn move_by(&mut self, delta: ScreenCoordinate) {
        ffi::MapRenderer_moveBy(self.instance.pin_mut(), &delta);
    }

    pub fn scale_by(&mut self, scale: f64, pos: ScreenCoordinate) {
        ffi::MapRenderer_scaleBy(self.instance.pin_mut(), scale, &pos);
    }

    pub fn set_map_size(&mut self, size: Size) {
        ffi::MapRenderer_setSize(self.instance.pin_mut(), &size);
    }

    pub fn render_once(&mut self) {
        ffi::MapRenderer_render_once(self.instance.pin_mut());
    }

    pub fn read_still_image(&mut self) -> ImagePtr {
        ImagePtr::new(ffi::MapRenderer_readStillImage(self.instance.pin_mut()))
    }
}

#[allow(clippy::cast_precision_loss)]
fn coords_to_lat_lon(zoom: f64, x: u32, y: u32) -> (f64, f64) {
    // https://github.com/oldmammuth/slippy_map_tilenames/blob/058678480f4b50b622cda7a48b98647292272346/src/lib.rs#L114
    let zz = 2_f64.powf(zoom);
    let lng = (f64::from(x) + 0.5) / zz * 360_f64 - 180_f64;
    let lat = ((PI * (1_f64 - 2_f64 * (f64::from(y) + 0.5) / zz)).sinh())
        .atan()
        .to_degrees();
    (lat, lng)
}

/// Errors that can occur during map rendering operations.
#[derive(thiserror::Error, Debug)]
pub enum RenderingError {
    /// Style must be specified before rendering can occur.
    #[error("Style must be specified to render a tile")]
    StyleNotSpecified,
    /// The renderer returned invalid or corrupted image data.
    #[error("Invalid image data received from renderer")]
    InvalidImageData,
}

impl Debug for MapObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MapObserver")
    }
}
