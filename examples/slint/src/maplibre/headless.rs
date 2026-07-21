use std::cell::RefCell;
use std::num::NonZeroU32;
use std::path::Path;
use std::rc::Rc;

use maplibre_native::tile_server_options::TileServerOptions;
use maplibre_native::{
    CameraUpdate, Continuous, ImageRenderer, ImageRendererBuilder, LatLng, MapLoadError,
    ResourceOptions, ScreenCoordinate, Size,
};

#[derive(Default)]
struct Flags {
    loading_style_error: Option<MapLoadError>,
    style_loaded: bool,
}

pub struct MapLibre {
    flags: Rc<RefCell<Flags>>,
    renderer: Option<ImageRenderer<Continuous>>,
    last_pos: ScreenCoordinate,
    map_size: Size,
}

impl MapLibre {
    pub fn new(map_size: Size) -> Self {
        Self {
            renderer: None,
            flags: Rc::default(),
            last_pos: ScreenCoordinate::default(),
            map_size,
        }
    }

    /// Builds the renderer once the host's pixel ratio (scale factor) is known.
    pub fn build_renderer(&mut self, pixel_ratio: f32) {
        let resource_options = ResourceOptions::default()
            .with_tile_server_options(&TileServerOptions::default())
            // .with_api_key(api_key)
            .with_cache_path(
                Path::new(env!("CARGO_MANIFEST_DIR")).join("maplibre_database.sqlite"),
            );

        let mut builder = ImageRendererBuilder::new().with_pixel_ratio(pixel_ratio);
        if self.map_size.width > 0 && self.map_size.height > 0 {
            builder = builder.with_size(
                NonZeroU32::new(self.map_size.width).unwrap(),
                NonZeroU32::new(self.map_size.height).unwrap(),
            );
        }
        let mut renderer =
            builder.with_resource_options(resource_options).build_continuous_renderer();
        renderer.update_camera(
            &CameraUpdate::new()
                .center(LatLng { lat: 0.0, lng: 0.0 })
                .zoom(0.0)
                .bearing(0.0)
                .pitch(0.0),
        );

        let observer = renderer.map_observer();
        observer.set_did_finish_loading_style_callback({
            let flags = Rc::downgrade(&self.flags);
            move || {
                if let Some(flags) = flags.upgrade() {
                    flags.borrow_mut().style_loaded = true;
                }
            }
        });
        observer.set_did_fail_loading_map_callback({
            let flags = Rc::downgrade(&self.flags);
            move |error| {
                println!("Failed to load map: {}", error.message);
                if let Some(flags) = flags.upgrade() {
                    let mut flags = flags.borrow_mut();
                    flags.style_loaded = false;
                    flags.loading_style_error = Some(error);
                }
            }
        });

        self.renderer = Some(renderer);
    }

    pub fn style_loaded(&self) -> bool {
        self.flags.borrow().style_loaded
    }

    pub fn style_loading_error(&self) -> Option<MapLoadError> {
        self.flags.borrow().loading_style_error.clone()
    }

    pub(super) fn renderer(&mut self) -> &mut ImageRenderer<Continuous> {
        self.renderer.as_mut().expect("renderer is built during RenderingSetup")
    }

    pub fn set_position(&mut self, pos: ScreenCoordinate) {
        self.last_pos = pos;
    }

    pub fn position(&self) -> ScreenCoordinate {
        self.last_pos
    }

    pub fn set_map_size(&mut self, size: Size) {
        self.map_size = size;
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.set_map_size(size);
        }
    }

    // Inverse the map rotation logic from MapLibre's `Map::rotateBy` to convert a control+wheel delta into a synthetic drag gesture.
    pub fn rotate_by(&mut self, delta: f32) {
        let first = self.position();
        let mut center = ScreenCoordinate {
            x: f64::from(self.map_size.width) / 2.0,
            y: f64::from(self.map_size.height) / 2.0,
        };

        let offset = first - center;
        let distance = offset.x.hypot(offset.y);

        if distance < 200.0 {
            let height_offset = -200.0;
            let rotate_bearing = offset.y.atan2(offset.x);
            center = ScreenCoordinate {
                x: first.x + rotate_bearing.cos() * height_offset,
                y: first.y + rotate_bearing.sin() * height_offset,
            };
        }

        let relative = first - center;
        let angle = f64::from(delta).to_radians();
        let second = ScreenCoordinate {
            x: center.x + relative.x * angle.cos() - relative.y * angle.sin(),
            y: center.y + relative.x * angle.sin() + relative.y * angle.cos(),
        };

        self.renderer().rotate_by(first, second);
    }
}

pub fn create_map(size: Size) -> Rc<RefCell<MapLibre>> {
    let map_size =
        if size.width > 0 && size.height > 0 { size } else { Size { width: 0, height: 0 } };
    Rc::new(RefCell::new(MapLibre::new(map_size)))
}
