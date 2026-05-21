pub(crate) mod bridge;
mod builder;
pub mod callbacks;
pub mod file_source;
mod image_renderer;
mod map_observer;
mod resource_options;
pub mod style;
pub mod tile_server_options;
pub use bridge::file_source::{FsErrorReason, ResourceKind};
pub use bridge::{
    ffi::{MapDebugOptions, MapMode},
    layers,
    map_observer::{MapLoadError, MapObserverCameraChangeMode},
    set_log_thread_enabled, Height, ScreenCoordinate, Size, Width, X, Y,
};
pub use builder::{ImageRendererBuilder, RunLoopMode};
pub use file_source::{register_file_source_callback, FileSourceRequestCallback, FsResponse};
pub use image_renderer::{Continuous, Image, ImageRenderer, RenderingError, Static, Tile};
pub use map_observer::MapObserver;
pub use resource_options::ResourceOptions;
pub use style::{
    CircleLayer, Color, FillLayer, GeoJson, GeoJsonError, GeoJsonSource, LineLayer, SourceId,
    Style, StyleLayer, StyleSource, StyleSourceRef, SymbolLayer,
};

/// Latitude coordinate value.
#[derive(Debug, Clone, Copy)]
pub struct Latitude(pub f64);

/// Longitude coordinate value.
#[derive(Debug, Clone, Copy)]
pub struct Longitude(pub f64);
