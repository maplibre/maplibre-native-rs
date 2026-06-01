use std::fmt;

use cxx::UniquePtr;

use crate::bridge::layers::{self, LineCapType, LineJoinType};
use crate::style::Color;

/// Line cap type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LineCap {
    /// Butt line cap.
    Butt,
    /// Round line cap.
    Round,
    /// Square line cap.
    Square,
}

impl Default for LineCap {
    /// Matches the MapLibre Style Spec default (`butt`).
    fn default() -> Self {
        Self::Butt
    }
}

impl From<LineCap> for LineCapType {
    fn from(value: LineCap) -> Self {
        match value {
            LineCap::Butt => Self::Butt,
            LineCap::Round => Self::Round,
            LineCap::Square => Self::Square,
        }
    }
}

/// Line join type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LineJoin {
    /// Miter line join.
    Miter,
    /// Bevel line join.
    Bevel,
    /// Round line join.
    Round,
}

impl Default for LineJoin {
    /// Matches the MapLibre Style Spec default (`miter`).
    fn default() -> Self {
        Self::Miter
    }
}

impl From<LineJoin> for LineJoinType {
    fn from(value: LineJoin) -> Self {
        match value {
            LineJoin::Miter => Self::Miter,
            LineJoin::Bevel => Self::Bevel,
            LineJoin::Round => Self::Round,
        }
    }
}

/// A line layer for rendering line data.
pub struct LineLayer {
    layer_id: String,
    layer: UniquePtr<layers::LineLayer>,
}

impl LineLayer {
    /// Creates a new line layer with the given layer and source IDs.
    pub fn new(layer_id: &str, source_id: impl AsRef<str>) -> Self {
        Self {
            layer_id: layer_id.to_owned(),
            layer: layers::create_line_layer(layer_id, source_id.as_ref()),
        }
    }

    #[cfg(feature = "json")]
    pub(crate) fn from_ffi_parts(layer_id: String, layer: UniquePtr<layers::LineLayer>) -> Self {
        Self { layer_id, layer }
    }

    pub(crate) fn layer_id(&self) -> &str {
        &self.layer_id
    }

    /// Sets the line color.
    pub fn set_line_color(&mut self, color: Color) {
        layers::setLineColor(&self.layer, &color);
    }

    /// Sets the line cap.
    pub fn set_line_cap(&mut self, cap: LineCap) {
        layers::setLineCap(&self.layer, cap.into());
    }

    /// Sets the line join.
    pub fn set_line_join(&mut self, join: LineJoin) {
        layers::setLineJoin(&self.layer, join.into());
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
        f.debug_struct("LineLayer")
            .field("layer_id", &self.layer_id)
            .field("Pointer", &self.layer.as_ptr())
            .finish()
    }
}
