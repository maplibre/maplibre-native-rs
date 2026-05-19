use std::fmt;

use cxx::UniquePtr;

use crate::renderer::{bridge::layers, style::Color};

/// A fill layer for rendering polygon data.
pub struct FillLayer {
    layer: UniquePtr<layers::FillLayer>,
}

impl FillLayer {
    /// Creates a new fill layer with the given layer and source IDs.
    pub fn new<S: super::StyleSourceRef>(layer_id: &str, source: &S) -> Self {
        Self { layer: layers::create_fill_layer(layer_id, source.source_id()) }
    }

    /// Sets the fill color.
    pub fn set_fill_color(&mut self, color: Color) {
        layers::setFillColor(&self.layer, &color);
    }

    /// Sets the fill opacity.
    pub fn set_fill_opacity(&mut self, opacity: f32) {
        layers::setFillOpacity(&self.layer, opacity);
    }

    /// Sets the fill outline color.
    pub fn set_fill_outline_color(&mut self, color: Color) {
        layers::setFillOutlineColor(&self.layer, &color);
    }

    pub(crate) fn into_inner(self) -> UniquePtr<layers::FillLayer> {
        self.layer
    }
}

impl fmt::Debug for FillLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FillLayer").field("Pointer", &self.layer.as_ptr()).finish()
    }
}

impl From<FillLayer> for super::StyleLayer {
    fn from(value: FillLayer) -> Self {
        Self::Fill(value)
    }
}
