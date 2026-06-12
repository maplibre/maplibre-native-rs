#pragma once

#include "rust/cxx.h"
#include <cstdint>
#include <memory>
#include <string>

namespace mbgl::style {
class Source;
class GeoJSONSource;
enum class SourceType : ::std::uint8_t;
} // namespace mbgl::style

namespace mln::bridge::geojson {
class GeoJson;
}

namespace mln::bridge::style::sources {
class GeoJSONSourceHandle;

// Non-owning handle to a source owned by the style.
// Valid only while the owning style outlives it;
// the Rust side ties this to a `&mut StyleRef`.
class SourceHandle {
public:
  explicit SourceHandle(mbgl::style::Source *source_) : source(source_) {}

  rust::String sourceId() const;
  mbgl::style::SourceType sourceType() const;
  std::unique_ptr<GeoJSONSourceHandle> asGeoJson() const;

private:
  mbgl::style::Source *source;
};

// Non-owning handle to a GeoJSON source owned by the style.
class GeoJSONSourceHandle {
public:
  explicit GeoJSONSourceHandle(mbgl::style::GeoJSONSource *source_)
      : source(source_) {}

  rust::String sourceId() const;
  void setGeoJson(const mln::bridge::geojson::GeoJson &geojson);

private:
  mbgl::style::GeoJSONSource *source;
};

// Upcasts derived `mbgl::style::Source` handles to the base type so that
// `Style::addSource(unique_ptr<Source>)` can be invoked through a single
// bridge function regardless of the concrete source type.
std::unique_ptr<mbgl::style::Source>
geojson_into_source(std::unique_ptr<mbgl::style::GeoJSONSource> source);
} // namespace mln::bridge::style::sources

namespace mln::bridge::style::sources::geojson {
std::unique_ptr<mbgl::style::GeoJSONSource> create(rust::Str id);

void setURL(const std::unique_ptr<mbgl::style::GeoJSONSource> &source,
            rust::Str url);

void setGeoJson(mbgl::style::GeoJSONSource &source,
                const mln::bridge::geojson::GeoJson &geojson);
} // namespace mln::bridge::style::sources::geojson
