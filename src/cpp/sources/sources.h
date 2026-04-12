#include <memory>
#include "rust/cxx.h"

namespace mbgl::style {
    class Source;
    class GeoJSONSource;
}

namespace mln::bridge::style::sources::geojson {
    std::unique_ptr<mbgl::style::GeoJSONSource> create(rust::Str id);

    void setPoint(const std::unique_ptr<mbgl::style::GeoJSONSource>& source, double latitude, double longitude);
}
