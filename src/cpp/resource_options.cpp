#include "rust/cxx.h"
#include "mbgl/storage/resource_options.hpp"
#include "mbgl/storage/database_file_source.hpp"
#include "mbgl/storage/file_source_manager.hpp"
#include "mbgl/util/logging.hpp"
#include "rust_file_source.h"
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

std::shared_ptr<mbgl::FileSource> applyMaximumAmbientCacheSize(const mbgl::ResourceOptions &resource_options) {
    if (::mln::bridge::has_rust_database_file_source()) {
        return nullptr;
    }
    auto fileSource =
        mbgl::FileSourceManager::get()->getFileSource(mbgl::FileSourceType::Database, resource_options);
    if (!fileSource) {
        return nullptr;
    }
    auto database = std::static_pointer_cast<mbgl::DatabaseFileSource>(fileSource);
    database->setMaximumAmbientCacheSize(resource_options.maximumCacheSize(), [](std::exception_ptr error) {
        if (!error) {
            return;
        }
        try {
            std::rethrow_exception(error);
        } catch (const std::exception &e) {
            mbgl::Log::Error(mbgl::Event::Database,
                             std::string("Failed to set the maximum ambient cache size: ") + e.what());
        }
    });
    return fileSource;
}

}
