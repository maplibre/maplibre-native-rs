#include "geojson.h"

#include <mbgl/style/conversion/geojson.hpp>

#include <optional>
#include <stdexcept>
#include <string>
#include <utility>

namespace mln::bridge::geojson {

GeoJson::GeoJson(mapbox::geojson::geojson value) : value_(std::move(value)) {}

const mapbox::geojson::geojson& GeoJson::get() const {
    return value_;
}

std::unique_ptr<GeoJson> parse(rust::Str json) {
    // Use MapLibre's public conversion API (`mbgl::*`) rather than the bundled
    // geojson-cpp `mapbox::geojson::parse`: the precompiled core amalgam is built
    // with `armerge --keep-symbols 'mbgl.*'`, which localizes every non-mbgl
    // symbol, so mapbox::geojson::parse is not linkable by downstream consumers.
    mbgl::style::conversion::Error error;
    std::optional<mbgl::GeoJSON> result =
        mbgl::style::conversion::parseGeoJSON(std::string(json), error);
    if (!result) {
        throw std::runtime_error(error.message);
    }
    return std::make_unique<GeoJson>(std::move(*result));
}

std::unique_ptr<GeoJson> clone(const GeoJson& geojson) {
    return std::make_unique<GeoJson>(geojson.get());
}

// TEMP(webgpu amalgam): commented out (not deleted) for easy restore.
// `mapbox::geojson::stringify` is localized by the amalgam's
// `armerge --keep-symbols 'mbgl.*'`, and MapLibre Native exposes no public
// `mbgl::*` GeoJSON serializer yet. Restore once one is available upstream.
// rust::String stringify(const GeoJson& geojson) {
//     return mapbox::geojson::stringify(geojson.get());
// }

} // namespace mln::bridge::geojson
