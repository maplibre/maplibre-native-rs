use crate::renderer::bridge::sources;
use cxx::UniquePtr;
use std::fmt;

/// Latitude coordinate value.
#[derive(Debug)]
pub struct Latitude(pub f64);

/// Longitude coordinate value.
#[derive(Debug)]
pub struct Longitude(pub f64);

/// A GeoJSON source for rendering geographic data.
pub struct GeoJsonSource {
    source_id: String,
    source: UniquePtr<sources::GeoJSONSource>,
}

impl GeoJsonSource {
    /// Creates a new GeoJSON source with the given ID.
    pub fn new(id: &str) -> Self {
        Self { source_id: id.to_owned(), source: sources::create(id) }
    }

    /// Sets the point for this source.
    pub fn set_point(&mut self, latitude: Latitude, longitude: Longitude) {
        sources::setPoint(&self.source, latitude.0, longitude.0);
    }

    pub(crate) fn into_inner(self) -> UniquePtr<sources::GeoJSONSource> {
        self.source
    }
}

impl fmt::Debug for GeoJsonSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeoJsonSource").field("source_id", &self.source_id).finish()
    }
}

impl super::StyleSourceRef for GeoJsonSource {
    fn source_id(&self) -> &str {
        &self.source_id
    }
}

impl From<GeoJsonSource> for super::StyleSource {
    fn from(value: GeoJsonSource) -> Self {
        Self::GeoJson(value)
    }
}
