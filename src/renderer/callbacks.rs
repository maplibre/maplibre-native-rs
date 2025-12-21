// Callback objects used in the bridge

use crate::renderer::bridge::ffi::MapLoadError;
use crate::renderer::bridge::ffi::MapObserverCameraChangeMode;

macro_rules! callback {
    ($callback_name:ident, $callback_f:path) => {
        pub struct $callback_name(Box<dyn $callback_f + 'static>);
        impl $callback_name {
            pub fn new<F: $callback_f + 'static>(callback: F) -> Self {
                Self(Box::new(callback))
            }
        }
    };
}

callback!(VoidCallback, Fn());
pub fn void_callback(callback: &VoidCallback) {
    (callback.0)();
}

callback!(FinishRenderingFrameCallback, Fn(bool, bool));
pub fn finish_rendering_frame_callback(
    callback: &FinishRenderingFrameCallback,
    needs_repaint: bool,
    placement_changed: bool,
) {
    (callback.0)(needs_repaint, placement_changed);
}

callback!(FailingLoadingMapCallback, Fn(MapLoadError, &str));
pub fn failing_loading_map_callback(
    callback: &FailingLoadingMapCallback,
    error: MapLoadError,
    what: &str,
) {
    (callback.0)(error, what);
}

callback!(CameraDidChangeCallback, Fn(MapObserverCameraChangeMode));
pub fn camera_did_change_callback(
    callback: &CameraDidChangeCallback,
    mode: MapObserverCameraChangeMode,
) {
    (callback.0)(mode);
}
