//! Integration tests for programmatic GeoJSON sources and data layers.

use std::num::NonZeroU32;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use maplibre_native::{
    CircleLayer, Color, FillLayer, GeoJson, GeoJsonSource, ImageRendererBuilder, LineLayer, Style,
};

const RENDER_TIMEOUT: Duration = Duration::from_secs(5);

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(name)
}

fn has_non_background_pixel(image: &image::RgbaImage) -> bool {
    image.pixels().any(|pixel| {
        let [red, green, blue, alpha] = pixel.0;
        alpha > 0 && !(red > 245 && green > 220 && blue < 20)
    })
}

#[test]
fn geojson_source_renders_circle_line_and_fill_layers() {
    let mut renderer = ImageRendererBuilder::new()
        .with_size(NonZeroU32::new(128).unwrap(), NonZeroU32::new(128).unwrap())
        .with_pixel_ratio(1.0)
        .build_static_renderer();

    renderer.load_style_from_path(fixture_path("test-style.json")).expect("test style should load");
    let background =
        renderer.render_static(0.0, 0.0, 0.0, 0.0, 0.0).expect("background style should render");
    assert_eq!(background.as_image().width(), 128);

    let mut source = GeoJsonSource::new("geojson-test-source");
    let geojson = r#"{
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
    .expect("inline GeoJSON should parse");
    source.set_geojson(&geojson);

    let mut style = Style::get_ref(&mut renderer);
    let source_id = style.add_source(source);

    let mut fill = FillLayer::new("geojson-test-fill", &source_id);
    fill.set_fill_color(Color::rgba(0.0, 0.8, 0.2, 0.9));
    fill.set_fill_outline_color(Color::rgb(0.0, 0.2, 0.0));
    style.add_layer(fill);

    let mut line = LineLayer::new("geojson-test-line", &source_id);
    line.set_line_color(Color::rgb(0.0, 0.0, 0.0));
    line.set_line_width(6.0);
    style.add_layer(line);

    let mut circle = CircleLayer::new("geojson-test-circle", &source_id);
    circle.set_circle_color(Color::rgb(0.0, 0.2, 1.0));
    circle.set_circle_radius(12.0);
    style.add_layer(circle);

    let started = Instant::now();
    let image = loop {
        let frame =
            renderer.render_static(0.0, 0.0, 1.0, 0.0, 0.0).expect("GeoJSON layers should render");
        if has_non_background_pixel(frame.as_image()) {
            break frame;
        }
        assert!(
            started.elapsed() < RENDER_TIMEOUT,
            "GeoJSON layers did not draw pixels within {RENDER_TIMEOUT:?}",
        );
    };

    assert_eq!(image.as_image().width(), 128);
    assert_eq!(image.as_image().height(), 128);
}
