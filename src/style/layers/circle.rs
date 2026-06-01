use std::fmt;

use cxx::UniquePtr;

use crate::bridge::layers;
use crate::style::Color;

/// A circle layer for rendering point data.
pub struct CircleLayer {
    layer_id: String,
    layer: UniquePtr<layers::CircleLayer>,
}

impl CircleLayer {
    /// Creates a new circle layer with the given layer and source IDs.
    pub fn new(layer_id: &str, source_id: impl AsRef<str>) -> Self {
        Self {
            layer_id: layer_id.to_owned(),
            layer: layers::create_circle_layer(layer_id, source_id.as_ref()),
        }
    }

    #[cfg(feature = "json")]
    pub(crate) fn from_ffi_parts(layer_id: String, layer: UniquePtr<layers::CircleLayer>) -> Self {
        Self { layer_id, layer }
    }

    pub(crate) fn layer_id(&self) -> &str {
        &self.layer_id
    }

    /// Sets the circle color.
    pub fn set_circle_color(&mut self, color: Color) {
        layers::setCircleColor(&self.layer, &color);
    }

    /// Sets the circle opacity.
    pub fn set_circle_opacity(&mut self, opacity: f32) {
        layers::setCircleOpacity(&self.layer, opacity);
    }

    /// Sets the circle radius in pixels.
    pub fn set_circle_radius(&mut self, radius: f32) {
        layers::setCircleRadius(&self.layer, radius);
    }

    /// Sets the circle stroke color.
    pub fn set_circle_stroke_color(&mut self, color: Color) {
        layers::setCircleStrokeColor(&self.layer, &color);
    }

    /// Sets the circle stroke opacity.
    pub fn set_circle_stroke_opacity(&mut self, opacity: f32) {
        layers::setCircleStrokeOpacity(&self.layer, opacity);
    }

    /// Sets the circle stroke width in pixels.
    pub fn set_circle_stroke_width(&mut self, width: f32) {
        layers::setCircleStrokeWidth(&self.layer, width);
    }

    pub(crate) fn into_inner(self) -> UniquePtr<layers::CircleLayer> {
        self.layer
    }
}

impl fmt::Debug for CircleLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CircleLayer")
            .field("layer_id", &self.layer_id)
            .field("Pointer", &self.layer.as_ptr())
            .finish()
    }
}
