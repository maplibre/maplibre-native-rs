mod maplibre;

slint::include_modules!();

fn main() {
    let ui = MainWindow::new().unwrap();

    // The window has no size until `ui.run()` lays it out, so seed the map with the preferred size.
    // `on_map_size_changed` (in `init`) resizes it once shown.
    let (width, height) = maplibre::DEFAULT_MAP_SIZE;
    let map = maplibre::create_map(width, height);

    maplibre::init(&ui, &map);

    ui.run().unwrap();
}
