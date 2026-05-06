#include "sources.h"
#include <mbgl/style/sources/geojson_source.hpp>
#include <mbgl/util/geometry.hpp>
#include <memory>

namespace mln::bridge::style::sources::geojson {
    std::unique_ptr<mbgl::style::GeoJSONSource> createWithDefaultOptions(rust::Str id) {
        return std::make_unique<mbgl::style::GeoJSONSource>(std::string(id));
    }

    void setURL(const std::unique_ptr<mbgl::style::GeoJSONSource>& source, rust::Str url) {
        source->setURL(std::string(url));
    }

    void setPoint(const std::unique_ptr<mbgl::style::GeoJSONSource>& source, double latitude, double longitude) {
        source->setGeoJSON(mbgl::Geometry<double>{mbgl::Point<double>{longitude, latitude}});
    }
}
