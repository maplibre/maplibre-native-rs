use std::fmt;

use cxx::UniquePtr;

use crate::bridge::layers;
use crate::style::Color;

/// A fill layer for rendering polygon data.
pub struct FillLayer {
    layer_id: String,
    layer: UniquePtr<layers::FillLayer>,
}

impl FillLayer {
    /// Creates a new fill layer with the given layer and source IDs.
    pub fn new(layer_id: &str, source_id: impl AsRef<str>) -> Self {
        Self {
            layer_id: layer_id.to_owned(),
            layer: layers::create_fill_layer(layer_id, source_id.as_ref()),
        }
    }

    #[cfg(feature = "json")]
    pub(crate) fn from_ffi_parts(layer_id: String, layer: UniquePtr<layers::FillLayer>) -> Self {
        Self { layer_id, layer }
    }

    pub(crate) fn layer_id(&self) -> &str {
        &self.layer_id
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
        f.debug_struct("FillLayer")
            .field("layer_id", &self.layer_id)
            .field("Pointer", &self.layer.as_ptr())
            .finish()
    }
}
