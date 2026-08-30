#pragma once

#include "rust/cxx.h"
#include <memory>

namespace mln {
    class FileSource;
    class ResourceOptions;
    class TileServerOptions;
}

namespace mln::bridge::resource_options {

std::unique_ptr<mln::ResourceOptions> new_();
void withAssetPath(mln::ResourceOptions &resource_options, rust::Slice<const uint8_t> path);
void withCachePath(mln::ResourceOptions &resource_options, rust::Slice<const uint8_t> path);
void withApiKey(mln::ResourceOptions &resource_options, rust::Str key);
void withMaximumCacheSize(mln::ResourceOptions &resource_options, uint64_t max_cache_size);
void withTileServerOptions(mln::ResourceOptions &resource_options, const mln::TileServerOptions& tile_server_options);

// The caller must keep the returned source alive; the manager holds only a weak reference.
std::shared_ptr<mln::FileSource> applyMaximumAmbientCacheSize(const mln::ResourceOptions &resource_options);

}
