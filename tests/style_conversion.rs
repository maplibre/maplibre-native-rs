//! Integration tests for style-spec conversion APIs.
#![cfg(feature = "json")]

use std::num::NonZeroU32;
use std::path::PathBuf;

use maplibre_native::{
    AnyLayer, CameraUpdate, GeoJson, GeoJsonSource, ImageRendererBuilder, LatLng, StyleError,
};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(name)
}

#[test]
fn from_json_str_parses_typed_circle_layer() {
    let json = r##"{
        "id": "my-circles",
        "type": "circle",
        "source": "src-id",
        "paint": { "circle-color": "#ff0000", "circle-radius": 4 }
    }"##;
    let layer = AnyLayer::from_json_str(json).expect("parse should succeed");
    assert_eq!(layer.layer_id(), "my-circles");
    assert_eq!(layer.type_str(), "circle");
    assert!(matches!(layer, AnyLayer::Circle(_)));
}

#[test]
fn from_json_str_parses_opaque_for_unsupported_type() {
    // Background layers don't have a typed wrapper in this crate; we expect Opaque.
    let json = r##"{
        "id": "bg",
        "type": "background",
        "paint": { "background-color": "#abcdef" }
    }"##;
    let layer = AnyLayer::from_json_str(json).expect("parse should succeed");
    assert_eq!(layer.layer_id(), "bg");
    assert_eq!(layer.type_str(), "background");
    assert!(matches!(layer, AnyLayer::Opaque(_)));
}

#[test]
fn from_json_str_rejects_invalid_layer_json() {
    // Valid JSON, but missing the required `"type"` field: MapLibre Native
    // rejects it, so this is a `Native` error, not a JSON parse error.
    let json = r#"{ "id": "broken", "source": "foo" }"#;
    let err = AnyLayer::from_json_str(json).expect_err("missing type must error");
    assert!(matches!(err, StyleError::Native(_)), "unexpected error: {err:?}");
}

#[test]
fn from_json_str_reports_json_parse_error() {
    // Not valid JSON at all: this fails in serde_json, not MapLibre Native.
    let err = AnyLayer::from_json_str("this is not json").expect_err("invalid JSON must error");
    assert!(matches!(err, StyleError::Json(_)), "unexpected error: {err:?}");
}

#[test]
fn add_layer_from_json_renders() {
    let mut renderer = ImageRendererBuilder::new()
        .with_size(NonZeroU32::new(128).unwrap(), NonZeroU32::new(128).unwrap())
        .with_pixel_ratio(1.0)
        .build_static_renderer();
    renderer
        .load_style_from_path(fixture_path("test-style.json"))
        .expect("test style path is valid")
        .wait()
        .expect("style loaded");

    let mut style = renderer.style();
    let mut source = GeoJsonSource::new("shapes");
    let geojson = r#"{
        "type": "FeatureCollection",
        "features": [{
            "type": "Feature",
            "properties": {},
            "geometry": {
                "type": "Polygon",
                "coordinates": [[
                    [-45.0, -45.0],
                    [45.0, -45.0],
                    [45.0, 45.0],
                    [-45.0, 45.0],
                    [-45.0, -45.0]
                ]]
            }
        }]
    }"#
    .parse::<GeoJson>()
    .expect("parse GeoJSON");
    source.set_geojson(&geojson);
    let _src_id = style.add_source(source).expect("source added");

    let layer = AnyLayer::from_json_str(
        r##"{
            "id": "shapes-fill",
            "type": "fill",
            "source": "shapes",
            "paint": { "fill-color": "#00ff00", "fill-opacity": 1.0 }
        }"##,
    )
    .expect("parse should succeed");
    assert!(matches!(layer, AnyLayer::Fill(_)), "expected fill variant");
    style.add_layer(layer).expect("layer added");

    let camera =
        CameraUpdate::new().center(LatLng { lat: 0.0, lng: 0.0 }).zoom(1.0).bearing(0.0).pitch(0.0);
    let image = renderer.render_static(&camera).expect("render");

    let buf = image.as_image();
    assert_eq!(buf.width(), 128);
    assert_eq!(buf.height(), 128);
    // Background is `#ff00f0` (pink). The added fill layer paints `#00ff00`
    // over a polygon at the center; a pixel with significantly more green than
    // red can only come from our layer.
    let saw_green = buf.pixels().any(|p| {
        let [r, g, _b, a] = p.0;
        a >= 250 && i32::from(g) > i32::from(r) + 20
    });
    assert!(saw_green, "added fill layer did not render");
}
