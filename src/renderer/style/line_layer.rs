use std::fmt;

use cxx::UniquePtr;

use crate::renderer::{bridge::layers, style::Color};

/// A line layer for rendering line data.
pub struct LineLayer {
    layer: UniquePtr<layers::LineLayer>,
}

impl LineLayer {
    /// Creates a new line layer with the given layer and source IDs.
    pub fn new<S: super::StyleSourceRef>(layer_id: &str, source: &S) -> Self {
        Self { layer: layers::create_line_layer(layer_id, source.source_id()) }
    }

    /// Sets the line color.
    pub fn set_line_color(&mut self, color: Color) {
        layers::setLineColor(&self.layer, &color);
    }

    /// Sets the line opacity.
    pub fn set_line_opacity(&mut self, opacity: f32) {
        layers::setLineOpacity(&self.layer, opacity);
    }

    /// Sets the line width in pixels.
    pub fn set_line_width(&mut self, width: f32) {
        layers::setLineWidth(&self.layer, width);
    }

    pub(crate) fn into_inner(self) -> UniquePtr<layers::LineLayer> {
        self.layer
    }
}

impl fmt::Debug for LineLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LineLayer").field("Pointer", &self.layer.as_ptr()).finish()
    }
}

impl From<LineLayer> for super::StyleLayer {
    fn from(value: LineLayer) -> Self {
        Self::Line(value)
    }
}
