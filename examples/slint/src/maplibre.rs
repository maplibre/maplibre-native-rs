use std::rc::Rc;

use slint::ComponentHandle;

use crate::{MainWindow, MapAdapter};
mod headless;
use std::cell::RefCell;
use std::path::Path;

pub use headless::{MapLibre, create_map};
use image::ImageReader;
use maplibre_native::{
    CircleLayer, Color, FillLayer, GeoJson, GeoJsonSource, LineLayer, ScreenCoordinate,
    SymbolAnchor, SymbolLayer,
};

pub fn init(ui: &MainWindow, map: &Rc<RefCell<MapLibre>>) {
    loop {
        let mut borrow = map.borrow_mut();
        borrow.renderer().render_once();

        if let Some(error) = borrow.style_loading_error() {
            panic!("Failed to load map: {}", error);
        }

        if borrow.style_loaded() {
            drop(borrow);
            style(map);
            break;
        }
    }

    ui.on_map_size_changed({
        let map = Rc::downgrade(map);
        move |size| {
            let size =
                maplibre_native::Size { width: size.width as u32, height: size.height as u32 };
            map.upgrade().unwrap().borrow_mut().renderer().set_map_size(size);
        }
    });

    ui.global::<MapAdapter>().on_tick_map_loop({
        let map = Rc::downgrade(map);
        let ui_handle = ui.as_weak();
        move || {
            let map = map.upgrade().unwrap();
            let mut map = map.borrow_mut();
            map.renderer().render_once();
            if map.updated() {
                let image = map.renderer().read_still_image();
                let size = image.size();
                let img = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
                    image.buffer(),
                    size.width,
                    size.height,
                );
                ui_handle
                    .upgrade()
                    .unwrap()
                    .global::<MapAdapter>()
                    .set_map_texture(slint::Image::from_rgba8(img)); // TODO: check if the image really changed, otherwise we don't need to clone!
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
        move |x: f32, y: f32, _z: bool| {
            let p = ScreenCoordinate { x: x.into(), y: y.into() };
            let map = map.upgrade().unwrap();
            let mut map = map.borrow_mut();
            let delta = p - map.position();
            map.renderer().move_by(delta);
            map.set_position(p);
        }
    });

    ui.global::<MapAdapter>().on_wheel_zoom({
        let map = Rc::downgrade(map);
        move |x: f32, y: f32, delta: f32| {
            const STEP: f64 = 1.2;
            let pos = ScreenCoordinate { x: x.into(), y: y.into() };
            let scale = if delta > 0. { STEP } else { 1.0 / STEP };
            map.upgrade().unwrap().borrow_mut().renderer().scale_by(scale, pos);
        }
    });
}

fn style(map: &Rc<RefCell<MapLibre>>) {
    let mut map_borrow = map.borrow_mut();
    let mut style = map_borrow.renderer().style();

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
