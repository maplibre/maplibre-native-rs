use crate::MainWindow;
use crate::MapAdapter;
use maplibre_native::Height;
use maplibre_native::ScreenCoordinate;
use maplibre_native::Width;
use maplibre_native::layers::SymbolAnchorType;
use slint::ComponentHandle;
use std::rc::Rc;
mod headless;
pub use headless::MapLibre;
pub use headless::create_map;
use image::ImageReader;
use maplibre_native::Style;
use maplibre_native::{GeoJsonSource, Latitude, Longitude, SymbolLayer};
use maplibre_native::{X, Y};
use std::cell::RefCell;
use std::path::Path;

pub fn init(ui: &MainWindow, map: &Rc<RefCell<MapLibre>>) {
    loop {
        let map = map.borrow_mut();
        map.renderer().render_once();
        if map.style_loaded() {
            drop(map);
            style(map);
            break;
        } else if let Some(error) = map.style_loading_error() {
            panic!("Failed to load map: {}", error);
        }
    }

    ui.on_map_size_changed({
        let map = Rc::downgrade(map);
        move |size| {
            let size =
                maplibre_native::Size::new(Width(size.width as u32), Height(size.height as u32));
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
                    size.width(),
                    size.height(),
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
                .set_position(ScreenCoordinate::new(X(x.into()), Y(y.into())));
        }
    });

    ui.global::<MapAdapter>().on_mouse_move({
        let map = Rc::downgrade(map);
        move |x: f32, y: f32, _z: bool| {
            let p = ScreenCoordinate::new(X(x.into()), Y(y.into()));
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
            let pos = ScreenCoordinate::new(X(x.into()), Y(y.into()));
            let scale = if delta > 0. { STEP } else { 1.0 / STEP };
            map.upgrade().unwrap().borrow_mut().renderer().scale_by(scale, pos);
        }
    });
}

fn style(map: &Rc<RefCell<MapLibre>>) {
    let mut map_borrow = map.borrow_mut();
    let mut style = Style::get_ref(map_borrow.renderer());

    let image = ImageReader::open(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("ui").join("icons").join("Marker.png"),
    )
    .unwrap()
    .decode()
    .unwrap();
    let image_id = style.add_image("The id", &image, true);

    let mut geo_json_source = GeoJsonSource::new("geojsonsourceid");
    geo_json_source.set_point(Latitude(52.67655), Longitude(13.28387));
    let source_id = style.add_source(geo_json_source);

    let layer = SymbolLayer::new("Layer id", &source_id);
    layer.set_icon_image(&image_id);
    layer.set_icon_anchor(SymbolAnchorType::Bottom);
    style.add_layer(layer);
}
