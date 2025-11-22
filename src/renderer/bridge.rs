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

#[allow(clippy::borrow_as_ptr)]
#[cxx::bridge(namespace = "mln::bridge")]
pub mod ffi {
    // CXX validates enum types against the C++ definition during compilation

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
    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum EventSeverity {
        Debug = 0,
        Info = 1,
        Warning = 2,
        Error = 3,
    }

    /// MapLibre Native Event types
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
    unsafe extern "C++" {
        include!("mbgl/map/mode.hpp");
        include!("mbgl/renderer/renderer_observer.hpp");

        type MapMode;
        type MapDebugOptions;
        pub type EventSeverity;
        pub type Event;
        type RendererObserver;
    }

    unsafe extern "C++" {
        include!("map_renderer.h");
        include!("renderer_observer.h");

        type MapRenderer;
        type RendererObserverMy;

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

        #[allow(clippy::too_many_arguments)]
        fn MapRenderer_new_with_observer(
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
            observer: UniquePtr<RendererObserver>,
        ) -> UniquePtr<MapRenderer>;
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
        fn MapRenderer_getStyle_loadURL(obj: Pin<&mut MapRenderer>, url: &str);

        fn RendererObserver_create_observer() -> UniquePtr<RendererObserver>;
    }

    extern "Rust" {
        /// Bridge logging from C++ to Rust log crate
        fn log_from_cpp(severity: EventSeverity, event: Event, code: i64, message: &str);
    }

    unsafe extern "C++" {
        include!("rust_log_observer.h");

        fn Log_useLogThread(enable: bool);
    }
}
