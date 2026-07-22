use crate::{MainWindow, MapAdapter};
use image::ImageReader;
use maplibre_native::{
    CircleLayer, Color, FillLayer, GeoJson, GeoJsonSource, LineLayer, ScreenCoordinate,
    SymbolAnchor, SymbolLayer,
};
use std::cell::{Cell, RefCell};
use std::path::Path;
use std::rc::Rc;
mod headless;
pub use headless::{MapLibre, create_map};
use slint::ComponentHandle;

/// Graphics context lifecycle.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Lifecycle {
    Uninitialized,
    Running,
    Stopped,
}

fn queue_frame(ui: &slint::Weak<MainWindow>, frame_requested: &Cell<bool>) {
    frame_requested.set(true);
    let _ = ui.upgrade_in_event_loop(|ui| ui.window().request_redraw());
}

pub fn init(
    ui: &MainWindow,
    map: &Rc<RefCell<MapLibre>>,
    style_url: impl Into<String>,
) -> Option<slint::Timer> {
    let style_url = style_url.into();
    let lifecycle = Rc::new(Cell::new(Lifecycle::Uninitialized));
    let frame_requested = Rc::new(Cell::new(false));

    ui.window()
        .set_rendering_notifier({
            let map = Rc::downgrade(map);
            let lifecycle = lifecycle.clone();
            let frame_requested = frame_requested.clone();
            let ui_weak = ui.as_weak();
            let styled = Cell::new(false);
            move |state, graphics_api| match state {
                slint::RenderingState::RenderingSetup => {
                    if lifecycle.get() != Lifecycle::Uninitialized {
                        eprintln!("graphics re-setup after teardown is unsupported");
                        return;
                    }
                    let slint::GraphicsAPI::WGPU29 { device, queue, .. } = graphics_api else {
                        return;
                    };
                    let ui = ui_weak.upgrade().unwrap();
                    let map = map.upgrade().unwrap();
                    let mut map = map.borrow_mut();

                    // Build the renderer now that the pixel ratio (scale factor) is known.
                    map.build_renderer(ui.window().scale_factor());
                    map.renderer().set_device_queue(device.clone(), queue.clone());

                    map.renderer().set_render_requested_callback({
                        let lifecycle = lifecycle.clone();
                        let frame_requested = frame_requested.clone();
                        let ui = ui.as_weak();
                        move || {
                            if lifecycle.get() == Lifecycle::Running {
                                queue_frame(&ui, &frame_requested);
                            }
                        }
                    });

                    map.renderer().load_style_from_url(
                        &style_url.parse().expect("style URL should be valid"),
                    );
                    lifecycle.set(Lifecycle::Running);
                    queue_frame(&ui_weak, &frame_requested);
                }
                slint::RenderingState::BeforeRendering => {
                    if lifecycle.get() != Lifecycle::Running
                        || !frame_requested.replace(false)
                    {
                        return;
                    }

                    let map = map.upgrade().unwrap();
                    let mut map = map.borrow_mut();
                    map.renderer().render_once();

                    if let Some(error) = map.style_loading_error() {
                        eprintln!("Failed to load map: {error}");
                        lifecycle.set(Lifecycle::Stopped);
                        return;
                    }

                    if map.style_loaded() && !styled.get() {
                        style(&mut map);
                        styled.set(true);
                    }

                    if let Some(image) = map.renderer().take_texture()
                        && let Ok(image) = image.try_into()
                    {
                        ui_weak
                            .upgrade()
                            .unwrap()
                            .global::<MapAdapter>()
                            .set_map_texture(image);
                    }
                }
                slint::RenderingState::RenderingTeardown => {
                    lifecycle.set(Lifecycle::Stopped);
                    frame_requested.set(false);
                }
                _ => {}
            }
        })
        .unwrap();

    ui.on_map_size_changed({
        let map = Rc::downgrade(map);
        move |size| {
            if size.width > 0. && size.height > 0. {
                let size =
                    maplibre_native::Size { width: size.width as u32, height: size.height as u32 };
                map.upgrade().unwrap().borrow_mut().set_map_size(size);
            }
        }
    });

    ui.global::<MapAdapter>().on_mouse_press({
        let map = Rc::downgrade(map);
        move |x: f32, y: f32| {
            map.upgrade()
                .unwrap()
                .borrow_mut()
                .set_position(ScreenCoordinate { x: x.into(), y: y.into() });
        }
    });

    ui.global::<MapAdapter>().on_mouse_move({
        let map = Rc::downgrade(map);
        let lifecycle = lifecycle.clone();
        move |x: f32, y: f32, ctrl: bool| {
            if lifecycle.get() != Lifecycle::Running {
                return;
            }
            let p = ScreenCoordinate { x: x.into(), y: y.into() };
            let map = map.upgrade().unwrap();
            let mut map = map.borrow_mut();
            let delta = p - map.position();
            if ctrl {
                const ROTATE_DEG_PER_PX: f64 = 0.5;
                const PITCH_DEG_PER_PX: f64 = 0.5;
                map.rotate_by((-delta.x * ROTATE_DEG_PER_PX) as f32);
                map.renderer().pitch_by(delta.y * PITCH_DEG_PER_PX);
            } else {
                map.renderer().move_by(delta);
            }
            map.set_position(p);
        }
    });

    ui.global::<MapAdapter>().on_wheel_zoom({
        let map = Rc::downgrade(map);
        let lifecycle = lifecycle.clone();
        move |x: f32, y: f32, delta: f32| {
            if lifecycle.get() != Lifecycle::Running {
                return;
            }
            const STEP: f64 = 1.2;
            const DELTA_PER_STEP: f64 = 60.0;
            let pos = ScreenCoordinate { x: x.into(), y: y.into() };
            let scale = STEP.powf(f64::from(delta) / DELTA_PER_STEP);
            map.upgrade().unwrap().borrow_mut().renderer().scale_by(scale, pos);
        }
    });

    pump_run_loop_timer()
}

fn pump_run_loop_timer() -> Option<slint::Timer> {
    // Slint drives Core Foundation, but libuv must be pumped explicitly.
    if !maplibre_native::RunLoopHandle::uses_libuv() {
        return None;
    }
    let timer = slint::Timer::default();
    timer.start(slint::TimerMode::Repeated, std::time::Duration::from_millis(16), || {
        maplibre_native::RunLoopHandle::current().tick();
    });
    Some(timer)
}

// Style the map and add a marker
fn style(map: &mut MapLibre) {
    let mut style = map.renderer().style();

    let image = ImageReader::open(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("ui").join("icons").join("Marker.png"),
    )
    .unwrap()
    .decode()
    .unwrap();
    let image_id = style.add_image("The id", &image, 1.0, true).unwrap();

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
