#include "rust/cxx.h"
#include "mbgl/storage/resource_options.hpp"
#include "util.h"
#include <cassert>
#include <memory>

namespace mln::bridge::resource_options {

std::unique_ptr<mbgl::ResourceOptions> new_() {
    return std::make_unique<mbgl::ResourceOptions>();
}

void withAssetPath(mbgl::ResourceOptions &resource_options, rust::Slice<const uint8_t> path) {
    resource_options.withAssetPath(::mln::bridge::rustSliceToString(path));
}

void withCachePath(mbgl::ResourceOptions &resource_options, rust::Slice<const uint8_t> path) {
    resource_options.withCachePath(::mln::bridge::rustSliceToString(path));
}

void withApiKey(mbgl::ResourceOptions &resource_options, rust::Str key) {
    resource_options.withApiKey(std::string(key.data(), key.size()));
}

void withMaximumCacheSize(mbgl::ResourceOptions &resource_options, uint64_t max_cache_size) {
    resource_options.withMaximumCacheSize(max_cache_size);
}

void withTileServerOptions(mbgl::ResourceOptions &resource_options, const mbgl::TileServerOptions& tile_server_options) {
    resource_options.withTileServerOptions(tile_server_options);
}

}
