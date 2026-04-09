mod maplibre;
use maplibre_native::Style;
use image::ImageReader;
use std::path::Path;

slint::include_modules!();

fn main() {
    let ui = MainWindow::new().unwrap();

    let size = ui.get_map_size();
    let map = maplibre::create_map(size);

    let mut map_borrow = map.borrow_mut();
    let mut style = Style::get_ref(map_borrow.renderer());

    let image = ImageReader::open(Path::new(env!("CARGO_MANIFEST_DIR")).join("ui").join("icons").join("Marker.png")).unwrap().decode().unwrap();
    style.add_image("The id", &image, true);
    drop(style);
    drop(map_borrow);

    maplibre::init(&ui, &map);

    ui.run().unwrap();
}
