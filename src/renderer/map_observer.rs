//! Map observer wrapper and callback registration helpers.

use std::fmt::Debug;

use crate::renderer::bridge::ffi;
use crate::renderer::bridge::map_observer;
use crate::renderer::callbacks::{
    CameraDidChangeCallback, FailingLoadingMapCallback, FinishRenderingFrameCallback, VoidCallback,
};
use cxx::SharedPtr;

/// Object to modify the map observer callbacks
pub struct MapObserver {
    instance: SharedPtr<ffi::MapObserver>,
}

impl Debug for MapObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapObserver").finish()
    }
}

impl MapObserver {
    pub(crate) fn new(instance: SharedPtr<ffi::MapObserver>) -> Self {
        Self { instance }
    }

    /// React on start loading map
    pub fn set_will_start_loading_map_callback<F: Fn() + 'static>(&self, callback: F) {
        self.instance
            .setWillStartLoadingMapCallback(Box::new(VoidCallback::new(callback)));
    }

    /// Set a callback to react when style loading finished
    pub fn set_did_finish_loading_style_callback<F: Fn() + 'static>(&self, callback: F) {
        self.instance
            .setFinishLoadingStyleCallback(Box::new(VoidCallback::new(callback)));
    }

    /// Set a callback when the map gets idle
    pub fn set_did_become_idle_callback<F: Fn() + 'static>(&self, callback: F) {
        self.instance
            .setBecomeIdleCallback(Box::new(VoidCallback::new(callback)));
    }

    /// Set callback to react on failing loading map
    pub fn set_did_fail_loading_map_callback<F: Fn(map_observer::MapLoadError, &str) + 'static>(
        &self,
        callback: F,
    ) {
        self.instance
            .setFailLoadingMapCallback(Box::new(FailingLoadingMapCallback::new(callback)));
    }

    /// Set a callback to react on camera changes
    pub fn set_camera_changed_callback<
        F: Fn(map_observer::MapObserverCameraChangeMode) + 'static,
    >(
        &self,
        callback: F,
    ) {
        self.instance
            .setCameraDidChangeCallback(Box::new(CameraDidChangeCallback::new(callback)));
    }

    /// Set a callback to react on finished rendering frames
    pub fn set_finish_rendering_frame_callback<
        F: Fn(/*needs_repaint:*/ bool, /*placement_changed:*/ bool) + 'static,
    >(
        &self,
        callback: F,
    ) {
        self.instance
            .setFinishRenderingFrameCallback(Box::new(FinishRenderingFrameCallback::new(callback)));
    }
}
