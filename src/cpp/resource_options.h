#pragma once

#include "rust/cxx.h"
#include <memory>

namespace mbgl {
    class FileSource;
    class ResourceOptions;
    class TileServerOptions;
}

namespace mln::bridge::resource_options {

std::unique_ptr<mbgl::ResourceOptions> new_();
void withAssetPath(mbgl::ResourceOptions &resource_options, rust::Slice<const uint8_t> path);
void withCachePath(mbgl::ResourceOptions &resource_options, rust::Slice<const uint8_t> path);
void withApiKey(mbgl::ResourceOptions &resource_options, rust::Str key);
void withMaximumCacheSize(mbgl::ResourceOptions &resource_options, uint64_t max_cache_size);
void withTileServerOptions(mbgl::ResourceOptions &resource_options, const mbgl::TileServerOptions& tile_server_options);

// The caller must keep the returned source alive; the manager holds only a weak reference.
std::shared_ptr<mbgl::FileSource> applyMaximumAmbientCacheSize(const mbgl::ResourceOptions &resource_options);

}
