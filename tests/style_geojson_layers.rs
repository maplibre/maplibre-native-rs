//! Integration tests for programmatic GeoJSON sources and data layers.

use std::num::NonZeroU32;
use std::path::PathBuf;

use maplibre_native::{
    CircleLayer, Color, FillLayer, GeoJson, GeoJsonSource, Image, ImageRenderer,
    ImageRendererBuilder, LineCap, LineJoin, LineLayer, Static,
};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(name)
}

fn overlay_geojson() -> GeoJson {
    r#"{
        "type": "FeatureCollection",
        "features": [
            {
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
            },
            {
                "type": "Feature",
                "properties": {},
                "geometry": {
                    "type": "LineString",
                    "coordinates": [[-60.0, 0.0], [60.0, 0.0]]
                }
            },
            {
                "type": "Feature",
                "properties": {},
                "geometry": {
                    "type": "Point",
                    "coordinates": [0.0, 0.0]
                }
            }
        ]
    }"#
    .parse::<GeoJson>()
    .expect("inline GeoJSON should parse")
}

fn has_non_background_pixel(image: &image::RgbaImage) -> bool {
    image.pixels().any(|pixel| {
        let [red, green, blue, alpha] = pixel.0;
        alpha > 0 && !(red > 245 && green > 220 && blue < 20)
    })
}

fn renderer() -> ImageRenderer<Static> {
    let mut renderer = ImageRendererBuilder::new()
        .with_size(NonZeroU32::new(128).unwrap(), NonZeroU32::new(128).unwrap())
        .with_pixel_ratio(1.0)
        .build_static_renderer();

    renderer
        .load_style_from_path(fixture_path("test-style.json"))
        .expect("test style path should be valid")
        .wait()
        .expect("style should load");
    renderer
}

fn render_geojson(renderer: &mut ImageRenderer<Static>) -> Image {
    let image =
        renderer.render_static(0.0, 0.0, 1.0, 0.0, 0.0).expect("GeoJSON layers should render");
    assert!(
        has_non_background_pixel(image.as_image()),
        "GeoJSON layers did not draw any non-background pixels",
    );
    image
}

#[test]
fn geojson_source_renders_circle_line_and_fill_layers() {
    let mut renderer = renderer();

    let mut source = GeoJsonSource::new("geojson-test-source");
    let geojson = overlay_geojson();
    source.set_geojson(&geojson);

    let mut style = renderer.style();
    let source_id = style.add_source(source).expect("GeoJSON source should be added");

    let mut fill = FillLayer::new("geojson-test-fill", &source_id);
    fill.set_fill_color(Color::rgba(0.0, 0.8, 0.2, 0.9));
    fill.set_fill_outline_color(Color::rgb(0.0, 0.2, 0.0));
    style.add_layer(fill).expect("fill layer should be added");

    let mut line = LineLayer::new("geojson-test-line", &source_id);
    line.set_line_color(Color::rgb(0.0, 0.0, 0.0));
    line.set_line_cap(LineCap::Round);
    line.set_line_join(LineJoin::Round);
    line.set_line_width(6.0);
    style.add_layer(line).expect("line layer should be added");

    let mut circle = CircleLayer::new("geojson-test-circle", &source_id);
    circle.set_circle_color(Color::rgb(0.0, 0.2, 1.0));
    circle.set_circle_radius(12.0);
    circle.set_circle_stroke_color(Color::rgb(1.0, 1.0, 1.0));
    circle.set_circle_stroke_opacity(1.0);
    circle.set_circle_stroke_width(2.0);
    style.add_layer(circle).expect("circle layer should be added");

    let image = render_geojson(&mut renderer);

    assert_eq!(image.as_image().width(), 128);
    assert_eq!(image.as_image().height(), 128);
}

#[test]
fn layer_management_methods_smoke_test() {
    let mut renderer = renderer();
    let mut style = renderer.style();

    style.remove_layer("missing-layer");
    style.remove_source("missing-source");

    let mut source = GeoJsonSource::new("removable-source");
    let geojson = overlay_geojson();
    source.set_geojson(&geojson);
    let source_id = style.add_source(source).expect("GeoJSON source should be added");

    let mut circle = CircleLayer::new("removable-layer", &source_id);
    circle.set_circle_color(Color::rgb(1.0, 0.0, 0.0));
    circle.set_circle_radius(30.0);
    let removable_layer = style.add_layer(circle).expect("circle layer should be added");
    assert_eq!(removable_layer.as_str(), "removable-layer");

    let duplicate = CircleLayer::new("removable-layer", &source_id);
    assert!(style.add_layer(duplicate).is_err());

    let mut before = CircleLayer::new("before-removable-layer", &source_id);
    before.set_circle_color(Color::rgb(0.0, 1.0, 0.0));
    before.set_circle_radius(20.0);
    let before_layer = style
        .add_layer_before(before, &removable_layer)
        .expect("layer should be added before existing layer");
    assert_eq!(before_layer.as_str(), "before-removable-layer");

    let mut missing_before = CircleLayer::new("missing-before-layer", &source_id);
    missing_before.set_circle_color(Color::rgb(0.0, 0.0, 1.0));
    missing_before.set_circle_radius(10.0);
    let missing_before_layer = style
        .add_layer_before(missing_before, "not-present")
        .expect("layer with missing before id should append");
    assert_eq!(missing_before_layer.as_str(), "missing-before-layer");

    style.remove_layer(&before_layer);
    style.remove_layer(&missing_before_layer);
    style.remove_layer(&removable_layer);
    style.remove_source(&source_id);

    let mut source = GeoJsonSource::new("removable-source");
    source.set_geojson(&geojson);
    let source_id = style.add_source(source).expect("GeoJSON source should be re-added");

    let mut circle = CircleLayer::new("removable-layer", &source_id);
    circle.set_circle_color(Color::rgb(0.0, 0.0, 1.0));
    circle.set_circle_radius(30.0);
    let removable_layer = style.add_layer(circle).expect("circle layer should be re-added");
    assert_eq!(removable_layer.as_str(), "removable-layer");

    let image = render_geojson(&mut renderer);
    assert_eq!(image.as_image().width(), 128);
    assert_eq!(image.as_image().height(), 128);
}
