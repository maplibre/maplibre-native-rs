//! Style abstractions for sources, layers, and images.

mod color;
mod error;
mod geojson;
mod image;
mod layers;
mod map_style;
mod sources;

pub use color::Color;
pub use error::StyleError;
pub use geojson::{GeoJson, GeoJsonError};
pub use image::ImageId;
pub use layers::{
    CircleLayer, FillLayer, Layer, LayerId, LineCap, LineJoin, LineLayer, SymbolAnchor, SymbolLayer,
};
pub use map_style::Style;
pub use sources::{GeoJsonSource, Source, SourceId};
