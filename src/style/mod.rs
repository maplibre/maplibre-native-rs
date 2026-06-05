//! Style abstractions for sources, layers, and images.

mod color;
mod error;
mod geojson;
mod image;
mod layers;
mod sources;
mod style_ref;
#[cfg(feature = "json")]
mod value;

pub use color::Color;
pub use error::StyleError;
pub use geojson::{GeoJson, GeoJsonError};
pub use image::ImageId;
pub use layers::{
    AnyLayer, CircleLayer, FillLayer, Layer, LayerId, LineCap, LineJoin, LineLayer, OpaqueLayer,
    SymbolAnchor, SymbolLayer,
};
pub use sources::{
    AnySource, GeoJsonSource, GeoJsonSourceRefMut, OpaqueSource, OpaqueSourceRefMut, Source,
    SourceId, SourceRefMut, SourceType,
};
pub use style_ref::StyleRef;
