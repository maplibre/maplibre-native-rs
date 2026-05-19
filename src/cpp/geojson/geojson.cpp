#include "geojson.h"

#include <string>
#include <utility>

namespace mln::bridge::geojson {

GeoJson::GeoJson(mapbox::geojson::geojson value) : value_(std::move(value)) {}

const mapbox::geojson::geojson& GeoJson::get() const {
    return value_;
}

std::unique_ptr<GeoJson> parse(rust::Str json) {
    return std::make_unique<GeoJson>(mapbox::geojson::parse(std::string(json)));
}

std::unique_ptr<GeoJson> clone(const GeoJson& geojson) {
    return std::make_unique<GeoJson>(geojson.get());
}

rust::String stringify(const GeoJson& geojson) {
    return mapbox::geojson::stringify(geojson.get());
}

} // namespace mln::bridge::geojson
