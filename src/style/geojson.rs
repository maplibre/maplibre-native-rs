use std::{fmt, str::FromStr};

use cxx::UniquePtr;

use crate::bridge::geojson;

/// Error returned when creating a MapLibre Native GeoJSON value.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GeoJsonError {
    /// MapLibre Native rejected or failed to serialize the GeoJSON data.
    #[error("GeoJSON error: {0}")]
    Native(String),

    /// The `geojson` crate value could not be serialized to JSON.
    #[cfg(feature = "geojson")]
    #[error("failed to serialize GeoJSON: {0}")]
    Serialize(#[from] serde_json::Error),
}

/// A GeoJSON value prepared for MapLibre Native.
///
/// This type owns MapLibre Native's C++ GeoJSON representation.
///
/// MapLibre Native's GeoJSON representation is two-dimensional and silently
/// drops `bbox`, foreign members, and any coordinate beyond X/Y, regardless of
/// the construction path.
pub struct GeoJson {
    inner: UniquePtr<geojson::GeoJson>,
}

impl GeoJson {
    fn from_json_str(json: &str) -> Result<Self, GeoJsonError> {
        Ok(Self {
            inner: geojson::parse(json).map_err(|error| GeoJsonError::Native(error.to_string()))?,
        })
    }

    /// Serializes this value to a GeoJSON string using MapLibre Native.
    ///
    /// # Errors
    ///
    /// Returns an error if MapLibre Native fails to serialize the value.
    pub fn to_json_string(&self) -> Result<String, GeoJsonError> {
        geojson::stringify(&self.inner).map_err(|error| GeoJsonError::Native(error.to_string()))
    }

    pub(crate) fn as_inner(&self) -> &geojson::GeoJson {
        self.inner.as_ref().expect("GeoJson bridge value is unexpectedly null")
    }
}

#[cfg(feature = "geojson")]
impl TryFrom<&::geojson::GeoJson> for GeoJson {
    type Error = GeoJsonError;

    fn try_from(value: &::geojson::GeoJson) -> Result<Self, Self::Error> {
        Self::from_json_str(&serde_json::to_string(value)?)
    }
}

#[cfg(feature = "geojson")]
impl TryFrom<::geojson::GeoJson> for GeoJson {
    type Error = GeoJsonError;

    fn try_from(value: ::geojson::GeoJson) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl Clone for GeoJson {
    fn clone(&self) -> Self {
        Self { inner: geojson::clone(&self.inner) }
    }
}

impl FromStr for GeoJson {
    type Err = GeoJsonError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_json_str(s)
    }
}

impl fmt::Debug for GeoJson {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeoJson").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::GeoJson;

    #[test]
    fn parse_clone_and_stringify_geojson() {
        let geojson = r#"{"type":"Point","coordinates":[1.0,2.0]}"#
            .parse::<GeoJson>()
            .expect("valid point GeoJSON should parse");
        let cloned = geojson.clone();
        let serialized = cloned.to_json_string().expect("valid GeoJSON should serialize");

        assert!(serialized.contains(r#""Point""#));
    }

    #[test]
    fn from_str_reports_invalid_geojson() {
        assert!("not json".parse::<GeoJson>().is_err());
    }

    #[test]
    fn from_str_drops_bbox_like_maplibre_native() {
        let geojson = r#"{"type":"Point","bbox":[0,0,1,1],"coordinates":[1.0,2.0]}"#
            .parse::<GeoJson>()
            .expect("valid point GeoJSON should parse");

        assert!(!geojson.to_json_string().expect("GeoJSON should serialize").contains("bbox"));
    }

    #[test]
    fn from_str_drops_z_coordinates_like_maplibre_native() {
        let geojson = r#"{"type":"Point","coordinates":[1.0,2.0,12345.6789]}"#
            .parse::<GeoJson>()
            .expect("valid 3D point GeoJSON should parse");
        let serialized = geojson.to_json_string().expect("GeoJSON should serialize");
        let json: serde_json::Value =
            serde_json::from_str(&serialized).expect("serialized GeoJSON should parse as JSON");
        let coordinates = json
            .get("coordinates")
            .and_then(serde_json::Value::as_array)
            .expect("serialized point should have coordinate array");

        assert_eq!(coordinates.len(), 2);
    }

    #[test]
    fn clone_survives_original_drop() {
        let cloned = {
            let geojson = r#"{"type":"Point","coordinates":[1.0,2.0]}"#
                .parse::<GeoJson>()
                .expect("valid point GeoJSON should parse");
            geojson.clone()
        };

        assert!(cloned
            .to_json_string()
            .expect("cloned GeoJSON should serialize")
            .contains("Point"));
    }

    #[cfg(feature = "geojson")]
    #[test]
    fn converts_from_geojson_crate_value() {
        let geojson = r#"{"type":"Feature","geometry":null,"properties":{"name":"empty"}}"#
            .parse::<::geojson::GeoJson>()
            .expect("valid geojson crate value should parse");
        let converted = GeoJson::try_from(geojson)
            .expect("geojson crate value should convert to MapLibre GeoJSON");

        assert!(converted
            .to_json_string()
            .expect("converted GeoJSON should serialize")
            .contains(r#""Feature""#));
    }
}
