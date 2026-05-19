#pragma once

#include <mapbox/geojson.hpp>
#include "rust/cxx.h"
#include <memory>

namespace mln::bridge::geojson {

class GeoJson {
public:
    explicit GeoJson(mapbox::geojson::geojson value);

    const mapbox::geojson::geojson& get() const;

private:
    mapbox::geojson::geojson value_;
};

std::unique_ptr<GeoJson> parse(rust::Str json);

std::unique_ptr<GeoJson> clone(const GeoJson& geojson);

rust::String stringify(const GeoJson& geojson);

} // namespace mln::bridge::geojson
