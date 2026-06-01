use std::fmt;

use cxx::UniquePtr;

use crate::bridge::layers::{self, SymbolAnchorType};

/// Symbol icon anchor point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SymbolAnchor {
    /// The center of the icon.
    Center,
    /// The left side of the icon.
    Left,
    /// The right side of the icon.
    Right,
    /// The top side of the icon.
    Top,
    /// The bottom side of the icon.
    Bottom,
    /// The top-left corner of the icon.
    TopLeft,
    /// The top-right corner of the icon.
    TopRight,
    /// The bottom-left corner of the icon.
    BottomLeft,
    /// The bottom-right corner of the icon.
    BottomRight,
}

impl From<SymbolAnchor> for SymbolAnchorType {
    fn from(value: SymbolAnchor) -> Self {
        match value {
            SymbolAnchor::Center => Self::Center,
            SymbolAnchor::Left => Self::Left,
            SymbolAnchor::Right => Self::Right,
            SymbolAnchor::Top => Self::Top,
            SymbolAnchor::Bottom => Self::Bottom,
            SymbolAnchor::TopLeft => Self::TopLeft,
            SymbolAnchor::TopRight => Self::TopRight,
            SymbolAnchor::BottomLeft => Self::BottomLeft,
            SymbolAnchor::BottomRight => Self::BottomRight,
        }
    }
}

/// A symbol layer for rendering labels and icons on the map.
pub struct SymbolLayer {
    layer_id: String,
    layer: UniquePtr<layers::SymbolLayer>,
}

impl fmt::Debug for SymbolLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SymbolLayer")
            .field("layer_id", &self.layer_id)
            .field("Pointer", &self.layer.as_ptr())
            .finish()
    }
}

impl SymbolLayer {
    /// Create a new symbol layer with the given layer and source IDs.
    pub fn new(layer_id: &str, source_id: impl AsRef<str>) -> Self {
        Self {
            layer_id: layer_id.to_owned(),
            layer: layers::create_symbol_layer(layer_id, source_id.as_ref()),
        }
    }

    #[cfg(feature = "json")]
    pub(crate) fn from_ffi_parts(layer_id: String, layer: UniquePtr<layers::SymbolLayer>) -> Self {
        Self { layer_id, layer }
    }

    pub(crate) fn layer_id(&self) -> &str {
        &self.layer_id
    }

    /// Set the icon used as marker
    pub fn set_icon_image(&mut self, image_id: impl AsRef<str>) {
        layers::setIconImage(&self.layer, image_id.as_ref());
    }

    /// Set the anchor point of the image
    pub fn set_icon_anchor(&mut self, anchor: SymbolAnchor) {
        layers::setIconAnchor(&self.layer, anchor.into());
    }

    pub(crate) fn into_inner(self) -> UniquePtr<layers::SymbolLayer> {
        self.layer
    }
}
