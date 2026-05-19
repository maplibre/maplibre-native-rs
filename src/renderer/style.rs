//! Style abstractions for sources, layers, and images.

mod circle_layer;
mod fill_layer;
mod geojson;
mod geojson_source;
mod line_layer;
mod symbol_layer;

use image::DynamicImage;

use crate::renderer::bridge::ffi::Size;
use crate::ImageRenderer;
pub use circle_layer::CircleLayer;
pub use fill_layer::FillLayer;
pub use geojson::{GeoJson, GeoJsonError};
pub use geojson_source::GeoJsonSource;
pub use line_layer::LineLayer;
pub use symbol_layer::SymbolLayer;

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
        assert!((0.0..=1.0).contains(&red));
        assert!((0.0..=1.0).contains(&green));
        assert!((0.0..=1.0).contains(&blue));
        assert!((0.0..=1.0).contains(&alpha));
        Self { r: red * alpha, g: green * alpha, b: blue * alpha, a: alpha }
    }
}

unsafe impl cxx::ExternType for Color {
    type Id = cxx::type_id!("mbgl::Color");
    type Kind = cxx::kind::Trivial;
}

/// Shared interface for style sources that expose a stable source ID.
pub trait StyleSourceRef {
    /// Returns the stable source ID.
    fn source_id(&self) -> &str;
}

/// Shared interface for style images that expose a stable image ID.
pub trait StyleImageRef {
    /// Returns the stable image ID.
    fn image_id(&self) -> &str;
}

/// Stable source ID handle that can be used after a source object is moved.
#[derive(Clone, Debug)]
pub struct SourceId(String);

impl SourceId {
    #[must_use]
    /// Returns the source ID as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl StyleSourceRef for SourceId {
    fn source_id(&self) -> &str {
        self.as_str()
    }
}

/// A stable image ID handle that can be used after an image object is moved.
#[derive(Clone, Debug)]
pub struct ImageId(String);

impl ImageId {
    #[must_use]
    /// Returns the image ID as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl StyleImageRef for ImageId {
    fn image_id(&self) -> &str {
        self.as_str()
    }
}

/// A style source for rendering data layers.
#[non_exhaustive]
#[derive(Debug)]
pub enum StyleSource {
    /// A `GeoJSON` source.
    GeoJson(GeoJsonSource),
}

/// A style layer for rendering.
#[non_exhaustive]
#[derive(Debug)]
pub enum StyleLayer {
    /// A circle layer.
    Circle(CircleLayer),
    /// A fill layer.
    Fill(FillLayer),
    /// A line layer.
    Line(LineLayer),
    /// A symbol layer.
    Symbol(SymbolLayer),
}

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
    pub fn load_url(&mut self, url: &str) {
        self.image_renderer.instance.pin_mut().style_load_from_url(url);
    }

    /// Adds an image to the style with the given ID and options.
    pub fn add_image(
        &mut self,
        id: &str,
        image: &DynamicImage,
        single_distance_field: bool,
    ) -> ImageId {
        use image::EncodableLayout;
        let image = image.to_rgba8();
        self.image_renderer.instance.pin_mut().style_add_image(
            id,
            image.as_bytes(),
            Size::new(super::Width(image.width()), super::Height(image.height())),
            single_distance_field,
        );
        ImageId(id.to_owned())
    }

    /// Removes an image from the style by ID.
    pub fn remove_image(&mut self, id: &str) {
        self.image_renderer.instance.pin_mut().style_remove_image(id);
    }

    /// Add a source to the current map style and return the source id required for the layer
    pub fn add_source<T: Into<StyleSource>>(&mut self, source: T) -> SourceId {
        match source.into() {
            StyleSource::GeoJson(source) => {
                let source_id = SourceId(source.source_id().to_owned());
                self.image_renderer
                    .instance
                    .pin_mut()
                    .style_add_geojson_source(source.into_inner());
                source_id
            }
        }
    }

    /// Add a new layer
    pub fn add_layer<T: Into<StyleLayer>>(&mut self, layer: T) {
        match layer.into() {
            StyleLayer::Circle(layer) => {
                self.image_renderer.instance.pin_mut().style_add_circle_layer(layer.into_inner());
            }
            StyleLayer::Fill(layer) => {
                self.image_renderer.instance.pin_mut().style_add_fill_layer(layer.into_inner());
            }
            StyleLayer::Line(layer) => {
                self.image_renderer.instance.pin_mut().style_add_line_layer(layer.into_inner());
            }
            StyleLayer::Symbol(layer) => {
                self.image_renderer.instance.pin_mut().style_add_symbol_layer(layer.into_inner());
            }
        }
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
