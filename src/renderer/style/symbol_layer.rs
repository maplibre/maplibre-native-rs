use std::fmt;

use cxx::UniquePtr;

use crate::renderer::bridge::layers::{self, SymbolAnchorType};

/// A symbol layer for rendering labels and icons on the map.
pub struct SymbolLayer {
    layer: UniquePtr<layers::SymbolLayer>,
}

impl fmt::Debug for SymbolLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SymbolLayer").field("Pointer", &self.layer.as_ptr()).finish()
    }
}

impl SymbolLayer {
    /// Create a new symbol layer with the given layer and source IDs.
    pub fn new<S: super::StyleSourceRef>(layer_id: &str, source: &S) -> Self {
        Self { layer: layers::create_symbol_layer(layer_id, source.source_id()) }
    }

    /// Set the icon used as marker
    pub fn set_icon_image<T: super::StyleImageRef>(&self, image_id: &T) {
        layers::setIconImage(&self.layer, image_id.image_id());
    }

    /// Set the anchor point of the image
    pub fn set_icon_anchor(&self, anchor: SymbolAnchorType) {
        layers::setIconAnchor(&self.layer, anchor);
    }

    pub(crate) fn into_inner(self) -> UniquePtr<layers::SymbolLayer> {
        self.layer
    }
}

impl From<SymbolLayer> for super::StyleLayer {
    fn from(value: SymbolLayer) -> Self {
        Self::Symbol(value)
    }
}
