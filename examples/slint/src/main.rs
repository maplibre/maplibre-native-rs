mod maplibre;

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

    maplibre::init(&ui, &map);

    ui.run().unwrap();
}
