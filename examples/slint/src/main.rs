mod maplibre;

use crate::maplibre::MapLibre;
use image::ImageReader;
use maplibre_native::layers::SymbolAnchorType;
use maplibre_native::Style;
use maplibre_native::{GeoJsonSource, Latitude, Longitude, SymbolLayer};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

slint::include_modules!();

const DEFAULT_WIDTH: u32 = 800;
const DEFAULT_HEIGHT: u32 = 600;

fn main() {
    env_logger::init();

    let mut wgpu_settings = slint::wgpu_29::WGPUSettings::default();
    // MapLibre's shaders use multiple storage buffers in the vertex stage.
    wgpu_settings.device_required_limits.max_storage_buffers_per_shader_stage = 8;
    // Keep these non-zero and high enough for MapLibre shader bind groups.
    // 65536 covers the observed 6656-byte binding and aligns with common WebGPU minimums.
    wgpu_settings.device_required_limits.max_uniform_buffer_binding_size = 65_536;
    wgpu_settings.device_required_limits.max_storage_buffer_binding_size = 65_536;

    slint::BackendSelector::new()
        .require_wgpu_29(slint::wgpu_29::WGPUConfiguration::Automatic(wgpu_settings))
        .select()
        .unwrap();
    let ui = MainWindow::new().unwrap();
    let map =
        maplibre::create_map(Size { width: DEFAULT_WIDTH as f32, height: DEFAULT_HEIGHT as f32 });
    // println!("Size: {:?}", size);
    map.borrow_mut()
        .renderer()
        .load_style_from_url(&"https://tiles.openfreemap.org/styles/liberty".parse().unwrap());
    style(&map);

    maplibre::init(&ui, &map);

    ui.run().unwrap();
}

// Style the map and add a marker
fn style(map: &Rc<RefCell<MapLibre>>) {
    let mut map_borrow = map.borrow_mut();
    let mut style = Style::get_ref(map_borrow.renderer());

    let image = ImageReader::open(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("ui").join("icons").join("Marker.png"),
    )
    .unwrap()
    .decode()
    .unwrap();
    let image_id = style.add_image("The id", &image, true).unwrap();

    let mut shapes_source = GeoJsonSource::new("shapes-source");
    let shapes = r#"{
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "properties": {},
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[
                        [-30.0, -30.0],
                        [30.0, -30.0],
                        [30.0, 30.0],
                        [-30.0, 30.0],
                        [-30.0, -30.0]
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
            }
        ]
    }"#
    .parse::<GeoJson>()
    .expect("shapes GeoJSON should parse");
    shapes_source.set_geojson(&shapes);
    let shapes_id = style.add_source(shapes_source).unwrap();

    let mut markers_source = GeoJsonSource::new("markers-source");
    let markers = r#"{
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "properties": {},
                "geometry": {
                    "type": "Point",
                    "coordinates": [13.28387, 52.67655]
                }
            }
        ]
    }"#
    .parse::<GeoJson>()
    .expect("marker GeoJSON should parse");
    markers_source.set_geojson(&markers);
    let markers_id = style.add_source(markers_source).unwrap();

    let mut fill = FillLayer::new("Fill layer id", &shapes_id);
    fill.set_fill_color(Color::rgba(0.0, 0.45, 0.95, 0.35));
    style.add_layer(fill).unwrap();

    let mut line = LineLayer::new("Line layer id", &shapes_id);
    line.set_line_color(Color::rgb(0.0, 0.5, 0.8));
    line.set_line_width(3.0);
    style.add_layer(line).unwrap();

    let mut circle = CircleLayer::new("Circle layer id", &shapes_id);
    circle.set_circle_color(Color::rgba(0.0, 0.7, 0.5, 0.85));
    circle.set_circle_radius(7.0);
    style.add_layer(circle).unwrap();

    let mut layer = SymbolLayer::new("Layer id", &markers_id);
    layer.set_icon_image(&image_id);
    layer.set_icon_anchor(SymbolAnchor::Bottom);
    style.add_layer(layer).unwrap();
}
