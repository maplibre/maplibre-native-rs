//! Callback types used by the renderer event API.

use std::fmt::Debug;

use crate::renderer::bridge::map_observer::{MapLoadError, MapObserverCameraChangeMode};

macro_rules! callback {
    ($callback_name:ident, $callback_f:path) => {
        /// Callback object
        pub struct $callback_name(Box<dyn $callback_f + 'static>);
        impl $callback_name {
            /// Create a new callback object
            pub fn new<F: $callback_f + 'static>(callback: F) -> Self {
                Self(Box::new(callback))
            }
        }

        impl Debug for $callback_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Callback: {}", stringify!($callback_name))
            }
        }
    };
}

callback!(VoidCallback, Fn());
/// General callback with any argument
pub fn void_callback(callback: &VoidCallback) {
    (callback.0)();
}

callback!(FinishRenderingFrameCallback, Fn(bool, bool));
/// `finish_rendering_frame_callback`
pub fn finish_rendering_frame_callback(
    callback: &FinishRenderingFrameCallback,
    needs_repaint: bool,
    placement_changed: bool,
) {
    (callback.0)(needs_repaint, placement_changed);
}

callback!(FailingLoadingMapCallback, Fn(MapLoadError, &str));
/// `failing_loading_map_callback`
pub fn failing_loading_map_callback(
    callback: &FailingLoadingMapCallback,
    error: MapLoadError,
    what: &str,
) {
    (callback.0)(error, what);
}

callback!(CameraDidChangeCallback, Fn(MapObserverCameraChangeMode));
/// `camera_did_change_callback`
pub fn camera_did_change_callback(
    callback: &CameraDidChangeCallback,
    mode: MapObserverCameraChangeMode,
) {
    (callback.0)(mode);
}
