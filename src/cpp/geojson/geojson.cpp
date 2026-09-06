#include "geojson.h"

#include <mln/style/conversion/geojson.hpp>

#include <stdexcept>
#include <string>
#include <utility>

namespace mln::bridge::geojson {

GeoJson::GeoJson(mln::GeoJSON value) : value_(std::move(value)) {}

const mln::GeoJSON& GeoJson::get() const {
    return value_;
}

std::unique_ptr<GeoJson> parse(rust::Str json) {
    mln::style::conversion::Error error;
    auto geojson = mln::style::conversion::parseGeoJSON(std::string(json), error);
    if (!geojson) {
        throw std::runtime_error(error.message.empty() ? "failed to parse GeoJSON" : error.message);
    }
    return std::make_unique<GeoJson>(std::move(*geojson));
}

std::unique_ptr<GeoJson> clone(const GeoJson& geojson) {
    return std::make_unique<GeoJson>(geojson.get());
}

// TODO(maplibre-native#4345): can be restored once the precompiled core exposes a
// public GeoJSON serializer. `mapbox::geojson::stringify` is localized (hidden) by the
// amalgam, and maplibre-native#4345 adds `mln::style::conversion::stringifyGeoJSON`.
// rust::String stringify(const GeoJson& geojson) {
//     return mln::style::conversion::stringifyGeoJSON(geojson.get());
// }

} // namespace mln::bridge::geojson
