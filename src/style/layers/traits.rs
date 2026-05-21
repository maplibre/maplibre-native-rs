use crate::bridge::{ffi, layers};
use crate::style::{CircleLayer, FillLayer, LineLayer, SymbolLayer};

use sealed::IntoLayer;

mod sealed {
    use crate::bridge::ffi;

    pub trait IntoLayer {
        fn layer_id(&self) -> &str;
        fn into_layer(self) -> cxx::UniquePtr<ffi::CxxLayer>;
    }
}

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
