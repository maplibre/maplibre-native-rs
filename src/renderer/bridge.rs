use crate::renderer::callbacks::{
    camera_did_change_callback, failing_loading_map_callback, finish_rendering_frame_callback,
    void_callback, CameraDidChangeCallback, FailingLoadingMapCallback,
    FinishRenderingFrameCallback, VoidCallback,
};
use cxx::UniquePtr;
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

pub(crate) struct Extent3d(pub(crate) wgpu::Extent3d);
pub(crate) struct TextureDimension(pub(crate) wgpu::TextureDimension);
pub(crate) struct TextureFormat(pub(crate) wgpu::TextureFormat);
pub(crate) struct TextureUsages(pub(crate) wgpu::TextureUsages);
pub struct TextureInterface(pub UniquePtr<ffi::Texture>);
pub struct TextureViewInterface(pub UniquePtr<ffi::TextureView>);
pub(crate) struct TextureAspect(pub(crate) wgpu::TextureAspect);
pub(crate) struct TextureViewDimension(pub(crate) wgpu::TextureViewDimension);

impl std::fmt::Debug for TextureInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TextureInterface")
    }
}

impl wgpu::custom::TextureInterface for TextureInterface {
    fn create_view(&self, desc: &wgpu::TextureViewDescriptor<'_>) -> wgpu::custom::DispatchTextureView {
        // TODO: get rid of unwraps!
        let format = TextureFormat(desc.format.unwrap_or(self.0.getFormat().0));
        let dimension = TextureViewDimension(desc.dimension.unwrap()); // _or(self.0.getDimension().0);
        let usage = TextureUsages(desc.usage.unwrap_or(self.0.getUsage().0));
        let aspect = TextureAspect(desc.aspect);
        let base_mip_level = desc.base_mip_level;
        let mip_level_count = desc.mip_level_count.unwrap();
        let base_array_layer = desc.base_array_layer;
        let array_layer_count = desc.array_layer_count.unwrap(); // _or(default)
        wgpu::custom::DispatchTextureView::custom(TextureViewInterface(self.0.createView(
            format,
            dimension,
            usage,
            aspect,
            base_mip_level,
            mip_level_count,
            base_array_layer,
            array_layer_count,
        )))
    }

    fn destroy(&self) {
        self.0.destroy();
    }
}

unsafe impl Send for TextureInterface {}
unsafe impl Sync for TextureInterface {}

impl std::fmt::Debug for TextureViewInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TextureViewInterface")
    }
}

impl wgpu::custom::TextureViewInterface for TextureViewInterface {}

unsafe impl Send for TextureViewInterface {}
unsafe impl Sync for TextureViewInterface {}

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
        pub type EventSeverity;
        pub type Event;
        type MapLoadError;
    }

    #[namespace = "mbgl"]
    extern "C++" {
        // Left side must match a type in C++! Right side must be defined in Rust
        // example: type VoidCallback = super::VoidCallbackampoline;
        type ScreenCoordinate = super::ScreenCoordinate;
        type Size = super::Size;
    }

    #[namespace = ""]
    extern "C++" {
        type WGPUTextureDimension = super::TextureDimension;
        type WGPUTextureFormat = super::TextureFormat;
        type WGPUTextureUsage = super::TextureUsages;
        type WGPUExtent3D = super::Extent3d;
        type WGPUTextureViewDimension = super::TextureViewDimension;
        type WGPUTextureAspect = super::TextureAspect;
    }

    // Declarations for Rust with implementations in C++
    unsafe extern "C++" {
        include!("map_renderer.h");
        include!("map_observer.h"); // Required to find functions below

        // Opaque types
        type BridgeImage;
        type MapObserverCameraChangeMode;
        type MapObserver; // Created custom map observer
        type MapRenderer;
        type Texture;
        type TextureView;

        #[allow(clippy::too_many_arguments)]
        fn MapRenderer_new(
            mapMode: MapMode,
            width: u32,
            height: u32,
            pixelRatio: f32,
            cachePath: &[u8],
            assetRoot: &[u8],
            apiKey: &str,
            baseUrl: &str,
            uriSchemeAlias: &str,
            apiKeyParameterName: &str,
            sourceTemplate: &str,
            styleTemplate: &str,
            spritesTemplate: &str,
            glyphsTemplate: &str,
            tileTemplate: &str,
            requiresApiKey: bool,
        ) -> UniquePtr<MapRenderer>;
        fn MapRenderer_readStillImage(obj: Pin<&mut MapRenderer>) -> UniquePtr<BridgeImage>;
        fn get(self: &BridgeImage) -> *const u8;
        fn size(self: &BridgeImage) -> Size;
        fn bufferLength(self: &BridgeImage) -> usize;
        fn getTexture(self: Pin<&mut MapRenderer>) -> UniquePtr<Texture>;
        fn MapRenderer_render_once(obj: Pin<&mut MapRenderer>);
        fn MapRenderer_render(obj: Pin<&mut MapRenderer>) -> UniquePtr<CxxString>;
        fn MapRenderer_setDebugFlags(obj: Pin<&mut MapRenderer>, flags: MapDebugOptions);
        fn MapRenderer_setCamera(
            obj: Pin<&mut MapRenderer>,
            lat: f64,
            lon: f64,
            zoom: f64,
            bearing: f64,
            pitch: f64,
        );
        fn MapRenderer_moveBy(obj: Pin<&mut MapRenderer>, delta: &ScreenCoordinate);
        fn MapRenderer_scaleBy(obj: Pin<&mut MapRenderer>, scale: f64, pos: &ScreenCoordinate);
        fn MapRenderer_getStyle_loadURL(obj: Pin<&mut MapRenderer>, url: &str);
        fn MapRenderer_setSize(obj: Pin<&mut MapRenderer>, size: &Size);
        fn observer(self: Pin<&mut MapRenderer>) -> SharedPtr<MapObserver>;

        // With `self: Pin<&mut MapObserver>` as first argument, it is a non static method of that object.
        // cxx searches for such a method
        fn setWillStartLoadingMapCallback(self: &MapObserver, callback: Box<VoidCallback>);
        fn setFinishLoadingStyleCallback(self: &MapObserver, callback: Box<VoidCallback>);
        fn setBecomeIdleCallback(self: &MapObserver, callback: Box<VoidCallback>);
        fn setFailLoadingMapCallback(self: &MapObserver, callback: Box<FailingLoadingMapCallback>);
        fn setFinishRenderingFrameCallback(
            self: &MapObserver,
            callback: Box<FinishRenderingFrameCallback>,
        );
        fn setCameraDidChangeCallback(self: &MapObserver, callback: Box<CameraDidChangeCallback>);

        // Texture
        fn createView(self: &Texture, format: WGPUTextureFormat, dimension: WGPUTextureViewDimension, usage: WGPUTextureUsage, aspect: WGPUTextureAspect, base_mip_level: u32, mip_level_count: u32, base_array_layer: u32, array_layer_count: u32) -> UniquePtr<TextureView>;
        fn destroy(self: &Texture);
        fn getMipLevelCount(self: &Texture) -> u32;
        fn getSampleCount(self: &Texture) -> u32;
        fn getDimension(self: &Texture) -> WGPUTextureDimension;
        fn getFormat(self: &Texture) -> WGPUTextureFormat;
        fn getUsage(self: &Texture) -> WGPUTextureUsage;
        fn getExtend3d(self: &Texture) -> WGPUExtent3D;
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

unsafe impl cxx::ExternType for TextureDimension {
    type Id = cxx::type_id!("WGPUTextureDimension");
    type Kind = cxx::kind::Trivial;
}

unsafe impl cxx::ExternType for TextureFormat {
    type Id = cxx::type_id!("WGPUTextureFormat");
    type Kind = cxx::kind::Trivial;
}

unsafe impl cxx::ExternType for TextureUsages {
    type Id = cxx::type_id!("WGPUTextureUsage");
    type Kind = cxx::kind::Trivial;
}

unsafe impl cxx::ExternType for Extent3d {
    type Id = cxx::type_id!("WGPUExtent3D");
    type Kind = cxx::kind::Trivial;
}

unsafe impl cxx::ExternType for TextureViewDimension {
    type Id = cxx::type_id!("WGPUTextureViewDimension");
    type Kind = cxx::kind::Trivial;
}

unsafe impl cxx::ExternType for TextureAspect {
    type Id = cxx::type_id!("WGPUTextureAspect");
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
