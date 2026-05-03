mod maplibre;

use crate::maplibre::MapLibre;
use image::ImageReader;
use maplibre_native::Style;
use maplibre_native::layers::SymbolAnchorType;
use maplibre_native::{GeoJsonSource, Latitude, Longitude, SymbolLayer};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

slint::include_modules!();

fn main() {
    env_logger::init();

    let mut wgpu_settings = slint::wgpu::WGPUSettings::default();
    // MapLibre's shaders use multiple storage buffers in the vertex stage.
    wgpu_settings.device_required_limits.max_storage_buffers_per_shader_stage = 8;
    // Keep these non-zero and high enough for MapLibre shader bind groups.
    // 65536 covers the observed 6656-byte binding and aligns with common WebGPU minimums.
    wgpu_settings.device_required_limits.max_uniform_buffer_binding_size = 65_536;
    wgpu_settings.device_required_limits.max_storage_buffer_binding_size = 65_536;

    slint::BackendSelector::new()
        .require_wgpu_29(slint::wgpu::WGPUConfiguration::Automatic(wgpu_settings))
        .select()
        .unwrap();
    let ui = MainWindow::new().unwrap();

    let size = ui.get_map_size();
    // println!("Size: {:?}", size);
    let map = maplibre::create_map(size);
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
    let image_id = style.add_image("The id", &image, true);

    let mut geo_json_source = GeoJsonSource::new("geojsonsourceid");
    geo_json_source.set_point(Latitude(52.67655), Longitude(13.28387));
    let source_id = style.add_source(geo_json_source);

    let layer = SymbolLayer::new("Layer id", &source_id);
    layer.set_icon_image(&image_id);
    layer.set_icon_anchor(SymbolAnchorType::Bottom);
    style.add_layer(layer);
}
