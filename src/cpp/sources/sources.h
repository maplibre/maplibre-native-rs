#include <memory>
#include "rust/cxx.h"

namespace mbgl::style {
    class Source;
    class GeoJSONSource;
}

namespace mln::bridge::style::sources::geojson {
    /// Creates a new GeoJSON source with default options.
    std::unique_ptr<mbgl::style::GeoJSONSource> createWithDefaultOptions(rust::Str id);

    /// Sets the URL for loading GeoJSON data.
    void setURL(const std::unique_ptr<mbgl::style::GeoJSONSource>& source, rust::Str url);

    void setPoint(const std::unique_ptr<mbgl::style::GeoJSONSource>& source, double latitude, double longitude);
}
