use std::fmt;
use std::marker::PhantomData;

use cxx::UniquePtr;

use crate::bridge::sources;
use crate::style::{GeoJson, SourceId};

/// A GeoJSON source for rendering geographic data.
pub struct GeoJsonSource {
    source_id: String,
    source: UniquePtr<sources::GeoJSONSource>,
}

impl GeoJsonSource {
    /// Create a new `GeoJSON` source with default options
    #[must_use]
    pub fn new(id: &str) -> Self {
        Self { source_id: id.to_owned(), source: sources::create(id) }
    }

    /// Sets the URL for loading `GeoJSON` data.
    pub fn set_url(&mut self, url: &str) {
        sources::setURL(&self.source, url);
    }

    pub(crate) fn source_id(&self) -> &str {
        &self.source_id
    }

    /// Sets the GeoJSON data for this source.
    pub fn set_geojson(&mut self, geojson: &GeoJson) {
        sources::setGeoJson(self.source.pin_mut(), geojson.as_inner());
    }

    pub(crate) fn into_inner(self) -> UniquePtr<sources::GeoJSONSource> {
        self.source
    }
}

impl fmt::Debug for GeoJsonSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeoJsonSource")
            .field("source_id", &self.source_id)
            .field("Pointer", &self.source.as_ptr())
            .finish()
    }
}

/// A mutable reference to a GeoJSON source owned by the current style.
pub struct GeoJsonSourceRefMut<'a> {
    source: UniquePtr<sources::GeoJSONSourceHandle>,
    _marker: PhantomData<&'a mut ()>,
    _not_send: PhantomData<*mut ()>,
}

impl GeoJsonSourceRefMut<'_> {
    pub(crate) fn new(source: UniquePtr<sources::GeoJSONSourceHandle>) -> Self {
        Self { source, _marker: PhantomData, _not_send: PhantomData }
    }

    /// Returns the source ID.
    #[must_use]
    pub fn source_id(&self) -> SourceId {
        SourceId::new(self.source.sourceId())
    }

    /// Sets the GeoJSON data for this source.
    pub fn set_geojson(&mut self, geojson: &GeoJson) {
        self.source.pin_mut().setGeoJson(geojson.as_inner());
    }
}

impl fmt::Debug for GeoJsonSourceRefMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeoJsonSourceRefMut").field("source_id", &self.source_id()).finish()
    }
}
