#include <memory>
#include "rust/cxx.h"

namespace mbgl::style {
    class Source;
    class GeoJSONSource;
}

namespace mln::bridge::geojson {
    class GeoJson;
}

namespace mln::bridge::style::sources {
    // Upcasts derived `mbgl::style::Source` handles to the base type so that
    // `Style::addSource(unique_ptr<Source>)` can be invoked through a single
    // bridge function regardless of the concrete source type.
    std::unique_ptr<mbgl::style::Source> geojson_into_source(
        std::unique_ptr<mbgl::style::GeoJSONSource> source);
}

namespace mln::bridge::style::sources::geojson {
    /// Creates a new GeoJSON source with default options.
    std::unique_ptr<mbgl::style::GeoJSONSource> createWithDefaultOptions(rust::Str id);

    /// Sets the URL for loading GeoJSON data.
    void setURL(const std::unique_ptr<mbgl::style::GeoJSONSource>& source, rust::Str url);

    void setGeoJson(const std::unique_ptr<mbgl::style::GeoJSONSource>& source,
                    const mln::bridge::geojson::GeoJson& geojson);
}
