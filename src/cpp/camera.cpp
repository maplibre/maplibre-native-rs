#include "maplibre_native/src/bridge.rs.h"

#include "geojson/geojson.h"
#include <mbgl/util/geojson.hpp>
#include <mbgl/util/geometry.hpp>
#include <mapbox/geojson.hpp>
#include <vector>

namespace mln {
namespace bridge {
namespace {

// Reduces a GeoJSON value (geometry / feature / feature collection) to a single
// geometry suitable for `Map::cameraForGeometry`.
mbgl::Geometry<double> toGeometry(const mbgl::GeoJSON& geojson) {
    return geojson.match(
        [](const mapbox::geojson::geometry& geometry) -> mbgl::Geometry<double> { return geometry; },
        [](const mapbox::geojson::feature& feature) -> mbgl::Geometry<double> {
            return feature.geometry;
        },
        [](const mapbox::geojson::feature_collection& features) -> mbgl::Geometry<double> {
            mapbox::geometry::geometry_collection<double> collection;
            collection.reserve(features.size());
            for (const auto& feature : features) {
                collection.push_back(feature.geometry);
            }
            return collection;
        });
}

mbgl::CameraOptions toCameraOptions(const FfiCameraOptions& camera) {
    mbgl::CameraOptions options;
    if (camera.has_center) {
        options.withCenter(mbgl::LatLng{camera.center.lat, camera.center.lng});
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
    return options;
}

FfiCameraOptions fromCameraOptions(const mbgl::CameraOptions& options) {
    FfiCameraOptions camera{};
    if (options.center) {
        camera.has_center = true;
        camera.center = LatLng{options.center->latitude(), options.center->longitude()};
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

FfiCameraOptions MapRenderer::cameraForLatLngs(rust::Slice<const LatLng> latLngs,
                                               const EdgeInsets& padding,
                                               double bearing,
                                               double pitch) {
    std::vector<mbgl::LatLng> mbglLatLngs;
    mbglLatLngs.reserve(latLngs.size());
    for (const auto& latLng : latLngs) {
        mbglLatLngs.emplace_back(latLng.lat, latLng.lng);
    }

    const mbgl::EdgeInsets mbglPadding{padding.top, padding.left, padding.bottom, padding.right};
    return fromCameraOptions(map->cameraForLatLngs(mbglLatLngs, mbglPadding, bearing, pitch));
}

FfiCameraOptions MapRenderer::cameraForGeoJson(const mln::bridge::geojson::GeoJson& geojson,
                                               const EdgeInsets& padding,
                                               double bearing,
                                               double pitch) {
    const mbgl::EdgeInsets mbglPadding{padding.top, padding.left, padding.bottom, padding.right};

    return fromCameraOptions(
        map->cameraForGeometry(toGeometry(geojson.get()), mbglPadding, bearing, pitch));
}

void MapRenderer::jumpTo(const FfiCameraOptions& cameraOptions) {
    map->jumpTo(toCameraOptions(cameraOptions));
}

} // namespace bridge
} // namespace mln
