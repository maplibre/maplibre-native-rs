use sealed::IntoSource;

use crate::bridge::{ffi, sources};
use crate::style::{AnySource, GeoJsonSource};

mod sealed {
    use crate::bridge::ffi;

    pub trait IntoSource {
        fn source_id(&self) -> &str;
        fn into_source(self) -> cxx::UniquePtr<ffi::CxxSource>;
    }
}

/// A style source type that can be added to a [`StyleRef`](crate::StyleRef).
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

impl IntoSource for AnySource {
    fn source_id(&self) -> &str {
        self.source_id()
    }

    fn into_source(self) -> cxx::UniquePtr<ffi::CxxSource> {
        self.into_inner()
    }
}

impl Source for AnySource {}
