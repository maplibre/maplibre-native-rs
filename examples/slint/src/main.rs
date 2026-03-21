mod maplibre;

slint::include_modules!();

fn main() {
    let ui = MainWindow::new().unwrap();

    let size = ui.get_map_size();
    let map = maplibre::create_map(size);

    maplibre::init(&ui, &map);

    ui.run().unwrap();
}
