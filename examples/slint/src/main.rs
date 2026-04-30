mod maplibre;

slint::include_modules!();

fn main() {
    env_logger::init();

    let mut wgpu_settings = slint::wgpu::WGPUSettings::default();
    // MapLibre's shaders use multiple storage buffers in the vertex stage.
    wgpu_settings.device_required_limits.max_storage_buffers_per_shader_stage = 8;
    // Some drivers/backends end up with 0 here in downlevel defaults, which fails bind group validation.
    wgpu_settings.device_required_limits.max_uniform_buffer_binding_size = 4096;
    wgpu_settings.device_required_limits.max_storage_buffer_binding_size = 4096;

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
