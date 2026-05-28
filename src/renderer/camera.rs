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
    /// Geographic coordinate at the center of the viewport.
    pub center: Option<LatLng>,
    /// Altitude of the center coordinate.
    pub center_altitude: Option<f64>,
    /// Insets from the viewport edges.
    pub padding: Option<EdgeInsets>,
    /// Screen coordinate to keep fixed while changing camera values.
    pub anchor: Option<ScreenCoordinate>,
    /// Zoom level.
    pub zoom: Option<f64>,
    /// Bearing in degrees clockwise from north.
    pub bearing: Option<f64>,
    /// Pitch in degrees.
    pub pitch: Option<f64>,
    /// Roll in degrees.
    pub roll: Option<f64>,
    /// Field of view in degrees.
    pub fov: Option<f64>,
}

impl CameraUpdate {
    pub(crate) fn from_camera_options(options: FfiCameraOptions) -> Self {
        Self {
            center: options.has_center.then_some(options.center),
            center_altitude: options.has_center_altitude.then_some(options.center_altitude),
            padding: options.has_padding.then_some(options.padding),
            anchor: options.has_anchor.then_some(options.anchor),
            zoom: options.has_zoom.then_some(options.zoom),
            bearing: options.has_bearing.then_some(options.bearing),
            pitch: options.has_pitch.then_some(options.pitch),
            roll: options.has_roll.then_some(options.roll),
            fov: options.has_fov.then_some(options.fov),
        }
    }

    pub(crate) fn to_camera_options(&self) -> FfiCameraOptions {
        let mut options = FfiCameraOptions::default();

        if let Some(center) = self.center {
            options.has_center = true;
            options.center = center;
        }
        if let Some(center_altitude) = self.center_altitude {
            options.has_center_altitude = true;
            options.center_altitude = center_altitude;
        }
        if let Some(padding) = self.padding {
            options.has_padding = true;
            options.padding = padding;
        }
        if let Some(anchor) = self.anchor {
            options.has_anchor = true;
            options.anchor = anchor;
        }
        if let Some(zoom) = self.zoom {
            options.has_zoom = true;
            options.zoom = zoom;
        }
        if let Some(bearing) = self.bearing {
            options.has_bearing = true;
            options.bearing = bearing;
        }
        if let Some(pitch) = self.pitch {
            options.has_pitch = true;
            options.pitch = pitch;
        }
        if let Some(roll) = self.roll {
            options.has_roll = true;
            options.roll = roll;
        }
        if let Some(fov) = self.fov {
            options.has_fov = true;
            options.fov = fov;
        }

        options
    }
}
