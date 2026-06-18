#include "geojson.h"

#include <mbgl/style/conversion/geojson.hpp>

#include <stdexcept>
#include <string>
#include <utility>

namespace mln::bridge::geojson {

GeoJson::GeoJson(mbgl::GeoJSON value) : value_(std::move(value)) {}

const mbgl::GeoJSON& GeoJson::get() const {
    return value_;
}

std::unique_ptr<GeoJson> parse(rust::Str json) {
    mbgl::style::conversion::Error error;
    auto geojson = mbgl::style::conversion::parseGeoJSON(std::string(json), error);
    if (!geojson) {
        throw std::runtime_error(error.message.empty() ? "failed to parse GeoJSON" : error.message);
    }
    return std::make_unique<GeoJson>(std::move(*geojson));
}

std::unique_ptr<GeoJson> clone(const GeoJson& geojson) {
    return std::make_unique<GeoJson>(geojson.get());
}

} // namespace mln::bridge::geojson
