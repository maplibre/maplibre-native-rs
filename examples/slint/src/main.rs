mod maplibre;

slint::include_modules!();

fn main() {
    env_logger::init();

    let mut wgpu_settings = slint::wgpu::WGPUSettings::default();
    wgpu_settings.device_required_limits.max_storage_buffers_per_shader_stage = 1;

    slint::BackendSelector::new()
        .require_wgpu_29(slint::wgpu::WGPUConfiguration::Automatic(wgpu_settings))
        .select()
        .unwrap();
    let ui = MainWindow::new().unwrap();

    let size = ui.get_map_size();
    // println!("Size: {:?}", size);
    let map = maplibre::create_map(size);

    maplibre::init(&ui, &map);

    ui.run().unwrap();
}
