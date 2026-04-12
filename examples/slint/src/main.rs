mod maplibre;

slint::include_modules!();

fn main() {
    slint::BackendSelector::new()
        .require_wgpu_29(slint::wgpu_29::WGPUConfiguration::default())
        .select()
        .unwrap();
    let ui = MainWindow::new().unwrap();

    let size = ui.get_map_size();
    // println!("Size: {:?}", size);
    let map = maplibre::create_map(size);

    maplibre::init(&ui, &map);

    ui.run().unwrap();
}
