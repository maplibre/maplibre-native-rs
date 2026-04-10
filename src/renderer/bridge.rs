use crate::renderer::callbacks::{
    camera_did_change_callback, failing_loading_map_callback, finish_rendering_frame_callback,
    void_callback, CameraDidChangeCallback, FailingLoadingMapCallback,
    FinishRenderingFrameCallback, VoidCallback,
};
use cxx;
use std::ops::Sub;

// https://maplibre.org/maplibre-native/docs/book/design/ten-thousand-foot-view.html

/// Enable or disable the internal logging thread
///
/// By default, logs are generated asynchronously except for Error level messages.
/// In crash scenarios, pending async log entries may be lost.
pub fn set_log_thread_enabled(enable: bool) {
    ffi::Log_useLogThread(enable);
}

fn log_from_cpp(severity: ffi::EventSeverity, event: ffi::Event, code: i64, message: &str) {
    #[cfg(feature = "log")]
    match severity {
        ffi::EventSeverity::Debug => log::debug!("{event:?} (code={code}) {message}"),
        ffi::EventSeverity::Info => log::info!("{event:?} (code={code}) {message}"),
        ffi::EventSeverity::Warning => log::warn!("{event:?} (code={code}) {message}"),
        ffi::EventSeverity::Error => log::error!("{event:?} (code={code}) {message}"),
        ffi::EventSeverity { repr } => {
            log::error!("{event:?} (severity={repr}, code={code}) {message}");
        }
    }
}

/// An x value
#[derive(Debug)]
pub struct X(pub f64);

/// An y value
#[derive(Debug)]
pub struct Y(pub f64);

/// A width value
#[derive(Debug)]
pub struct Width(pub u32);

/// A height value
#[derive(Debug)]
pub struct Height(pub u32);

/// A position in screen coordinates
#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct ScreenCoordinate {
    x: f64,
    y: f64,
}

impl ScreenCoordinate {
    /// Create a new `ScreenCoordinate` object
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(x: X, y: Y) -> Self {
        Self { x: x.0, y: y.0 }
    }
}

impl Sub for ScreenCoordinate {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

/// A size
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Size {
    width: u32,
    heigth: u32,
}

impl Size {
    /// Create a new size object
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(width: Width, height: Height) -> Self {
        Self {
            width: width.0,
            heigth: height.0,
        }
    }

    /// get width
    #[must_use]
    pub fn width(self) -> u32 {
        self.width
    }

    /// get height
    #[must_use]
    pub fn height(self) -> u32 {
        self.heigth
    }
}

#[cxx::bridge()]
pub mod sources {
    #[namespace = "mbgl::style"]
    extern "C++" {
        include!("mbgl/style/sources/geojson_source.hpp");
        // Opaque types
        type GeoJSONSource;
    }

    #[namespace = "mln::bridge::style::sources::geojson"]
    unsafe extern "C++" {
        include!("sources/sources.h");

        fn create(id: &str) -> UniquePtr<GeoJSONSource>;
        fn setPoint(source: &UniquePtr<GeoJSONSource>, latitude: f64, longitude: f64);
    }
}

#[cxx::bridge()]
pub mod layers {
    #[namespace = "mbgl::style"]
    extern "C++" {
        include!("mbgl/style/layers/symbol_layer.hpp");
        // Opaque types
        type SymbolLayer;
    }

    #[namespace = "mln::bridge::style::layers"]
    unsafe extern "C++" {
        include!("layers/layers.h");

        fn create_symbol_layer(layer_id: &str, source_id: &str) -> UniquePtr<SymbolLayer>;
        fn setIconImage(layer: &UniquePtr<SymbolLayer>, image_id: &str);
    }
}

#[cxx::bridge()]
pub mod resource_options {

    #[namespace = "mbgl"]
    extern "C++" {
        // Opaque types
        type ResourceOptions;

        // The name must be unique but for some reason this is required
        #[rust_name = "CxxTileServerOptions"]
        type TileServerOptions = super::tile_server_options::TileServerOptions;
    }

    #[namespace = "mln::bridge::resource_options"]
    unsafe extern "C++" {
        include!("resource_options.h");

        #[rust_name = "new"]
        fn new_() -> UniquePtr<ResourceOptions>;

        fn withApiKey(obj: Pin<&mut ResourceOptions>, key: &str);
        fn withAssetPath(obj: Pin<&mut ResourceOptions>, path: &[u8]);
        fn withCachePath(obj: Pin<&mut ResourceOptions>, path: &[u8]);

        fn withMaximumCacheSize(obj: Pin<&mut ResourceOptions>, max_cache_size: u64);
        fn withTileServerOptions(
            obj: Pin<&mut ResourceOptions>,
            tile_server_options: UniquePtr<CxxTileServerOptions>,
        );
    }
}

#[cxx::bridge()]
pub mod tile_server_options {
    #[namespace = "mbgl"]
    extern "C++" {
        // Opaque types
        type TileServerOptions;
    }

    #[namespace = "mln::bridge::tile_server_options"]
    unsafe extern "C++" {
        include!("tile_server_options.h");

        #[rust_name = "new_tile_server_options"]
        fn new_() -> UniquePtr<TileServerOptions>;

        fn withBaseUrl(obj: Pin<&mut TileServerOptions>, path: &[u8]);
        fn withUriSchemeAlias(obj: Pin<&mut TileServerOptions>, path: &[u8]);
        fn withSourceTemplate(
            obj: Pin<&mut TileServerOptions>,
            styleTemplate: &[u8],
            domainName: &[u8],
            versionPrefix: &[u8],
        );
        fn withSpritesTemplate(
            obj: Pin<&mut TileServerOptions>,
            spritesTemplate: &[u8],
            domainName: &[u8],
            versionPrefix: &[u8],
        );
        fn withGlyphsTemplate(
            obj: Pin<&mut TileServerOptions>,
            glyphsTemplate: &[u8],
            domainName: &[u8],
            versionPrefix: &[u8],
        );
        fn withTileTemplate(
            obj: Pin<&mut TileServerOptions>,
            tileTemplate: &[u8],
            domainName: &[u8],
            versionPrefix: &[u8],
        );
        fn withApiKeyParameterName(obj: Pin<&mut TileServerOptions>, apiKeyParameterName: &[u8]);
        fn setRequiresApiKey(obj: Pin<&mut TileServerOptions>, apiKeyRequired: bool);
    }
}

#[allow(clippy::borrow_as_ptr)]
#[cxx::bridge(namespace = "mln::bridge")]
pub mod map_observer {

    #[namespace = "mln::bridge"]
    #[repr(u32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    // Map Load error
    enum MapObserverCameraChangeMode {
        Immediate,
        Animated,
    }

    #[namespace = "mbgl"]
    #[repr(u32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    // Map Load error
    enum MapLoadError {
        StyleParseError,
        StyleLoadError,
        NotFoundError,
        UnknownError,
    }

    #[namespace = "mbgl"]
    extern "C++" {
        include!("mbgl/map/mode.hpp");
        type MapLoadError;
    }

    // Declarations for C++ with implementations in Rust
    extern "Rust" {
        type VoidCallback;
        type FinishRenderingFrameCallback;
        type CameraDidChangeCallback;
        type FailingLoadingMapCallback;

        fn void_callback(callback: &VoidCallback);
        fn finish_rendering_frame_callback(
            callback: &FinishRenderingFrameCallback,
            needsRepaint: bool,
            placementChanged: bool,
        );
        fn camera_did_change_callback(
            callback: &CameraDidChangeCallback,
            mode: MapObserverCameraChangeMode,
        );
        fn failing_loading_map_callback(
            callback: &FailingLoadingMapCallback,
            error: MapLoadError,
            what: &str,
        );
    }

    // Declarations for Rust with implementations in C++
    extern "C++" {
        include!("map_observer.h"); // Required to find functions below

        type MapObserverCameraChangeMode;

        // C++ Opaque types
        #[rust_name = "CxxMapObserver"]
        type MapObserver = super::ffi::MapObserver; // Created custom map observer
    }

    unsafe extern "C++" {
        // With `self: Pin<&mut MapObserver>` as first argument, it is a non static method of that object.
        // cxx searches for such a method
        fn setWillStartLoadingMapCallback(self: &CxxMapObserver, callback: Box<VoidCallback>);
        fn setFinishLoadingStyleCallback(self: &CxxMapObserver, callback: Box<VoidCallback>);
        fn setBecomeIdleCallback(self: &CxxMapObserver, callback: Box<VoidCallback>);
        fn setFailLoadingMapCallback(
            self: &CxxMapObserver,
            callback: Box<FailingLoadingMapCallback>,
        );
        fn setFinishRenderingFrameCallback(
            self: &CxxMapObserver,
            callback: Box<FinishRenderingFrameCallback>,
        );
        fn setCameraDidChangeCallback(
            self: &CxxMapObserver,
            callback: Box<CameraDidChangeCallback>,
        );
    }
}

#[allow(clippy::borrow_as_ptr)]
#[cxx::bridge(namespace = "mln::bridge")]
pub mod ffi {
    // CXX validates enum types against the C++ definition during compilation

    // The mbgl enums must be defined in the same namespace than on the C++ side
    #[namespace = "mbgl"]
    #[repr(u32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// Map rendering mode configuration.
    enum MapMode {
        /// Continually updating map
        Continuous,
        /// Once-off still image of an arbitrary viewport
        Static,
        /// Once-off still image of a single tile
        Tile,
    }

    #[namespace = "mbgl"]
    #[repr(u32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// Debug visualization options for map rendering.
    enum MapDebugOptions {
        /// No debug visualization.
        NoDebug = 0,
        /// Edges of tile boundaries are shown as thick, red lines.
        ///
        /// Can help diagnose tile clipping issues.
        TileBorders = 0b0000_0010, // 1 << 1
        /// Shows tile parsing status information.
        ParseStatus = 0b0000_0100, // 1 << 2
        /// Each tile shows a timestamp indicating when it was loaded.
        Timestamps = 0b0000_1000, // 1 << 3
        /// Edges of glyphs and symbols are shown as faint, green lines.
        ///
        /// Can help diagnose collision and label placement issues.
        Collision = 0b0001_0000, // 1 << 4
        /// Each drawing operation is replaced by a translucent fill.
        ///
        /// Overlapping drawing operations appear more prominent to help diagnose overdrawing.
        Overdraw = 0b0010_0000, // 1 << 5
        /// The stencil buffer is shown instead of the color buffer.
        ///
        /// Note: This option does nothing in Release builds of the SDK.
        StencilClip = 0b0100_0000, // 1 << 6
        /// The depth buffer is shown instead of the color buffer.
        ///
        /// Note: This option does nothing in Release builds of the SDK
        DepthBuffer = 0b1000_0000, // 1 << 7
    }

    /// MapLibre Native Event Severity levels
    #[namespace = "mbgl"]
    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum EventSeverity {
        Debug = 0,
        Info = 1,
        Warning = 2,
        Error = 3,
    }

    /// MapLibre Native Event types
    #[namespace = "mbgl"]
    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Event {
        General = 0,
        Setup = 1,
        Shader = 2,
        ParseStyle = 3,
        ParseTile = 4,
        Render = 5,
        Style = 6,
        Database = 7,
        HttpRequest = 8,
        Sprite = 9,
        Image = 10,
        OpenGL = 11,
        JNI = 12,
        Android = 13,
        Crash = 14,
        Glyph = 15,
        Timing = 16,
    }

    #[namespace = "mbgl"]
    extern "C++" {
        include!("mbgl/map/mode.hpp");
        include!("mbgl/map/map_observer.hpp");
        include!("mbgl/util/geo.hpp");

        type MapMode;
        type MapDebugOptions;
        // The name must be unique but for some reason this is required
        #[rust_name = "CxxResourceOptions"]
        type ResourceOptions = super::resource_options::ResourceOptions;
        pub type EventSeverity;
        pub type Event;
    }

    #[namespace = "mbgl"]
    extern "C++" {
        type ScreenCoordinate = super::ScreenCoordinate;
        type Size = super::Size;
    }

    #[namespace = "mbgl::style"]
    extern "C++" {
        #[rust_name = "CxxGeoJSONSource"]
        type GeoJSONSource = super::sources::GeoJSONSource;
        #[rust_name = "CxxSymbolLayer"]
        type SymbolLayer = super::layers::SymbolLayer;
    }

    // Declarations for Rust with implementations in C++
    unsafe extern "C++" {
        include!("map_renderer.h");

        // C++ Opaque types
        type BridgeImage;
        type MapObserver; // Created custom map observer
        type MapRenderer;
        // Left side must match a type in C++! Right side must be defined in Rust
        // example: type VoidCallback = super::VoidTrVoidCallbackampoline;

        #[allow(clippy::too_many_arguments)]
        fn MapRenderer_new(
            mapMode: MapMode,
            width: u32,
            height: u32,
            pixelRatio: f32,
            resource_options: UniquePtr<CxxResourceOptions>,
        ) -> UniquePtr<MapRenderer>;
        fn readStillImage(self: Pin<&mut MapRenderer>) -> UniquePtr<BridgeImage>;
        fn get(self: &BridgeImage) -> *const u8;
        fn size(self: &BridgeImage) -> Size;
        fn bufferLength(self: &BridgeImage) -> usize;
        fn render_once(self: Pin<&mut MapRenderer>);
        fn render(self: Pin<&mut MapRenderer>) -> UniquePtr<CxxString>;
        fn setDebugFlags(self: Pin<&mut MapRenderer>, flags: MapDebugOptions);
        fn setCamera(
            self: Pin<&mut MapRenderer>,
            lat: f64,
            lon: f64,
            zoom: f64,
            bearing: f64,
            pitch: f64,
        );
        fn moveBy(self: Pin<&mut MapRenderer>, delta: &ScreenCoordinate);
        fn scaleBy(self: Pin<&mut MapRenderer>, scale: f64, pos: &ScreenCoordinate);
        fn style_load_from_url(self: Pin<&mut MapRenderer>, url: &str);
        fn setSize(self: Pin<&mut MapRenderer>, size: &Size);
        fn observer(self: Pin<&mut MapRenderer>) -> SharedPtr<MapObserver>;
        fn style_add_image(
            self: Pin<&mut MapRenderer>,
            id: &str,
            data: &[u8],
            size: Size,
            single_distance_field: bool,
        );
        fn style_remove_image(self: Pin<&mut MapRenderer>, id: &str);
        fn style_add_geojson_source(
            self: Pin<&mut MapRenderer>,
            source: UniquePtr<CxxGeoJSONSource>,
        );
        fn style_add_symbol_layer(self: Pin<&mut MapRenderer>, layer: UniquePtr<CxxSymbolLayer>);
    }

    // Declarations for C++ with implementations in Rust
    extern "Rust" {
        /// Bridge logging from C++ to Rust log crate
        fn log_from_cpp(severity: EventSeverity, event: Event, code: i64, message: &str);
    }

    unsafe extern "C++" {
        include!("rust_log_observer.h");

        fn Log_useLogThread(enable: bool);
    }
}

unsafe impl cxx::ExternType for Size {
    type Id = cxx::type_id!("mbgl::Size");
    type Kind = cxx::kind::Trivial;
}

unsafe impl cxx::ExternType for ScreenCoordinate {
    type Id = cxx::type_id!("mbgl::ScreenCoordinate");
    type Kind = cxx::kind::Trivial;
}

#[cfg(test)]
mod test {
    use crate::{ScreenCoordinate, X, Y};

    #[test]
    fn screen_corrdinate_diff() {
        let s1 = ScreenCoordinate::new(X(5.), Y(-1.));
        let s2 = ScreenCoordinate::new(X(3.), Y(-10.));

        let res = s1 - s2;
        assert!((res.x - 2.).abs() < 0.00001);
        assert!((res.y - 9.).abs() < 0.00001);
    }
}
