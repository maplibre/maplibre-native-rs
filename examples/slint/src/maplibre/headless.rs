use crate::Size;
use maplibre_native::Continuous;
use maplibre_native::ImageRenderer;
use maplibre_native::ImageRendererBuilder;
use maplibre_native::MapLoadError;
use maplibre_native::ResourceOptions;
use maplibre_native::ScreenCoordinate;
use maplibre_native::Size as MapSize;
use maplibre_native::tile_server_options::TileServerOptions;
use maplibre_native::{Latitude, Longitude};
use maplibre_native::{X, Y};
use std::cell::RefCell;
use std::num::NonZeroU32;
use std::path::Path;
use std::rc::Rc;

#[derive(Default)]
struct Flags {
    loading_style_error: Option<MapLoadError>,
    style_loaded: bool,
    map_idle: bool,
    frame_updated: bool,
}

pub struct MapLibre {
    flags: Rc<RefCell<Flags>>,
    renderer: ImageRenderer<Continuous>,
    last_pos: ScreenCoordinate,
    map_size: MapSize,
}

impl MapLibre {
    pub fn new(renderer: ImageRenderer<Continuous>, map_size: MapSize) -> Self {
        Self { renderer, flags: Rc::default(), last_pos: ScreenCoordinate::default(), map_size }
    }

    pub fn style_loaded(&self) -> bool {
        self.flags.borrow().style_loaded
    }

    pub fn style_loading_error(&self) -> Option<MapLoadError> {
        self.flags.borrow().loading_style_error
    }

    pub fn updated(&mut self) -> bool {
        let updated = self.flags.borrow().frame_updated;
        self.flags.borrow_mut().frame_updated = false;
        updated
    }

    pub fn renderer(&mut self) -> &mut ImageRenderer<Continuous> {
        &mut self.renderer
    }

    pub fn set_position(&mut self, pos: ScreenCoordinate) {
        self.last_pos = pos;
    }

    pub fn position(&self) -> ScreenCoordinate {
        self.last_pos
    }

    pub fn set_map_size(&mut self, size: MapSize) {
        self.map_size = size;
        self.renderer.set_map_size(size);
    }

    // Invers the map rotation logic from MapLibre's `Map::rotateBy` to convert a control+wheel delta into a synthetic drag gesture.
    pub fn rotate_by(&mut self, delta: f32) {
        let first = self.position();
        let mut center = ScreenCoordinate::new(
            X(f64::from(self.map_size.width()) / 2.0),
            Y(f64::from(self.map_size.height()) / 2.0),
        );

        let offset = first - center;
        let distance = offset.x().hypot(offset.y());

        if distance < 200.0 {
            let height_offset = -200.0;
            let rotate_bearing = offset.y().atan2(offset.x());
            center = ScreenCoordinate::new(
                X(first.x() + rotate_bearing.cos() * height_offset),
                Y(first.y() + rotate_bearing.sin() * height_offset),
            );
        }

        let relative = first - center;
        let angle = f64::from(delta).to_radians();
        let second = ScreenCoordinate::new(
            X(center.x() + relative.x() * angle.cos() - relative.y() * angle.sin()),
            Y(center.y() + relative.x() * angle.sin() + relative.y() * angle.cos()),
        );

        self.renderer.rotate_by(first, second);
    }
}

pub fn create_map(size: Size) -> Rc<RefCell<MapLibre>> {
    let resource_options = ResourceOptions::default()
        .with_tile_server_options(&TileServerOptions::default())
        // .with_api_key(api_key)
        .with_cache_path(Path::new(env!("CARGO_MANIFEST_DIR")).join("maplibre_database.sqlite"));

    let mut renderer = ImageRendererBuilder::new();
    let mut map_size = MapSize::new(maplibre_native::Width(0), maplibre_native::Height(0));
    if size.width > 0. && size.height > 0. {
        map_size = MapSize::new(
            maplibre_native::Width(size.width as u32),
            maplibre_native::Height(size.height as u32),
        );
        renderer = renderer.with_size(
            NonZeroU32::new(size.width as u32).unwrap(),
            NonZeroU32::new(size.height as u32).unwrap(),
        );
    }
    let mut renderer = renderer
        .with_pixel_ratio(1.0)
        .with_resource_options(resource_options)
        .build_continuous_renderer();
    renderer.set_camera(Latitude(0.0), Longitude(0.0), 0.0, 0.0, 0.0); // setting the camera is important, otherwise map libre does nothing (no logs are comming and no map gets generated)

    let map = Rc::new(RefCell::new(MapLibre::new(renderer, map_size)));

    let map_observer = map.borrow_mut().renderer().map_observer();
    map_observer.set_did_become_idle_callback({
        let flags = Rc::downgrade(&map.borrow().flags);
        move || {
            flags.upgrade().inspect(|v| {
                v.borrow_mut().map_idle = true;
            });
        }
    });
    map_observer.set_will_start_loading_map_callback({
        let flags = Rc::downgrade(&map.borrow().flags);
        move || {
            println!("set_on_will_start_loading_map_callback");
            flags.upgrade().inspect(|v| {
                v.borrow_mut().map_idle = false;
                v.borrow_mut().style_loaded = false;
            });
        }
    });
    map_observer.set_did_finish_loading_style_callback({
        let flags = Rc::downgrade(&map.borrow().flags);
        move || {
            println!("set_on_did_finish_loading_style_callback");
            flags.upgrade().inspect(|v| {
                v.borrow_mut().style_loaded = true;
            });
        }
    });
    map_observer.set_did_fail_loading_map_callback({
        let flags = Rc::downgrade(&map.borrow().flags);
        move |error, what| {
            println!("Failed to load map: {what}");
            flags.upgrade().inspect(|v| {
                let mut borrow = v.borrow_mut();
                borrow.style_loaded = false;
                borrow.loading_style_error = Some(error);
            });
        }
    });
    map_observer.set_camera_changed_callback(|_mode| {});
    map_observer.set_finish_rendering_frame_callback({
        let flags = Rc::downgrade(&map.borrow().flags);
        move |needs_repaint: bool, placement_changed: bool| {
            if needs_repaint || placement_changed {
                flags.upgrade().inspect(|v| {
                    v.borrow_mut().frame_updated = true;
                });
            }
        }
    });

    map
}
