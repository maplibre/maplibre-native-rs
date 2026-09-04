#include "sources.h"
#include "geojson/geojson.h"
#include <mln/style/source.hpp>
#include <mln/style/sources/geojson_source.hpp>
#include <mln/style/types.hpp>
#include <memory>

namespace mln::bridge::style::sources {
    rust::String SourceHandle::sourceId() const {
        return source->getID();
    }

    mln::style::SourceType SourceHandle::sourceType() const {
        return source->getType();
    }

    std::unique_ptr<GeoJSONSourceHandle> SourceHandle::asGeoJson() const {
        auto* geojson = source->as<mln::style::GeoJSONSource>();
        if (!geojson) {
            return nullptr;
        }
        return std::make_unique<GeoJSONSourceHandle>(geojson);
    }

    rust::String GeoJSONSourceHandle::sourceId() const {
        return source->getID();
    }

    void GeoJSONSourceHandle::setGeoJson(const mln::bridge::geojson::GeoJson& geojson) {
        source->setGeoJSON(geojson.get());
    }

    std::unique_ptr<mln::style::Source> geojson_into_source(
        std::unique_ptr<mln::style::GeoJSONSource> source) {
        return source;
    }
}

namespace mln::bridge::style::sources::geojson {
    std::unique_ptr<mln::style::GeoJSONSource> create(rust::Str id) {
        return std::make_unique<mln::style::GeoJSONSource>(std::string(id));
    }

    void setURL(const std::unique_ptr<mln::style::GeoJSONSource>& source, rust::Str url) {
        source->setURL(std::string(url));
    }

    void setGeoJson(mln::style::GeoJSONSource& source,
                    const mln::bridge::geojson::GeoJson& geojson) {
        source.setGeoJSON(geojson.get());
    }
}
