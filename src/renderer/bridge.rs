use crate::renderer::callbacks::{
    camera_did_change_callback, failing_loading_map_callback, finish_rendering_frame_callback,
    void_callback, CameraDidChangeCallback, FailingLoadingMapCallback,
    FinishRenderingFrameCallback, VoidCallback,
};
use cxx::{SharedPtr, UniquePtr};
use std::fmt::Display;
use std::ops::Sub;
use std::sync::Arc;

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
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
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
        Self { width: width.0, heigth: height.0 }
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

/// FFI bindings for map source operations.
///
/// This module provides C++/Rust interoperability for various source types.
/// Currently supports GeoJSON sources, with extensibility for additional source types.
#[cxx::bridge()]
pub mod sources {
    #[namespace = "mbgl::style"]
    extern "C++" {
        include!("mbgl/style/sources/geojson_source.hpp");
        // Opaque types
        /// A GeoJSON source for MapLibre rendering.
        type GeoJSONSource;
    }

    #[namespace = "mln::bridge::style::sources::geojson"]
    unsafe extern "C++" {
        include!("sources/sources.h");

        /// Creates a new GeoJSON source with the given ID.
        fn create(id: &str) -> UniquePtr<GeoJSONSource>;
        /// Sets a point for the GeoJSON source.
        fn setPoint(source: &UniquePtr<GeoJSONSource>, latitude: f64, longitude: f64);
    }
}

impl std::fmt::Debug for sources::GeoJSONSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeoJSONSource").finish()
    }
}

#[cxx::bridge()]
/// FFI bindings for map layer operations.
///
/// This module provides C++/Rust interoperability for various layer types.
/// Currently supports symbol layers, with extensibility for additional layer types like fill, line, and background layers.
pub mod layers {
    // Must have the same namespace than on the C++ side
    #[namespace = "mbgl::style"]
    /// Symbol anchor position type.
    pub enum SymbolAnchorType {
        /// Center anchor point.
        Center,
        /// Left anchor point.
        Left,
        /// Right anchor point.
        Right,
        /// Top anchor point.
        Top,
        /// Bottom anchor point.
        Bottom,
        /// Top-left anchor point.
        TopLeft,
        /// Top-right anchor point.
        TopRight,
        /// Bottom-left anchor point.
        BottomLeft,
        /// Bottom-right anchor point.
        BottomRight,
    }

    #[namespace = "mbgl::style"]
    extern "C++" {
        include!("mbgl/style/layers/symbol_layer.hpp");
        include!("mbgl/style/types.hpp");
        // Opaque types
        /// A symbol layer for rendering labels and icons on the map.
        type SymbolLayer;

        /// Symbol anchor position type.
        type SymbolAnchorType;
    }

    #[namespace = "mln::bridge::style::layers"]
    unsafe extern "C++" {
        include!("layers/layers.h");

        /// Creates a new symbol layer.
        #[must_use]
        pub(crate) fn create_symbol_layer(
            layer_id: &str,
            source_id: &str,
        ) -> UniquePtr<SymbolLayer>;
        /// Sets the icon image for a layer by image ID.
        fn setIconImage(layer: &UniquePtr<SymbolLayer>, image_id: &str);
        /// Sets the anchor point for layer icons.
        fn setIconAnchor(layer: &UniquePtr<SymbolLayer>, anchor: SymbolAnchorType);
    }
}

impl std::fmt::Debug for layers::SymbolLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SymbolLayer").finish()
    }
}

impl std::fmt::Debug for layers::SymbolAnchorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SymbolAnchorType").finish()
    }
}

#[cxx::bridge()]
/// Resource and configuration options for MapLibre.
pub mod resource_options {

    #[namespace = "mbgl"]
    extern "C++" {
        // Opaque types
        /// Resource configuration options.
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
            tile_server_options: &CxxTileServerOptions,
        );
    }
}

impl std::fmt::Debug for resource_options::ResourceOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceOptions").finish()
    }
}

#[cxx::bridge()]
/// Tile server configuration options.
pub mod tile_server_options {
    #[namespace = "mbgl"]
    extern "C++" {
        // Opaque types
        /// Tile server configuration.
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

impl std::fmt::Debug for tile_server_options::TileServerOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TileServerOptions").finish()
    }
}

impl Display for map_observer::MapLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match *self {
            Self::StyleParseError => "Failed parsing style",
            Self::StyleLoadError => "Failed loading style",
            Self::NotFoundError => "Style not found",
            Self::UnknownError => "Unknown error",
            _ => "Unrecognized error",
        };
        write!(f, "{s}")
    }
}

#[allow(clippy::borrow_as_ptr)]
#[cxx::bridge(namespace = "mln::bridge")]
/// Map observer callbacks and related types.
pub mod map_observer {
    #[namespace = "mln::bridge"]
    #[repr(u32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// Camera change mode for map observer callbacks.
    pub enum MapObserverCameraChangeMode {
        /// Camera changed immediately without animation.
        Immediate,
        /// Camera changed using an animated transition.
        Animated,
    }

    #[namespace = "mbgl"]
    #[repr(u32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// Map loading error types.
    pub enum MapLoadError {
        /// Style parsing error.
        StyleParseError,
        /// Style loading error.
        StyleLoadError,
        /// Resource not found.
        NotFoundError,
        /// Unknown error.
        UnknownError,
    }

    #[namespace = "mbgl"]
    extern "C++" {
        include!("mbgl/map/map_observer.hpp");
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
        /// Sets the callback for when loading of the map will start.
        fn setWillStartLoadingMapCallback(self: &CxxMapObserver, callback: Box<VoidCallback>);
        /// Sets the callback for when the style has finished loading.
        fn setFinishLoadingStyleCallback(self: &CxxMapObserver, callback: Box<VoidCallback>);
        /// Sets the callback for when the map becomes idle.
        fn setBecomeIdleCallback(self: &CxxMapObserver, callback: Box<VoidCallback>);
        /// Sets the callback for when loading of the map fails.
        fn setFailLoadingMapCallback(
            self: &CxxMapObserver,
            callback: Box<FailingLoadingMapCallback>,
        );
        /// Sets the callback for when a frame finishes rendering.
        fn setFinishRenderingFrameCallback(
            self: &CxxMapObserver,
            callback: Box<FinishRenderingFrameCallback>,
        );
        /// Sets the callback for when the camera finishes changing.
        fn setCameraDidChangeCallback(
            self: &CxxMapObserver,
            callback: Box<CameraDidChangeCallback>,
        );
    }
}

#[allow(clippy::borrow_as_ptr)]
#[cxx::bridge(namespace = "mln::bridge")]
/// Core FFI definitions and types for the MapLibre bridge.
pub mod ffi {
    // CXX validates enum types against the C++ definition during compilation

    // The mbgl enums must be defined in the same namespace than on the C++ side
    #[namespace = "mbgl"]
    #[repr(u32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// Map rendering mode configuration.
    pub enum MapMode {
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
    pub enum MapDebugOptions {
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
        /// Debug severity level.
        Debug = 0,
        /// Info severity level.
        Info = 1,
        /// Warning severity level.
        Warning = 2,
        /// Error severity level.
        Error = 3,
    }

    /// MapLibre Native Event types
    #[namespace = "mbgl"]
    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Event {
        /// General event.
        General = 0,
        /// Setup event.
        Setup = 1,
        /// Shader event.
        Shader = 2,
        /// Style parsing event.
        ParseStyle = 3,
        /// Tile parsing event.
        ParseTile = 4,
        /// Render event.
        Render = 5,
        /// Style event.
        Style = 6,
        /// Database event.
        Database = 7,
        /// HTTP request event.
        HttpRequest = 8,
        /// Sprite event.
        Sprite = 9,
        /// Image event.
        Image = 10,
        /// OpenGL event.
        OpenGL = 11,
        /// JNI event.
        JNI = 12,
        /// Android event.
        Android = 13,
        /// Crash event.
        Crash = 14,
        /// Glyph event.
        Glyph = 15,
        /// Timing event.
        Timing = 16,
    }

    #[namespace = "mbgl"]
    extern "C++" {
        include!("mbgl/map/mode.hpp");

        type MapMode;
        type MapDebugOptions;
        // The name must be unique but for some reason this is required
        /// Resource configuration options.
        #[rust_name = "CxxResourceOptions"]
        type ResourceOptions = super::resource_options::ResourceOptions;
        /// Event severity enumeration.
        pub type EventSeverity;
        /// Event type enumeration.
        pub type Event;
    }

    #[namespace = "mbgl"]
    extern "C++" {
        /// Screen coordinate type.
        type ScreenCoordinate = super::ScreenCoordinate;
        /// Size type.
        type Size = super::Size;
    }

    #[namespace = "mbgl::style"]
    extern "C++" {
        /// GeoJSON source opaque type.
        #[rust_name = "CxxGeoJSONSource"]
        type GeoJSONSource = super::sources::GeoJSONSource;
        /// Symbol layer opaque type.
        #[rust_name = "CxxSymbolLayer"]
        type SymbolLayer = super::layers::SymbolLayer;
    }

    #[namespace = "mbgl::webgpu"]
    extern "C++" {
        #[cfg(feature = "wgpu")]
        type Texture2D;
    }

    #[namespace = ""]
    extern "C++" {
        #[cfg(feature = "wgpu")]
        type WGPUDevice = binding_generator::WGPUDeviceWrapper;
        #[cfg(feature = "wgpu")]
        type WGPUQueue = binding_generator::WGPUQueueWrapper;
    }

    // Declarations for Rust with implementations in C++
    unsafe extern "C++" {
        include!("map_renderer.h");
        include!("mbgl/webgpu/texture2d.hpp");

        // C++ Opaque types
        /// Bridge image for rendering output.
        type BridgeImage;
        /// Map observer for handling map events.
        type MapObserver; // Created custom map observer
        /// Map renderer for rendering map content.
        type MapRenderer;

        /// Creates a new map renderer instance.
        #[allow(clippy::too_many_arguments)]
        fn MapRenderer_new(
            mapMode: MapMode,
            width: u32,
            height: u32,
            pixelRatio: f32,
            resource_options: &CxxResourceOptions,
        ) -> UniquePtr<MapRenderer>;
        /// Reads the current still image from the renderer.
        fn readStillImage(self: Pin<&mut MapRenderer>) -> UniquePtr<BridgeImage>;
        /// Gets the pixel data pointer from a bridge image.
        fn get(self: &BridgeImage) -> *const u8;
        /// Gets the size of a bridge image.
        fn size(self: &BridgeImage) -> Size;
        /// Gets the buffer length of a bridge image.
        fn bufferLength(self: &BridgeImage) -> usize;
        /// Renders a single frame.
        fn render_once(self: Pin<&mut MapRenderer>);
        /// Renders continuously.
        fn render(self: Pin<&mut MapRenderer>) -> UniquePtr<CxxString>;
        /// Sets debug visualization flags.
        fn setDebugFlags(self: Pin<&mut MapRenderer>, flags: MapDebugOptions);
        /// Sets the camera position and orientation.
        fn setCamera(
            self: Pin<&mut MapRenderer>,
            lat: f64,
            lon: f64,
            zoom: f64,
            bearing: f64,
            pitch: f64,
        );
        /// Moves the camera by the given delta.
        fn moveBy(self: Pin<&mut MapRenderer>, delta: &ScreenCoordinate);
        /// Scales the camera based on the given scale factor.
        fn scaleBy(self: Pin<&mut MapRenderer>, scale: f64, pos: &ScreenCoordinate);
        /// Loads a style from a URL.
        fn style_load_from_url(self: Pin<&mut MapRenderer>, url: &str);
        /// Sets the renderer size.
        fn setSize(self: Pin<&mut MapRenderer>, size: &Size);
        /// Gets the map observer.
        fn observer(self: Pin<&mut MapRenderer>) -> SharedPtr<MapObserver>;
        /// Adds an image to the style.
        fn style_add_image(
            self: Pin<&mut MapRenderer>,
            id: &str,
            data: &[u8],
            size: Size,
            single_distance_field: bool,
        );
        /// Removes an image from the style.
        fn style_remove_image(self: Pin<&mut MapRenderer>, id: &str);
        /// Adds a GeoJSON source to the style.
        fn style_add_geojson_source(
            self: Pin<&mut MapRenderer>,
            source: UniquePtr<CxxGeoJSONSource>,
        );
        /// Adds a symbol layer to the style.
        fn style_add_symbol_layer(self: Pin<&mut MapRenderer>, layer: UniquePtr<CxxSymbolLayer>);

        #[cfg(feature = "wgpu")]
        fn setDeviceAndQueue(self: Pin<&mut MapRenderer>, device: WGPUDevice, queue: WGPUQueue);

        #[cfg(feature = "wgpu")]
        fn takeTexture(self: Pin<&mut MapRenderer>) -> SharedPtr<Texture2D>;
    }

    #[namespace = "mln::bridge::texture"]
    unsafe extern "C++" {
        include!("texture.h");

        #[cfg(feature = "wgpu")]
        fn getRawTextureHandle(texture: &SharedPtr<Texture2D>) -> usize;
    }

    // Declarations for C++ with implementations in Rust
    extern "Rust" {
        /// Bridge logging from C++ to Rust log crate
        fn log_from_cpp(severity: EventSeverity, event: Event, code: i64, message: &str);
    }

    unsafe extern "C++" {
        include!("rust_log_observer.h");

        /// Enables or disables logging from a separate thread.
        fn Log_useLogThread(enable: bool);
    }
}

impl std::fmt::Debug for ffi::BridgeImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BridgeImage").finish()
    }
}

impl std::fmt::Debug for ffi::MapObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapObserver").finish()
    }
}

impl std::fmt::Debug for ffi::MapRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapRenderer").finish()
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
