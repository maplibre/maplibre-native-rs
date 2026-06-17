#pragma once

#include <mbgl/util/geojson.hpp>
#include "rust/cxx.h"
#include <memory>

namespace mln::bridge::geojson {

class GeoJson {
public:
    explicit GeoJson(mbgl::GeoJSON value);

    const mbgl::GeoJSON& get() const;

private:
    mbgl::GeoJSON value_;
};

std::unique_ptr<GeoJson> parse(rust::Str json);

std::unique_ptr<GeoJson> clone(const GeoJson& geojson);

} // namespace mln::bridge::geojson
