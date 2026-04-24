#include "rust_file_source.h"
#include "maplibre_native/src/renderer/bridge.rs.h"

#include <mbgl/storage/file_source.hpp>
#include <mbgl/storage/file_source_manager.hpp>
#include <mbgl/storage/resource.hpp>
#include <mbgl/storage/resource_options.hpp>
#include <mbgl/storage/response.hpp>
#include <mbgl/util/async_request.hpp>
#include <mbgl/util/client_options.hpp>
#include <mbgl/util/logging.hpp>

#include <memory>
#include <string>
#include <utility>

namespace mln {
namespace bridge {

namespace {

// Compile-time cross-checks: the Rust-side `ResourceKind` and `FsErrorReason`
// enums duplicate mbgl discriminants over the cxx boundary as raw u8. Pin
// each discriminant so an upstream mbgl reorder fails the build instead of
// silently mapping tile requests to spritejson requests.
static_assert(static_cast<uint8_t>(mbgl::Resource::Kind::Unknown)      == 0, "ResourceKind::Unknown discriminant drift");
static_assert(static_cast<uint8_t>(mbgl::Resource::Kind::Style)        == 1, "ResourceKind::Style discriminant drift");
static_assert(static_cast<uint8_t>(mbgl::Resource::Kind::Source)       == 2, "ResourceKind::Source discriminant drift");
static_assert(static_cast<uint8_t>(mbgl::Resource::Kind::Tile)         == 3, "ResourceKind::Tile discriminant drift");
static_assert(static_cast<uint8_t>(mbgl::Resource::Kind::Glyphs)       == 4, "ResourceKind::Glyphs discriminant drift");
static_assert(static_cast<uint8_t>(mbgl::Resource::Kind::SpriteImage)  == 5, "ResourceKind::SpriteImage discriminant drift");
static_assert(static_cast<uint8_t>(mbgl::Resource::Kind::SpriteJSON)   == 6, "ResourceKind::SpriteJSON discriminant drift");
static_assert(static_cast<uint8_t>(mbgl::Resource::Kind::Image)        == 7, "ResourceKind::Image discriminant drift");

// RustFsResponse uses `0` as the no-error sentinel. Verify mbgl's Reason
// enum never produces `0`, and pin the reason discriminants we map onto.
static_assert(static_cast<uint8_t>(mbgl::Response::Error::Reason::Success)    != 0, "FsErrorReason sentinel 0 collides with Success");
static_assert(static_cast<uint8_t>(mbgl::Response::Error::Reason::NotFound)   == 2, "FsErrorReason::NotFound discriminant drift");
static_assert(static_cast<uint8_t>(mbgl::Response::Error::Reason::Server)     == 3, "FsErrorReason::Server discriminant drift");
static_assert(static_cast<uint8_t>(mbgl::Response::Error::Reason::Connection) == 4, "FsErrorReason::Connection discriminant drift");
static_assert(static_cast<uint8_t>(mbgl::Response::Error::Reason::RateLimit)  == 5, "FsErrorReason::RateLimit discriminant drift");
static_assert(static_cast<uint8_t>(mbgl::Response::Error::Reason::Other)      == 6, "FsErrorReason::Other discriminant drift");

// AsyncRequest subclass whose destructor cancels. Our dispatch is synchronous
// so by the time this handle reaches the caller the callback has already
// fired — cancellation is a structural no-op.
class NoopAsyncRequest final : public mbgl::AsyncRequest {
public:
    NoopAsyncRequest() = default;
    ~NoopAsyncRequest() override = default;
};

class RustFileSource final : public mbgl::FileSource {
public:
    RustFileSource(std::shared_ptr<rust::Box<FileSourceRequestCallback>> callback,
                   mbgl::ResourceOptions resourceOpts,
                   mbgl::ClientOptions clientOpts)
        : callback_(std::move(callback)),
          resourceOpts_(std::move(resourceOpts)),
          clientOpts_(std::move(clientOpts)) {}

    std::unique_ptr<mbgl::AsyncRequest> request(const mbgl::Resource& resource,
                                                Callback cb) override {
        // Static-render mode has no pumped run loop on the render thread, so
        // posting the callback via `RunLoop::Get()->invokeCancellable` would
        // deadlock the `frontend->render` call waiting for its own tile.
        // Sync dispatch is safe because Rust-backed sources (mbtiles SQLite,
        // filesystem read) don't block long enough to matter, matching mbgl's
        // in-memory test doubles.
        mbgl::Log::Debug(mbgl::Event::General,
                         "rust-fs request kind=" + std::to_string(static_cast<unsigned>(resource.kind))
                         + " url=" + resource.url);
        cb(invokeCallback(resource));
        return std::make_unique<NoopAsyncRequest>();
    }

    bool canRequest(const mbgl::Resource&) const override { return true; }

    void setResourceOptions(mbgl::ResourceOptions options) override {
        resourceOpts_ = std::move(options);
    }
    mbgl::ResourceOptions getResourceOptions() override {
        return resourceOpts_.clone();
    }

    void setClientOptions(mbgl::ClientOptions options) override {
        clientOpts_ = std::move(options);
    }
    mbgl::ClientOptions getClientOptions() override {
        return clientOpts_.clone();
    }

private:
    mbgl::Response invokeCallback(const mbgl::Resource& resource) {
        const rust::Str url(resource.url.data(), resource.url.size());
        RustFsResponse rr = fs_request_callback(**callback_, url,
                                                static_cast<uint8_t>(resource.kind));

        mbgl::Response response;
        if (rr.error_reason != 0) {
            response.error = std::make_unique<mbgl::Response::Error>(
                static_cast<mbgl::Response::Error::Reason>(rr.error_reason),
                std::string(rr.error_message.data(), rr.error_message.size()));
            return response;
        }

        response.noContent = rr.no_content;
        if (!rr.no_content && !rr.data.empty()) {
            // The Rust `Vec<u8>` was moved across the cxx boundary (no copy);
            // this is the one unavoidable copy — mbgl insists on
            // `shared_ptr<const std::string>`.
            response.data = std::make_shared<std::string>(
                reinterpret_cast<const char*>(rr.data.data()),
                rr.data.size());
        }
        return response;
    }

    std::shared_ptr<rust::Box<FileSourceRequestCallback>> callback_;
    mbgl::ResourceOptions resourceOpts_;
    mbgl::ClientOptions clientOpts_;
};

} // namespace

void register_rust_file_source_factory(rust::Box<FileSourceRequestCallback> callback) {
    auto shared_callback = std::make_shared<rust::Box<FileSourceRequestCallback>>(std::move(callback));
    mbgl::FileSourceManager::get()->registerFileSourceFactory(
        mbgl::FileSourceType::ResourceLoader,
        [shared_callback](const mbgl::ResourceOptions& ro, const mbgl::ClientOptions& co)
            -> std::unique_ptr<mbgl::FileSource> {
            return std::make_unique<RustFileSource>(shared_callback, ro.clone(), co.clone());
        });
}

} // namespace bridge
} // namespace mln
