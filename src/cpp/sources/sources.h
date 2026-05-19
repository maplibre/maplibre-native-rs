#include <memory>
#include "rust/cxx.h"

namespace mbgl::style {
    class Source;
    class GeoJSONSource;
}

namespace mln::bridge::geojson {
    class GeoJson;
}

namespace mln::bridge::style::sources::geojson {
    std::unique_ptr<mbgl::style::GeoJSONSource> create(rust::Str id);

    void setGeoJson(const std::unique_ptr<mbgl::style::GeoJSONSource>& source,
                    const mln::bridge::geojson::GeoJson& geojson);
}
