use crate::bridge::ffi::{EdgeInsets, FfiCameraOptions, LatLng};
use crate::ScreenCoordinate;

impl EdgeInsets {
    /// Creates equal insets for all edges.
    #[must_use]
    pub fn all(value: f64) -> Self {
        Self { top: value, left: value, bottom: value, right: value }
    }
}

/// A partial camera update.
///
/// All fields are optional, so this can represent partial camera updates such
/// as changing only the zoom or only the viewport padding.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CameraUpdate {
    options: FfiCameraOptions,
}

impl CameraUpdate {
    /// Creates an empty camera update.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the geographic coordinate at the center of the viewport.
    #[must_use]
    pub fn center(mut self, center: LatLng) -> Self {
        self.options.has_center = true;
        self.options.center = center;
        self
    }

    /// Sets the insets from the viewport edges.
    #[must_use]
    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.options.has_padding = true;
        self.options.padding = padding;
        self
    }

    /// Sets the screen coordinate to keep fixed while changing camera values.
    #[must_use]
    pub fn anchor(mut self, anchor: ScreenCoordinate) -> Self {
        self.options.has_anchor = true;
        self.options.anchor = anchor;
        self
    }

    /// Sets the zoom level.
    #[must_use]
    pub fn zoom(mut self, zoom: f64) -> Self {
        self.options.has_zoom = true;
        self.options.zoom = zoom;
        self
    }

    /// Sets the bearing in degrees clockwise from north.
    #[must_use]
    pub fn bearing(mut self, bearing: f64) -> Self {
        self.options.has_bearing = true;
        self.options.bearing = bearing;
        self
    }

    /// Sets the pitch in degrees.
    #[must_use]
    pub fn pitch(mut self, pitch: f64) -> Self {
        self.options.has_pitch = true;
        self.options.pitch = pitch;
        self
    }

    pub(crate) fn from_camera_options(options: FfiCameraOptions) -> Self {
        Self { options }
    }

    pub(crate) fn to_camera_options(&self) -> FfiCameraOptions {
        self.options
    }
}
