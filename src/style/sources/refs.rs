use std::fmt;
use std::marker::PhantomData;

use cxx::UniquePtr;

use super::geojson::GeoJsonSourceRefMut;
use crate::bridge::sources;
use crate::SourceId;

/// MapLibre style-spec source type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SourceType {
    /// Vector tile source.
    Vector,
    /// Raster tile source.
    Raster,
    /// Raster DEM source.
    RasterDem,
    /// GeoJSON source.
    GeoJson,
    /// Video source.
    Video,
    /// Annotations source.
    Annotations,
    /// Image source.
    Image,
    /// Custom vector source.
    CustomVector,
    /// Source type not known by this crate version.
    Unknown,
}

impl From<sources::SourceType> for SourceType {
    fn from(value: sources::SourceType) -> Self {
        match value {
            sources::SourceType::Vector => Self::Vector,
            sources::SourceType::Raster => Self::Raster,
            sources::SourceType::RasterDEM => Self::RasterDem,
            sources::SourceType::GeoJSON => Self::GeoJson,
            sources::SourceType::Video => Self::Video,
            sources::SourceType::Annotations => Self::Annotations,
            sources::SourceType::Image => Self::Image,
            sources::SourceType::CustomVector => Self::CustomVector,
            _ => Self::Unknown,
        }
    }
}

/// A mutable reference to a source owned by the current style.
#[non_exhaustive]
pub enum SourceRefMut<'a> {
    /// A GeoJSON source.
    GeoJson(GeoJsonSourceRefMut<'a>),
    /// A source type that does not have a typed Rust reference yet.
    Opaque(OpaqueSourceRefMut<'a>),
}

impl SourceRefMut<'_> {
    /// Wraps a raw FFI handle.
    /// The caller must ensure the returned reference's
    /// lifetime stays bounded by the borrow of the owning style.
    pub(crate) fn from_ffi(source: UniquePtr<sources::SourceHandle>) -> Option<Self> {
        let geojson = source.as_ref()?.asGeoJson();
        Some(if geojson.is_null() {
            Self::Opaque(OpaqueSourceRefMut::new(source))
        } else {
            Self::GeoJson(GeoJsonSourceRefMut::new(geojson))
        })
    }

    /// Returns the source ID.
    #[must_use]
    pub fn source_id(&self) -> SourceId {
        match self {
            Self::GeoJson(source) => source.source_id(),
            Self::Opaque(source) => source.source_id(),
        }
    }

    /// Returns the MapLibre style-spec source type.
    #[must_use]
    pub fn source_type(&self) -> SourceType {
        match self {
            Self::GeoJson(_) => SourceType::GeoJson,
            Self::Opaque(source) => source.source_type(),
        }
    }
}

impl fmt::Debug for SourceRefMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceRefMut")
            .field("source_id", &self.source_id())
            .field("source_type", &self.source_type())
            .finish_non_exhaustive()
    }
}

/// A mutable reference to a style-owned source without a typed Rust wrapper.
pub struct OpaqueSourceRefMut<'a> {
    source: UniquePtr<sources::SourceHandle>,
    _marker: PhantomData<&'a mut ()>,
    _not_send: PhantomData<*mut ()>,
}

impl OpaqueSourceRefMut<'_> {
    fn new(source: UniquePtr<sources::SourceHandle>) -> Self {
        Self { source, _marker: PhantomData, _not_send: PhantomData }
    }

    /// Returns the source ID.
    #[must_use]
    pub fn source_id(&self) -> SourceId {
        SourceId::new(self.source.sourceId())
    }

    /// Returns the MapLibre style-spec source type.
    #[must_use]
    pub fn source_type(&self) -> SourceType {
        self.source.sourceType().into()
    }
}

impl fmt::Debug for OpaqueSourceRefMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpaqueSourceRefMut")
            .field("source_id", &self.source_id())
            .field("source_type", &self.source_type())
            .finish()
    }
}
