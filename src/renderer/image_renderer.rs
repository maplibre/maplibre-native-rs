use super::MapObserver;
use crate::renderer::bridge::ffi;
use crate::renderer::bridge::ffi::BridgeImage;
use crate::renderer::MapDebugOptions;
use crate::{Latitude, Longitude, ScreenCoordinate, Size};
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
/// use maplibre_native::{ImageRendererBuilder, Image};
/// use std::num::NonZeroU32;
///
/// let mut renderer = ImageRendererBuilder::new()
///     .with_size(NonZeroU32::new(512).unwrap(), NonZeroU32::new(512).unwrap())
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
    #[cfg(feature = "wgpu")]
    /// Bind the renderer to the WGPU device and queue provided by the host UI. So Textures between maplibre-native and a rust application can be shared
    pub fn set_device_queue(&mut self, device: wgpu::Device, queue: wgpu::Queue) {
        self.instance.pin_mut().setDeviceAndQueue(device.into(), queue.into());
    }

    /// Set the style URL for the map.
    pub fn load_style_from_url(&mut self, url: &url::Url) -> &mut Self {
        self.style_specified = true;
        self.instance.pin_mut().style_load_from_url(url.as_str());
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

        self.instance.pin_mut().setCamera(lat, lon, zoom, bearing, pitch);
        let data = self.instance.pin_mut().render();
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
        self.instance.pin_mut().setCamera(lat.0, lon.0, f64::from(zoom), 0.0, 0.0);

        let data = self.instance.pin_mut().render();
        let bytes = data.as_bytes();
        let image = Image::from_raw(bytes).ok_or(RenderingError::InvalidImageData)?;
        Ok(image)
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
    /// Set the camera position using geographic coordinates.
    ///
    /// See [this grapic](https://en.wikipedia.org/wiki/Degrees_of_freedom_(mechanics)#/media/File:Flight_dynamics_with_text.svg)
    /// as a reminder what bearing, pitch (and yaw) is.
    ///
    /// Important: Without setting the camera initially no image will be generated!
    pub fn set_camera(
        &mut self,
        latitude: Latitude,
        longitude: Longitude,
        zoom: f64,
        bearing: f64,
        pitch: f64,
    ) {
        self.instance.pin_mut().setCamera(latitude.0, longitude.0, zoom, bearing, pitch);
    }

    /// Get access to the map observer to setup callbacks
    pub fn map_observer(&mut self) -> MapObserver {
        MapObserver::new(self.instance.pin_mut().observer())
    }

    /// Move map by
    pub fn move_by(&mut self, delta: ScreenCoordinate) {
        self.instance.pin_mut().moveBy(&delta);
    }

    /// Scale map (zooming)
    pub fn scale_by(&mut self, scale: f64, pos: ScreenCoordinate) {
        self.instance.pin_mut().scaleBy(scale, &pos);
    }

    /// Adjust the map pitch by the given delta in degrees.
    pub fn pitch_by(&mut self, pitch: f64) {
        self.instance.pin_mut().pitchBy(pitch);
    }

    /// Rotate the map using two screen coordinates that represent the gesture delta.
    pub fn rotate_by(&mut self, first: ScreenCoordinate, second: ScreenCoordinate) {
        self.instance.pin_mut().rotateBy(&first, &second);
    }

    /// Trigger render loop once (animations)
    pub fn render_once(&mut self) {
        self.instance.pin_mut().render_once();
    }

    /// Reading rendered image
    pub fn read_still_image(&mut self) -> ImagePtr {
        ImagePtr::new(self.instance.pin_mut().readStillImage())
    }

    #[cfg(feature = "wgpu")]
    /// Take the latest rendered map texture, if one is available.
    pub fn take_texture(&mut self) -> Option<wgpu::Texture> {
        let texture_2d = self.instance.pin_mut().takeTexture();
        if texture_2d.is_null() {
            return None;
        }

        let raw_texture = ffi::getRawTextureHandle(&texture_2d);
        raw_texture.try_into().ok()
    }
}

#[allow(clippy::cast_precision_loss)]
fn coords_to_lat_lon(zoom: f64, x: u32, y: u32) -> (Latitude, Longitude) {
    // https://github.com/oldmammuth/slippy_map_tilenames/blob/058678480f4b50b622cda7a48b98647292272346/src/lib.rs#L114
    let zz = 2_f64.powf(zoom);
    let lng = (f64::from(x) + 0.5) / zz * 360_f64 - 180_f64;
    let lat = ((PI * (1_f64 - 2_f64 * (f64::from(y) + 0.5) / zz)).sinh()).atan().to_degrees();
    (Latitude(lat), Longitude(lng))
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

#[cfg(test)]
mod tests {
    use super::{coords_to_lat_lon, Latitude, Longitude};

    #[test]
    fn converts_tile_zero_to_geographic_center() {
        let (lat, lon) = coords_to_lat_lon(0.0, 0, 0);
        assert!(matches!(lat, Latitude(v) if v.abs() < f64::EPSILON));
        assert!(matches!(lon, Longitude(v) if v.abs() < f64::EPSILON));
    }

    #[test]
    fn tile_coordinate_conversion_returns_typed_coordinates() {
        let (lat, lon) = coords_to_lat_lon(1.0, 1, 1);
        let Latitude(lat) = lat;
        let Longitude(lon) = lon;
        assert!((-90.0..=90.0).contains(&lat));
        assert!((-180.0..=180.0).contains(&lon));
    }
}
