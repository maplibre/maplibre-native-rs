use crate::renderer::bridge::sources;
use cxx::UniquePtr;

pub struct Latitude(pub f64);
pub struct Longitude(pub f64);

pub struct GeoJsonSource {
    source_id: String,
    source: UniquePtr<sources::GeoJSONSource>,
}

impl GeoJsonSource {
    pub fn new(id: &str) -> Self {
        Self {
            source_id: id.to_owned(),
            source: sources::create(id),
        }
    }

    pub fn set_point(&mut self, latitude: Latitude, longitude: Longitude) {
        sources::setPoint(&self.source, latitude.0, longitude.0);
    }

    pub(crate) fn into_inner(self) -> UniquePtr<sources::GeoJSONSource> {
        self.source
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
