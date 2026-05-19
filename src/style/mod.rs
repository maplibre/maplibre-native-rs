//! Style abstractions for sources, layers, and images.

mod circle_layer;
mod fill_layer;
mod geojson;
mod geojson_source;
mod line_layer;
mod symbol_layer;

use image::DynamicImage;

use crate::bridge::{ffi, layers, sources};
use crate::ImageRenderer;
pub use circle_layer::CircleLayer;
pub use fill_layer::FillLayer;
pub use geojson::{GeoJson, GeoJsonError};
pub use geojson_source::GeoJsonSource;
pub use line_layer::{LineCap, LineJoin, LineLayer};
pub use symbol_layer::{SymbolAnchor, SymbolLayer};

/// A color constructed from straight RGBA channels in the `0.0..=1.0` range.
///
/// MapLibre Native stores colors as premultiplied RGBA. Constructors on this
/// type accept straight RGBA and store the premultiplied representation expected
/// by MapLibre Native.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    /// Creates an opaque RGB color from channel values in the `0.0..=1.0` range.
    #[must_use]
    pub fn rgb(red: f32, green: f32, blue: f32) -> Self {
        Self::rgba(red, green, blue, 1.0)
    }

    /// Creates an RGBA color from channel values in the `0.0..=1.0` range.
    ///
    /// # Panics
    ///
    /// Panics if any channel is outside the `0.0..=1.0` range.
    #[must_use]
    pub fn rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        assert!(
            (0.0..=1.0).contains(&red)
                && (0.0..=1.0).contains(&green)
                && (0.0..=1.0).contains(&blue)
                && (0.0..=1.0).contains(&alpha),
            "color channels must be in the 0.0..=1.0 range; got rgba({red}, {green}, {blue}, {alpha})",
        );
        Self { r: red * alpha, g: green * alpha, b: blue * alpha, a: alpha }
    }
}

unsafe impl cxx::ExternType for Color {
    type Id = cxx::type_id!("mbgl::Color");
    type Kind = cxx::kind::Trivial;
}

/// Error returned when MapLibre Native rejects a style mutation.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StyleError {
    /// MapLibre Native rejected or failed to apply the style mutation.
    #[error("style error: {0}")]
    Native(String),
}

impl From<cxx::Exception> for StyleError {
    fn from(value: cxx::Exception) -> Self {
        Self::Native(value.to_string())
    }
}

/// Stable source ID handle that can be used after a source object is moved.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SourceId(String);

impl SourceId {
    #[must_use]
    /// Returns the source ID as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for SourceId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Stable layer ID handle that can be used after a layer object is moved.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LayerId(String);

impl LayerId {
    #[must_use]
    /// Returns the layer ID as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for LayerId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// A stable image ID handle that can be used after an image object is moved.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImageId(String);

impl ImageId {
    #[must_use]
    /// Returns the image ID as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ImageId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

mod sealed {
    use crate::bridge::ffi;

    pub trait IntoLayer {
        fn layer_id(&self) -> &str;
        fn into_layer(self) -> cxx::UniquePtr<ffi::CxxLayer>;
    }

    pub trait IntoSource {
        fn source_id(&self) -> &str;
        fn into_source(self) -> cxx::UniquePtr<ffi::CxxSource>;
    }
}

/// A style source type that can be added to a [`Style`].
///
/// This trait is sealed; only source types provided by this crate can implement
/// it.
pub trait Source: sealed::IntoSource {}

impl sealed::IntoSource for GeoJsonSource {
    fn source_id(&self) -> &str {
        self.source_id()
    }

    fn into_source(self) -> cxx::UniquePtr<ffi::CxxSource> {
        sources::geojson_into_source(self.into_inner())
    }
}

impl Source for GeoJsonSource {}

/// A style layer type that can be added to a [`Style`].
///
/// This trait is sealed; only layer types provided by this crate can implement
/// it.
pub trait Layer: sealed::IntoLayer {}

impl sealed::IntoLayer for CircleLayer {
    fn layer_id(&self) -> &str {
        self.layer_id()
    }

    fn into_layer(self) -> cxx::UniquePtr<ffi::CxxLayer> {
        layers::circle_into_layer(self.into_inner())
    }
}

impl Layer for CircleLayer {}

impl sealed::IntoLayer for FillLayer {
    fn layer_id(&self) -> &str {
        self.layer_id()
    }

    fn into_layer(self) -> cxx::UniquePtr<ffi::CxxLayer> {
        layers::fill_into_layer(self.into_inner())
    }
}

impl Layer for FillLayer {}

impl sealed::IntoLayer for LineLayer {
    fn layer_id(&self) -> &str {
        self.layer_id()
    }

    fn into_layer(self) -> cxx::UniquePtr<ffi::CxxLayer> {
        layers::line_into_layer(self.into_inner())
    }
}

impl Layer for LineLayer {}

impl sealed::IntoLayer for SymbolLayer {
    fn layer_id(&self) -> &str {
        self.layer_id()
    }

    fn into_layer(self) -> cxx::UniquePtr<ffi::CxxLayer> {
        layers::symbol_into_layer(self.into_inner())
    }
}

impl Layer for SymbolLayer {}

/// The style of the map
#[derive(Debug)]
pub struct Style<'a, S> {
    image_renderer: &'a mut ImageRenderer<S>,
}

impl<'a, S> Style<'a, S> {
    /// get a style reference from the current map
    pub fn get_ref(image_renderer: &'a mut ImageRenderer<S>) -> Self {
        Self { image_renderer }
    }

    /// Apply the style from the url to the map
    pub fn load_url(&mut self, url: impl AsRef<str>) {
        self.image_renderer.instance.pin_mut().style_load_from_url(url.as_ref());
    }

    /// Adds an image to the style with the given ID and options.
    ///
    /// Pass `true` for `sdf` to register the image as a signed distance field
    /// icon; pass `false` for a regular bitmap icon.
    ///
    /// # Errors
    ///
    /// Returns an error if MapLibre Native rejects the image.
    pub fn add_image(
        &mut self,
        id: impl AsRef<str>,
        image: &DynamicImage,
        sdf: bool,
    ) -> Result<ImageId, StyleError> {
        use image::EncodableLayout;
        let id = id.as_ref();
        let image = image.to_rgba8();
        self.image_renderer.instance.pin_mut().style_add_image(
            id,
            image.as_bytes(),
            ffi::Size::new(super::Width(image.width()), super::Height(image.height())),
            sdf,
        )?;
        Ok(ImageId(id.to_owned()))
    }

    /// Removes an image from the style by ID.
    ///
    /// No-op if `id` does not match an existing image.
    pub fn remove_image(&mut self, id: impl AsRef<str>) {
        self.image_renderer.instance.pin_mut().style_remove_image(id.as_ref());
    }

    /// Add a source to the current map style and return the source id required for the layer.
    ///
    /// # Errors
    ///
    /// Returns an error if MapLibre Native rejects the source.
    pub fn add_source<T: Source>(&mut self, source: T) -> Result<SourceId, StyleError> {
        let source_id = SourceId(source.source_id().to_owned());
        self.image_renderer.instance.pin_mut().style_add_source(source.into_source())?;
        Ok(source_id)
    }

    /// Add a new layer and return its stable layer ID.
    ///
    /// # Errors
    ///
    /// Returns an error if MapLibre Native rejects the layer.
    pub fn add_layer<T: Layer>(&mut self, layer: T) -> Result<LayerId, StyleError> {
        let layer_id = LayerId(layer.layer_id().to_owned());
        self.add_layer_inner(layer.into_layer(), None)?;
        Ok(layer_id)
    }

    /// Add a new layer before an existing layer.
    ///
    /// `before_layer` is the ID of an existing layer; the new layer is inserted
    /// directly below it. If `before_layer` does not match any existing layer,
    /// the new layer is appended to the end of the style.
    ///
    /// # Errors
    ///
    /// Returns an error if MapLibre Native rejects the layer.
    pub fn add_layer_before<T: Layer>(
        &mut self,
        layer: T,
        before_layer: impl AsRef<str>,
    ) -> Result<LayerId, StyleError> {
        let layer_id = LayerId(layer.layer_id().to_owned());
        self.add_layer_inner(layer.into_layer(), Some(before_layer.as_ref()))?;
        Ok(layer_id)
    }

    fn add_layer_inner(
        &mut self,
        layer: cxx::UniquePtr<ffi::CxxLayer>,
        before_id: Option<&str>,
    ) -> Result<(), StyleError> {
        // The C++ bridge encodes "append" as an empty string; that detail is
        // contained here so the public API can use separate append/before methods.
        let before_id = before_id.unwrap_or_default();
        self.image_renderer.instance.pin_mut().style_add_layer(layer, before_id)?;
        Ok(())
    }

    /// Removes a layer from the current map style by ID.
    ///
    /// No-op if `layer_id` does not match an existing layer.
    pub fn remove_layer(&mut self, layer_id: impl AsRef<str>) {
        self.image_renderer.instance.pin_mut().style_remove_layer(layer_id.as_ref());
    }

    /// Removes a source from the current map style by ID.
    ///
    /// No-op if `source_id` does not match an existing source.
    pub fn remove_source(&mut self, source_id: impl AsRef<str>) {
        self.image_renderer.instance.pin_mut().style_remove_source(source_id.as_ref());
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn rgba_stores_premultiplied_channels() {
        assert_eq!(Color::rgba(1.0, 0.0, 0.0, 0.5), Color { r: 0.5, g: 0.0, b: 0.0, a: 0.5 });
    }

    #[test]
    fn rgb_stores_opaque_channels() {
        assert_eq!(Color::rgb(1.0, 0.0, 0.0), Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 });
    }
}
