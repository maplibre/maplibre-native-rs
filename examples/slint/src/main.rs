mod maplibre;
use image::ImageReader;
use maplibre_native::Style;
use maplibre_native::{GeoJsonSource, Latitude, Longitude, SymbolLayer};
use maplibre::MapLibre;
use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;

slint::include_modules!();

fn main() {
    let ui = MainWindow::new().unwrap();

    let size = ui.get_map_size();
    let map = maplibre::create_map(size);

    style(&map);

    maplibre::init(&ui, &map);

    ui.run().unwrap();
}

fn style(map: &Rc<RefCell<MapLibre>>) {
    let mut map_borrow = map.borrow_mut();
    let mut style = Style::get_ref(map_borrow.renderer());

    let image = ImageReader::open(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("ui")
            .join("icons")
            .join("Marker.png"),
    )
    .unwrap()
    .decode()
    .unwrap();
    style.add_image("The id", &image, true);

    let mut geo_json_source = GeoJsonSource::new("geojsonsourceid");
    geo_json_source.set_point(Latitude(46.62381), Longitude(11.11785));
    let source_id = style.add_source(geo_json_source);

    let layer = SymbolLayer::new("Layer id", &source_id);
    style.add_layer(layer);
}
