//! Map observer wrapper and callback registration helpers.

use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use cxx::SharedPtr;

use crate::bridge::{ffi, map_observer};
use crate::renderer::callbacks::{
    CameraDidChangeCallback, FailingLoadingMapCallback, FinishRenderingFrameCallback, VoidCallback,
};

type VoidCallbackFn = Rc<dyn Fn() + 'static>;
type FailLoadingMapCallbackFn = Rc<dyn Fn(MapLoadError) + 'static>;

/// Error kind reported while loading a map or style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum MapLoadErrorKind {
    /// Style parsing error.
    #[error("failed parsing style")]
    StyleParse,
    /// Style loading error.
    #[error("failed loading style")]
    StyleLoad,
    /// Resource not found.
    #[error("style not found")]
    NotFound,
    /// Unknown error.
    #[error("unknown error")]
    Unknown,
}

impl From<map_observer::MapLoadError> for MapLoadErrorKind {
    fn from(error: map_observer::MapLoadError) -> Self {
        match error {
            map_observer::MapLoadError::StyleParseError => Self::StyleParse,
            map_observer::MapLoadError::StyleLoadError => Self::StyleLoad,
            map_observer::MapLoadError::NotFoundError => Self::NotFound,
            _ => Self::Unknown,
        }
    }
}

/// Error reported while loading a map or style.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{kind}: {message}")]
#[non_exhaustive]
pub struct MapLoadError {
    /// The MapLibre Native error kind.
    pub kind: MapLoadErrorKind,
    /// The detailed message reported by MapLibre Native.
    pub message: String,
}

impl MapLoadError {
    pub(crate) fn new(kind: map_observer::MapLoadError, message: impl Into<String>) -> Self {
        Self { kind: MapLoadErrorKind::from(kind), message: message.into() }
    }
}

#[derive(Default)]
pub(crate) struct MapObserverCallbacks {
    did_finish_loading_style: RefCell<Option<VoidCallbackFn>>,
    did_fail_loading_map: RefCell<Option<FailLoadingMapCallbackFn>>,
    style_load_request_finished: RefCell<Option<VoidCallbackFn>>,
    style_load_request_failed: RefCell<Option<FailLoadingMapCallbackFn>>,
}

impl MapObserverCallbacks {
    /// Installs the C++ dispatchers that fan the style-load events out to both
    /// the internal style-load-request slots and the user-facing slots.
    ///
    /// Called once, when the renderer is created. Afterwards the `set_*`
    /// methods only swap the stored closures, so user callbacks and
    /// [`StyleLoadRequest`](crate::StyleLoadRequest) coexist without either
    /// clobbering the other.
    pub(crate) fn install(self: &Rc<Self>, observer: &SharedPtr<ffi::MapObserver>) {
        observer.setFinishLoadingStyleCallback(Box::new(VoidCallback::new({
            let callbacks = Rc::clone(self);
            move || {
                let callback = callbacks.style_load_request_finished.borrow().clone();
                if let Some(callback) = callback {
                    callback();
                }
                let callback = callbacks.did_finish_loading_style.borrow().clone();
                if let Some(callback) = callback {
                    callback();
                }
            }
        })));
        observer.setFailLoadingMapCallback(Box::new(FailingLoadingMapCallback::new({
            let callbacks = Rc::clone(self);
            move |error, what| {
                let error = MapLoadError::new(error, what);
                let callback = callbacks.style_load_request_failed.borrow().clone();
                if let Some(callback) = callback {
                    callback(error.clone());
                }
                let callback = callbacks.did_fail_loading_map.borrow().clone();
                if let Some(callback) = callback {
                    callback(error);
                }
            }
        })));
    }
}

/// Object to modify the map observer callbacks
pub struct MapObserver {
    instance: SharedPtr<ffi::MapObserver>,
    callbacks: Rc<MapObserverCallbacks>,
}

impl Debug for MapObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapObserver").finish()
    }
}

impl MapObserver {
    pub(crate) fn new(
        instance: SharedPtr<ffi::MapObserver>,
        callbacks: Rc<MapObserverCallbacks>,
    ) -> Self {
        Self { instance, callbacks }
    }

    pub(crate) fn set_style_load_request_callbacks<
        F: Fn() + 'static,
        G: Fn(MapLoadError) + 'static,
    >(
        &self,
        finished: F,
        failed: G,
    ) {
        *self.callbacks.style_load_request_finished.borrow_mut() = Some(Rc::new(finished));
        *self.callbacks.style_load_request_failed.borrow_mut() = Some(Rc::new(failed));
    }

    /// React on start loading map
    pub fn set_will_start_loading_map_callback<F: Fn() + 'static>(&self, callback: F) {
        self.instance.setWillStartLoadingMapCallback(Box::new(VoidCallback::new(callback)));
    }

    /// Set a callback to react when style loading finished
    pub fn set_did_finish_loading_style_callback<F: Fn() + 'static>(&self, callback: F) {
        *self.callbacks.did_finish_loading_style.borrow_mut() = Some(Rc::new(callback));
    }

    /// Set a callback when the map gets idle
    pub fn set_did_become_idle_callback<F: Fn() + 'static>(&self, callback: F) {
        self.instance.setBecomeIdleCallback(Box::new(VoidCallback::new(callback)));
    }

    /// Set callback to react on failing loading map
    pub fn set_did_fail_loading_map_callback<F: Fn(MapLoadError) + 'static>(&self, callback: F) {
        *self.callbacks.did_fail_loading_map.borrow_mut() = Some(Rc::new(callback));
    }

    /// Set a callback to react on camera changes
    pub fn set_camera_changed_callback<
        F: Fn(map_observer::MapObserverCameraChangeMode) + 'static,
    >(
        &self,
        callback: F,
    ) {
        self.instance.setCameraDidChangeCallback(Box::new(CameraDidChangeCallback::new(callback)));
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
