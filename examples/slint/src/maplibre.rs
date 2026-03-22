use crate::MainWindow;
use crate::MapAdapter;
use maplibre_native::Height;
use maplibre_native::ScreenCoordinate;
use maplibre_native::Width;
use slint::ComponentHandle;
use std::sync::Arc;
mod headless;
use headless::MapLibre;
pub use headless::create_map;
use maplibre_native::{X, Y};
use std::cell::RefCell;

pub fn init(ui: &MainWindow, map: &Arc<RefCell<MapLibre>>) {
    ui.on_map_size_changed({
        let map = Arc::downgrade(map);
        move |size| {
            let size =
                maplibre_native::Size::new(Width(size.width as u32), Height(size.height as u32));
            map.upgrade()
                .unwrap()
                .borrow_mut()
                .renderer()
                .set_map_size(size);
        }
    });

    ui.global::<MapAdapter>().on_tick_map_loop({
        let map = Arc::downgrade(map);
        let ui_handle = ui.as_weak();
        move || {
            let map = map.upgrade().unwrap();
            let mut map = map.borrow_mut();
            map.renderer().render_once();
            if map.updated() {
                let image = map.renderer().get_texture();
                // let image = map.renderer().read_still_image();
                // let size = image.size();
                // let img = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
                //     image.buffer(),
                //     size.width(),
                //     size.height(),
                // );
                // println!("New image: ({}, {})", size.width(), size.height());
                if let Ok(image) = image.try_into() {
                    ui_handle
                        .upgrade()
                        .unwrap()
                        .global::<MapAdapter>()
                        .set_map_texture(image); // TODO: check if the image really changed, otherwise we don't need to clone!
                }
            }
        }
    });

    ui.global::<MapAdapter>().on_mouse_press({
        let map = Arc::downgrade(map);
        move |x: f32, y: f32| {
            map.upgrade()
                .unwrap()
                .borrow_mut()
                .set_position(ScreenCoordinate::new(X(x.into()), Y(y.into())));
        }
    });

    ui.global::<MapAdapter>().on_mouse_move({
        let map = Arc::downgrade(map);
        move |x: f32, y: f32, _z: bool| {
            println!("Mouse move");
            let p = ScreenCoordinate::new(X(x.into()), Y(y.into()));
            let map = map.upgrade().unwrap();
            let mut map = map.borrow_mut();
            let delta = p - map.position();
            map.renderer().move_by(delta);
            map.set_position(p);
        }
    });

    ui.global::<MapAdapter>().on_wheel_zoom({
        let map = Arc::downgrade(map);
        move |x: f32, y: f32, delta: f32| {
            const STEP: f64 = 1.2;
            let pos = ScreenCoordinate::new(X(x.into()), Y(y.into()));
            let scale = if delta > 0. { STEP } else { 1.0 / STEP };
            map.upgrade()
                .unwrap()
                .borrow_mut()
                .renderer()
                .scale_by(scale, pos);
        }
    });
}
