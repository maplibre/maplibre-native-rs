use crate::bridge::{ffi, layers, sources};
use crate::style::{CircleLayer, FillLayer, GeoJsonSource, LineLayer, SymbolLayer};

use sealed::{IntoLayer, IntoSource};

mod sealed {
    use crate::bridge::ffi;

    pub trait IntoLayer {
        fn layer_id(&self) -> &str;
        fn into_layer(self) -> cxx::UniquePtr<ffi::CxxLayer>;
    }

    pub trait IntoSource {
        fn source_id(&self) -> &str;
        fn into_source(self) -> cxx::UniquePtr<ffi::CxxSource>;
    }
}

/// A style source type that can be added to a [`Style`](crate::Style).
///
/// This trait is sealed; only source types provided by this crate can implement
/// it.
pub trait Source: IntoSource {}

impl IntoSource for GeoJsonSource {
    fn source_id(&self) -> &str {
        self.source_id()
    }

    fn into_source(self) -> cxx::UniquePtr<ffi::CxxSource> {
        sources::geojson_into_source(self.into_inner())
    }
}

impl Source for GeoJsonSource {}

/// A style layer type that can be added to a [`Style`](crate::Style).
///
/// This trait is sealed; only layer types provided by this crate can implement
/// it.
pub trait Layer: IntoLayer {}

impl IntoLayer for CircleLayer {
    fn layer_id(&self) -> &str {
        self.layer_id()
    }

    fn into_layer(self) -> cxx::UniquePtr<ffi::CxxLayer> {
        layers::circle_into_layer(self.into_inner())
    }
}

impl Layer for CircleLayer {}

impl IntoLayer for FillLayer {
    fn layer_id(&self) -> &str {
        self.layer_id()
    }

    fn into_layer(self) -> cxx::UniquePtr<ffi::CxxLayer> {
        layers::fill_into_layer(self.into_inner())
    }
}

impl Layer for FillLayer {}

impl IntoLayer for LineLayer {
    fn layer_id(&self) -> &str {
        self.layer_id()
    }

    fn into_layer(self) -> cxx::UniquePtr<ffi::CxxLayer> {
        layers::line_into_layer(self.into_inner())
    }
}

impl Layer for LineLayer {}

impl IntoLayer for SymbolLayer {
    fn layer_id(&self) -> &str {
        self.layer_id()
    }

    fn into_layer(self) -> cxx::UniquePtr<ffi::CxxLayer> {
        layers::symbol_into_layer(self.into_inner())
    }
}

impl Layer for SymbolLayer {}
