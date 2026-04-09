use crate::renderer::bridge::ffi::{self, Size};
use crate::ImageRenderer;
use image::DynamicImage;

mod geojson_source;
pub use geojson_source::GeoJsonSource;
pub use geojson_source::Latitude;
pub use geojson_source::Longitude;

/// Shared interface for style sources that expose a stable source ID.
pub trait StyleSourceRef {
    fn source_id(&self) -> &str;
}

/// Stable source ID handle that can be used after a source object is moved.
#[derive(Clone, Debug)]
pub struct SourceId(String);

impl SourceId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl StyleSourceRef for SourceId {
    fn source_id(&self) -> &str {
        self.as_str()
    }
}

pub enum StyleSource {
    GeoJson(GeoJsonSource),
}

pub enum StyleLayer {
    Symbol(SymbolLayer),
}

mod symbol_layer;
pub use symbol_layer::SymbolLayer;

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
        self.image_renderer
            .instance
            .pin_mut()
            .style_load_from_url(url);
    }

    pub fn add_image(&mut self, id: &str, image: &DynamicImage, single_distance_field: bool) {
        use image::EncodableLayout;
        let image = image.to_rgba8();
        self.image_renderer.instance.pin_mut().style_add_image(
            id,
            image.as_bytes(),
            Size::new(super::Width(image.width()), super::Height(image.height())),
            single_distance_field,
        );
    }

    pub fn remove_image(&mut self, id: &str) {
        self.image_renderer
            .instance
            .pin_mut()
            .style_remove_image(id);
    }

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

    pub fn add_layer<T: Into<StyleLayer>>(&mut self, layer: T) {
        match layer.into() {
            StyleLayer::Symbol(layer) => self
                .image_renderer
                .instance
                .pin_mut()
                .style_add_symbol_layer(layer.into_inner()),
        }
    }
}
