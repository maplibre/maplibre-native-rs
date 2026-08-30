#include "rust/cxx.h"
#include "mln/storage/resource_options.hpp"
#include "mln/storage/database_file_source.hpp"
#include "mln/storage/file_source_manager.hpp"
#include "mln/util/logging.hpp"
#include "rust_file_source.h"
#include "util.h"
#include <cassert>
#include <memory>

namespace mln::bridge::resource_options {

std::unique_ptr<mln::ResourceOptions> new_() {
    return std::make_unique<mln::ResourceOptions>();
}

void withAssetPath(mln::ResourceOptions &resource_options, rust::Slice<const uint8_t> path) {
    resource_options.withAssetPath(::mln::bridge::rustSliceToString(path));
}

void withCachePath(mln::ResourceOptions &resource_options, rust::Slice<const uint8_t> path) {
    resource_options.withCachePath(::mln::bridge::rustSliceToString(path));
}

void withApiKey(mln::ResourceOptions &resource_options, rust::Str key) {
    resource_options.withApiKey(std::string(key.data(), key.size()));
}

void withMaximumCacheSize(mln::ResourceOptions &resource_options, uint64_t max_cache_size) {
    resource_options.withMaximumCacheSize(max_cache_size);
}

void withTileServerOptions(mln::ResourceOptions &resource_options, const mln::TileServerOptions& tile_server_options) {
    resource_options.withTileServerOptions(tile_server_options);
}

std::shared_ptr<mln::FileSource> applyMaximumAmbientCacheSize(const mln::ResourceOptions &resource_options) {
    if (::mln::bridge::has_rust_database_file_source()) {
        return nullptr;
    }
    auto fileSource =
        mln::FileSourceManager::get()->getFileSource(mln::FileSourceType::Database, resource_options);
    if (!fileSource) {
        return nullptr;
    }
    auto database = std::static_pointer_cast<mln::DatabaseFileSource>(fileSource);
    database->setMaximumAmbientCacheSize(resource_options.maximumCacheSize(), [](std::exception_ptr error) {
        if (!error) {
            return;
        }
        try {
            std::rethrow_exception(error);
        } catch (const std::exception &e) {
            mln::Log::Error(mln::Event::Database,
                             std::string("Failed to set the maximum ambient cache size: ") + e.what());
        }
    });
    return fileSource;
}

}
