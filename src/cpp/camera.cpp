#include "maplibre_native/src/bridge.rs.h"

namespace mln {
namespace bridge {
namespace {

mbgl::CameraOptions toCameraOptions(const FfiCameraOptions& camera) {
    mbgl::CameraOptions options;
    if (camera.has_center) {
        options.withCenter(mbgl::LatLng{camera.center.lat, camera.center.lng});
    }
    if (camera.has_center_altitude) {
        options.withCenterAltitude(camera.center_altitude);
    }
    if (camera.has_padding) {
        options.withPadding(mbgl::EdgeInsets{
            camera.padding.top,
            camera.padding.left,
            camera.padding.bottom,
            camera.padding.right,
        });
    }
    if (camera.has_anchor) {
        options.withAnchor(camera.anchor);
    }
    if (camera.has_zoom) {
        options.withZoom(camera.zoom);
    }
    if (camera.has_bearing) {
        options.withBearing(camera.bearing);
    }
    if (camera.has_pitch) {
        options.withPitch(camera.pitch);
    }
    if (camera.has_roll) {
        options.withRoll(camera.roll);
    }
    if (camera.has_fov) {
        options.withFov(camera.fov);
    }
    return options;
}

FfiCameraOptions fromCameraOptions(const mbgl::CameraOptions& options) {
    FfiCameraOptions camera{};
    if (options.center) {
        camera.has_center = true;
        camera.center = LatLng{options.center->latitude(), options.center->longitude()};
    }
    if (options.centerAltitude) {
        camera.has_center_altitude = true;
        camera.center_altitude = *options.centerAltitude;
    }
    if (options.padding) {
        camera.has_padding = true;
        camera.padding = EdgeInsets{
            options.padding->top(),
            options.padding->left(),
            options.padding->bottom(),
            options.padding->right(),
        };
    }
    if (options.anchor) {
        camera.has_anchor = true;
        camera.anchor = *options.anchor;
    }
    if (options.zoom) {
        camera.has_zoom = true;
        camera.zoom = *options.zoom;
    }
    if (options.bearing) {
        camera.has_bearing = true;
        camera.bearing = *options.bearing;
    }
    if (options.pitch) {
        camera.has_pitch = true;
        camera.pitch = *options.pitch;
    }
    if (options.roll) {
        camera.has_roll = true;
        camera.roll = *options.roll;
    }
    if (options.fov) {
        camera.has_fov = true;
        camera.fov = *options.fov;
    }
    return camera;
}

} // namespace

FfiCameraOptions MapRenderer::cameraForLatLngBounds(const LatLngBounds& bounds,
                                                   const EdgeInsets& padding,
                                                   double bearing,
                                                   double pitch) {
    const mbgl::LatLngBounds mbglBounds = mbgl::LatLngBounds::hull(
        {bounds.southwest.lat, bounds.southwest.lng},
        {bounds.northeast.lat, bounds.northeast.lng});
    const mbgl::EdgeInsets mbglPadding{padding.top, padding.left, padding.bottom, padding.right};

    return fromCameraOptions(map->cameraForLatLngBounds(mbglBounds, mbglPadding, bearing, pitch));
}

void MapRenderer::jumpTo(const FfiCameraOptions& cameraOptions) {
    map->jumpTo(toCameraOptions(cameraOptions));
}

} // namespace bridge
} // namespace mln
