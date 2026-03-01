use maplibre_native::Continuous;
use maplibre_native::ImageRenderer;
use maplibre_native::ImageRendererBuilder;
use maplibre_native::ScreenCoordinate;
use std::cell::RefCell;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Arc;
use crate::Size;

#[derive(Default)]
struct Flags {
    style_loaded: bool,
    map_idle: bool,
    frame_updated: bool,  
}

pub struct MapLibre {
    flags: Arc<RefCell<Flags>>,
    renderer: ImageRenderer<Continuous>,
    last_pos: ScreenCoordinate,
}

impl MapLibre {
    pub fn new(renderer: ImageRenderer<Continuous>) -> Self {
        Self {
            renderer,
            flags: Arc::default(),
            last_pos: ScreenCoordinate::default(),
        }
    }

    pub fn updated(&mut self) -> bool {
        let updated = self.flags.borrow().frame_updated;
        self.flags.borrow_mut().frame_updated = false;
        updated
    }

    pub fn renderer<'a>(&'a mut self) -> &'a mut ImageRenderer<Continuous> {
        &mut self.renderer
    }

    pub fn set_position(&mut self, pos: ScreenCoordinate) {
        self.last_pos = pos;
    }

    pub fn position(&self) -> ScreenCoordinate {
        self.last_pos
    }
}

pub fn create_map(size: Size) -> Arc<RefCell<MapLibre>> {
    let mut renderer = ImageRendererBuilder::new()
        .with_size(NonZeroU32::new(size.width as u32).unwrap(), NonZeroU32::new(size.height as u32).unwrap())
        .with_pixel_ratio(1.0)
        .with_cache_path(Path::new(env!("CARGO_MANIFEST_DIR")).join("maplibre_database.sqlite"))
        .build_continuous_renderer();
    renderer.set_camera(0, 0, 0, 0., 0.); // setting the camera is important, otherwise map libre does nothing (no logs are comming and no map gets generated)
    renderer.load_style_from_url(&"https://demotiles.maplibre.org/style.json".parse().unwrap());
    let map = Arc::new(RefCell::new(MapLibre::new(renderer)));

    let mut map_observer = map.borrow_mut().renderer().map_observer();
    map_observer.set_did_become_idle_callback({
        let flags = Arc::downgrade(&map.borrow().flags);
        move || {
            println!("set_on_did_become_idle_callback");
            flags.upgrade().inspect(|v| {
                v.borrow_mut().map_idle = true;
            });
        }
    });
    map_observer.set_will_start_loading_map_callback({
        let flags = Arc::downgrade(&map.borrow().flags);
        move || {
            println!("set_on_will_start_loading_map_callback");
            flags.upgrade().inspect(|v| {
                v.borrow_mut().map_idle = false;
                v.borrow_mut().style_loaded = false;
            });
        }
    });
    map_observer.set_did_finish_loading_style_callback({
        let flags = Arc::downgrade(&map.borrow().flags);
        move || {
            println!("set_on_did_finish_loading_style_callback");
            flags.upgrade().inspect(|v| {
                v.borrow_mut().style_loaded = true;
            });
        }
    });
    map_observer.set_did_fail_loading_map_callback({
        let flags = Arc::downgrade(&map.borrow().flags);
        move |_error, what| {
            println!("Failed to load map: {what}");
            flags.upgrade().inspect(|v| {
                v.borrow_mut().style_loaded = false;
            });
        }
    });
    map_observer.set_camera_changed_callback(|mode| {});
    map_observer.set_finish_rendering_frame_callback({
        let flags = Arc::downgrade(&map.borrow().flags);
        move |needs_repaint: bool, placement_changed: bool| {
            if needs_repaint || placement_changed {
                flags.upgrade().inspect(|v| {
                    println!("Frame updated");
                    v.borrow_mut().frame_updated = true;
                });
            }
        }
    });
    
    map
}
