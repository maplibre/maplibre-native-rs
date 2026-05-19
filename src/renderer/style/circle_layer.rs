use std::fmt;

use cxx::UniquePtr;

use crate::renderer::{bridge::layers, style::Color};

/// A circle layer for rendering point data.
pub struct CircleLayer {
    layer: UniquePtr<layers::CircleLayer>,
}

impl CircleLayer {
    /// Creates a new circle layer with the given layer and source IDs.
    pub fn new<S: super::StyleSourceRef>(layer_id: &str, source: &S) -> Self {
        Self { layer: layers::create_circle_layer(layer_id, source.source_id()) }
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

    pub(crate) fn into_inner(self) -> UniquePtr<layers::CircleLayer> {
        self.layer
    }
}

impl fmt::Debug for CircleLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CircleLayer").field("Pointer", &self.layer.as_ptr()).finish()
    }
}

impl From<CircleLayer> for super::StyleLayer {
    fn from(value: CircleLayer) -> Self {
        Self::Circle(value)
    }
}
