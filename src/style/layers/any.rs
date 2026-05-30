use std::fmt;

use cxx::UniquePtr;

#[cfg(feature = "json")]
use crate::bridge::style_value;
use crate::bridge::{ffi, layers};

#[cfg(feature = "json")]
use crate::style::{value::build_style_value, StyleError};
use crate::style::{CircleLayer, FillLayer, LineLayer, SymbolLayer};

/// A style layer of a type that does not (yet) have a typed Rust wrapper.
pub struct OpaqueLayer {
    layer_id: String,
    layer_type: String,
    layer: UniquePtr<ffi::CxxLayer>,
}

impl OpaqueLayer {
    /// Returns the layer's ID.
    #[must_use]
    pub fn layer_id(&self) -> &str {
        &self.layer_id
    }

    /// Returns the layer's MapLibre style-spec type string (e.g. `"raster"`,
    /// `"background"`, `"heatmap"`).
    #[must_use]
    pub fn type_str(&self) -> &str {
        &self.layer_type
    }

    pub(crate) fn into_inner(self) -> UniquePtr<ffi::CxxLayer> {
        self.layer
    }
}

impl fmt::Debug for OpaqueLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpaqueLayer")
            .field("layer_id", &self.layer_id)
            .field("type", &self.layer_type)
            .finish_non_exhaustive()
    }
}

/// A style layer of any type, parsed from a style-spec layer object.
///
/// Variants for layer types that this crate has typed wrappers for hold those
/// typed wrappers directly, so existing setters keep working. Other layer types
/// are held by [`OpaqueLayer`].
#[derive(Debug)]
#[non_exhaustive]
pub enum AnyLayer {
    /// A typed [`CircleLayer`].
    Circle(CircleLayer),
    /// A typed [`FillLayer`].
    Fill(FillLayer),
    /// A typed [`LineLayer`].
    Line(LineLayer),
    /// A typed [`SymbolLayer`].
    Symbol(SymbolLayer),
    /// A layer of a type that does not have a typed wrapper in this crate yet.
    Opaque(OpaqueLayer),
}

impl AnyLayer {
    /// Parses a single style-spec layer object from a JSON string.
    ///
    /// # Errors
    ///
    /// Returns [`StyleError::Json`] if the input is not valid JSON, or
    /// [`StyleError::Native`] if MapLibre Native rejects the layer.
    #[cfg(feature = "json")]
    pub fn from_json_str(json: &str) -> Result<Self, StyleError> {
        let value: serde_json::Value = serde_json::from_str(json)?;
        Self::from_json_value(&value)
    }

    /// Parses a style-spec layer object from a [`serde_json::Value`].
    ///
    /// # Errors
    ///
    /// Returns [`StyleError::JsonNumber`] if a JSON number cannot be converted,
    /// or [`StyleError::Native`] if MapLibre Native rejects the value.
    #[cfg(feature = "json")]
    pub fn from_json_value(value: &serde_json::Value) -> Result<Self, StyleError> {
        let style_value = build_style_value(value)?;
        let mut error_message = String::new();
        let layer = style_value::layer_from_value(&style_value, &mut error_message);
        if layer.is_null() {
            return Err(StyleError::Native(error_message));
        }
        let layer_id = layers::layer_id(&layer);
        let layer_type = layers::layer_type(&layer);

        Ok(match layer_type.as_str() {
            "circle" => {
                Self::Circle(CircleLayer::from_ffi_parts(layer_id, layers::try_into_circle(layer)))
            }
            "fill" => Self::Fill(FillLayer::from_ffi_parts(layer_id, layers::try_into_fill(layer))),
            "line" => Self::Line(LineLayer::from_ffi_parts(layer_id, layers::try_into_line(layer))),
            "symbol" => {
                Self::Symbol(SymbolLayer::from_ffi_parts(layer_id, layers::try_into_symbol(layer)))
            }
            _ => Self::Opaque(OpaqueLayer { layer_id, layer_type, layer }),
        })
    }

    /// Returns the layer's ID.
    #[must_use]
    pub fn layer_id(&self) -> &str {
        match self {
            Self::Circle(l) => l.layer_id(),
            Self::Fill(l) => l.layer_id(),
            Self::Line(l) => l.layer_id(),
            Self::Symbol(l) => l.layer_id(),
            Self::Opaque(l) => l.layer_id(),
        }
    }

    /// Returns the MapLibre style-spec type string for this layer
    /// (e.g. `"circle"`, `"fill"`, `"raster"`).
    #[must_use]
    pub fn type_str(&self) -> &str {
        match self {
            Self::Circle(_) => "circle",
            Self::Fill(_) => "fill",
            Self::Line(_) => "line",
            Self::Symbol(_) => "symbol",
            Self::Opaque(l) => l.type_str(),
        }
    }

    pub(crate) fn into_layer_ptr(self) -> UniquePtr<ffi::CxxLayer> {
        match self {
            Self::Circle(l) => layers::circle_into_layer(l.into_inner()),
            Self::Fill(l) => layers::fill_into_layer(l.into_inner()),
            Self::Line(l) => layers::line_into_layer(l.into_inner()),
            Self::Symbol(l) => layers::symbol_into_layer(l.into_inner()),
            Self::Opaque(l) => l.into_inner(),
        }
    }
}
