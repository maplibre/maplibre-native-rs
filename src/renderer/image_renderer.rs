use std::f64::consts::PI;
use std::marker::PhantomData;
use std::path::Path;

use cxx::{CxxString, UniquePtr};

use crate::renderer::bridge::ffi;
use crate::renderer::{ImageRendererOptions, MapDebugOptions, MapMode};

/// A rendered map image.
///
/// The image is stored as a PNG byte array in a buffer allocated by the C++ code.
pub struct Image(UniquePtr<CxxString>);

impl Image {
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// Internal state type to render a static map image.
pub struct Static;
/// Internal state type to render a map tile.
pub struct Tile;

/// Configuration options for a tile server.
pub struct ImageRenderer<S> {
    pub(crate) instance: UniquePtr<ffi::MapRenderer>,
    pub(crate) _marker: PhantomData<S>,
    pub(crate) style_specified: bool,
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
        ffi::MapRenderer_getStyle_loadURL(self.instance.pin_mut(), &format!("file://{path}"));
        Ok(self)
    }

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
        let img = ffi::MapRenderer_render(self.instance.pin_mut());
        Ok(Image(img))
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

        let img = ffi::MapRenderer_render(self.instance.pin_mut());
        Ok(Image(img))
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

#[derive(thiserror::Error, Debug)]
pub enum RenderingError {
    #[error("Style must be specified to render a tile")]
    StyleNotSpecified,
}
