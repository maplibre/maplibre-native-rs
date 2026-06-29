#include "rust_file_source.h"
#include "maplibre_native/src/bridge.rs.h"

#include <mbgl/actor/scheduler.hpp>
#include <mbgl/storage/file_source.hpp>
#include <mbgl/storage/file_source_manager.hpp>
#include <mbgl/storage/resource.hpp>
#include <mbgl/storage/resource_options.hpp>
#include <mbgl/storage/response.hpp>
#include <mbgl/util/async_request.hpp>
#include <mbgl/util/chrono.hpp>
#include <mbgl/util/client_options.hpp>
#include <mbgl/util/run_loop.hpp>

#include <atomic>
#include <chrono>
#include <cstddef>
#include <cstdint>
#include <functional>
#include <memory>
#include <mutex>
#include <optional>
#include <string>
#include <utility>

namespace mln {
namespace bridge {

namespace {

// Shared state for one mbgl resource request.
struct RequestState {
    mbgl::FileSource::Callback cb;
    // Scheduler of the thread that issued the request.
    mbgl::Scheduler* scheduler = nullptr;
    std::mutex mutex;
    std::atomic<bool> done{false};
    std::atomic<bool> cancelled{false};
};

// Holds a forward's (cache-write) completion callback until `forward_complete`.
struct ForwardState {
    std::function<void()> cb;
};

// RawResponse -> mbgl::Response (response delivery).
//
// `error` and the other fields are independent and can coexist: a cache miss is
// `error = NotFound` together with `noContent = true`, mirroring mbgl's own
// `DatabaseFileSource`, so don't return early on an error.
mbgl::Response buildResponse(const RawResponse& r) {
    mbgl::Response response;
    if (r.has_error) {
        std::optional<mbgl::Timestamp> retryAfter;
        if (r.has_retry_after) {
            retryAfter = mbgl::Timestamp(mbgl::Seconds(r.retry_after_epoch_s));
        }
        response.error = std::make_unique<mbgl::Response::Error>(
            r.error_reason, std::string(r.error_message), std::move(retryAfter));
    }
    response.noContent = r.no_content;
    response.notModified = r.not_modified;
    response.mustRevalidate = r.must_revalidate;
    if (r.has_data) {
        // Non-null data pointer even for empty bodies.
        response.data = std::make_shared<std::string>(
            reinterpret_cast<const char*>(r.data.data()), r.data.size());
    }
    if (r.has_modified) {
        response.modified = mbgl::Timestamp(mbgl::Seconds(r.modified_epoch_s));
    }
    if (r.has_expires) {
        response.expires = mbgl::Timestamp(mbgl::Seconds(r.expires_epoch_s));
    }
    if (r.has_etag) {
        response.etag = std::string(r.etag);
    }
    return response;
}

// mbgl::Response -> RawResponse
RawResponse toRustResponse(const mbgl::Response& response) {
    RawResponse out{};
    out.error_reason = ErrorReason::Success;
    if (response.error) {
        out.has_error = true;
        out.error_reason = response.error->reason;
        // Lossy: a non-UTF-8 message must not throw across the FFI boundary.
        out.error_message = rust::String::lossy(response.error->message);
        if (response.error->retryAfter) {
            out.has_retry_after = true;
            out.retry_after_epoch_s = response.error->retryAfter->time_since_epoch().count();
        }
    }
    out.no_content = response.noContent;
    out.not_modified = response.notModified;
    out.must_revalidate = response.mustRevalidate;
    if (response.data) {
        out.has_data = true;
        out.data.reserve(response.data->size());
        for (unsigned char byte : *response.data) {
            out.data.push_back(byte);
        }
    }
    if (response.modified) {
        out.has_modified = true;
        out.modified_epoch_s = response.modified->time_since_epoch().count();
    }
    if (response.expires) {
        out.has_expires = true;
        out.expires_epoch_s = response.expires->time_since_epoch().count();
    }
    if (response.etag) {
        out.has_etag = true;
        // Lossy: a server-provided non-UTF-8 etag must not throw across the FFI.
        out.etag = rust::String::lossy(*response.etag);
    }
    return out;
}

// mbgl::Resource -> RawResourceRequest
RawResourceRequest toRustResourceRequest(const mbgl::Resource& resource, bool include_prior_data) {
    RawResourceRequest out{};
    out.url = rust::String::lossy(resource.url);
    out.kind = resource.kind;
    out.loading_methods = static_cast<std::uint8_t>(resource.loadingMethod);
    out.is_volatile = resource.storagePolicy == mbgl::Resource::StoragePolicy::Volatile;

    if (resource.tileData) {
        out.has_tile = true;
        out.tile_url_template = rust::String::lossy(resource.tileData->urlTemplate);
        out.tile_pixel_ratio = resource.tileData->pixelRatio;
        out.tile_x = resource.tileData->x;
        out.tile_y = resource.tileData->y;
        out.tile_z = resource.tileData->z;
    }

    if (resource.dataRange) {
        out.has_data_range = true;
        out.data_range_start = resource.dataRange->first;
        out.data_range_end = resource.dataRange->second;
    }
    if (resource.priorModified) {
        out.has_prior_modified = true;
        out.prior_modified_epoch_s = resource.priorModified->time_since_epoch().count();
    }
    if (resource.priorExpires) {
        out.has_prior_expires = true;
        out.prior_expires_epoch_s = resource.priorExpires->time_since_epoch().count();
    }
    if (resource.priorEtag) {
        out.has_prior_etag = true;
        out.prior_etag = rust::String::lossy(*resource.priorEtag);
    }
    if (include_prior_data && resource.priorData) {
        out.has_prior_data = true;
        out.prior_data.reserve(resource.priorData->size());
        for (unsigned char byte : *resource.priorData) {
            out.prior_data.push_back(byte);
        }
    }
    out.minimum_update_interval_ms =
        std::chrono::duration_cast<mbgl::Milliseconds>(resource.minimumUpdateInterval).count();

    return out;
}

// MapLibre Native requires the request callback on the thread that issued request(),
// so marshal it back to that thread's scheduler rather than calling cb here.
void completeState(const std::shared_ptr<RequestState>& state, mbgl::Response response) {
    mbgl::Scheduler* scheduler = nullptr;
    {
        std::lock_guard lock(state->mutex);
        if (state->cancelled.load()) {
            return;
        }
        bool expected = false;
        if (!state->done.compare_exchange_strong(expected, true)) {
            return;
        }
        scheduler = state->scheduler;
    }

    scheduler->schedule([state, response = std::move(response)]() mutable {
        if (!state->cancelled.load()) {
            state->cb(std::move(response));
        }
    });
}

// Hand an owning ref to Rust as an opaque token; reclaimed by takeToken().
template <typename State>
std::size_t makeToken(const std::shared_ptr<State>& state) {
    return reinterpret_cast<std::size_t>(new std::shared_ptr<State>(state));
}

// Reclaim the owning ref Rust held, deleting the holder. Call exactly once.
template <typename State>
std::shared_ptr<State> takeToken(std::size_t token) {
    auto* holder = reinterpret_cast<std::shared_ptr<State>*>(token);
    std::shared_ptr<State> state = *holder;
    delete holder;
    return state;
}

void completeForwardState(const std::shared_ptr<ForwardState>& state) {
    if (state->cb) {
        state->cb();
    }
}

class RustAsyncRequest final : public mbgl::AsyncRequest {
public:
    RustAsyncRequest(rust::Box<RequestHandleFfi> handle, std::shared_ptr<RequestState> state)
        : handle_(std::move(handle)), state_(std::move(state)) {}

    ~RustAsyncRequest() override {
        bool should_cancel = false;
        {
            std::lock_guard lock(state_->mutex);
            state_->cancelled.store(true);
            should_cancel = !state_->done.load();
        }
        if (should_cancel) {
            handle_->cancel();
        }
    }

private:
    rust::Box<RequestHandleFfi> handle_;
    std::shared_ptr<RequestState> state_;
};

class RustFileSource final : public mbgl::FileSource {
public:
    RustFileSource(std::shared_ptr<rust::Box<BoxedFileSource>> source,
                   mbgl::ResourceOptions resourceOpts,
                   mbgl::ClientOptions clientOpts)
        : source_(std::move(source)),
          resourceOpts_(std::move(resourceOpts)),
          clientOpts_(std::move(clientOpts)) {}

    std::unique_ptr<mbgl::AsyncRequest> request(const mbgl::Resource& resource,
                                                Callback cb) override {
        auto state = std::make_shared<RequestState>();
        state->cb = std::move(cb);
        state->scheduler = mbgl::util::RunLoop::Get();

        // Extra owning ref held by Rust until completion or drop.
        auto token = makeToken(state);

        auto request = toRustResourceRequest(resource, true);
        rust::Box<RequestHandleFfi> handle = (**source_).request(request, token);
        return std::make_unique<RustAsyncRequest>(std::move(handle), std::move(state));
    }

    bool canRequest(const mbgl::Resource& resource) const override {
        auto request = toRustResourceRequest(resource, false);
        return (**source_).can_request(request);
    }

    void forward(const mbgl::Resource& resource,
                 const mbgl::Response& response,
                 std::function<void()> callback) override {
        auto state = std::make_shared<ForwardState>();
        if (callback) {
            state->cb = mbgl::Scheduler::GetCurrent()->bindOnce(std::move(callback));
        }

        // Extra owning ref held by Rust until the cache write completes.
        auto token = makeToken(state);

        auto request = toRustResourceRequest(resource, true);
        (**source_).forward(request, toRustResponse(response), token);
    }

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
    std::shared_ptr<rust::Box<BoxedFileSource>> source_;
    mbgl::ResourceOptions resourceOpts_;
    mbgl::ClientOptions clientOpts_;
};

} // namespace

void responder_complete(std::size_t token, const RawResponse& response) {
    completeState(takeToken<RequestState>(token), buildResponse(response));
}

void forward_complete(std::size_t token) {
    completeForwardState(takeToken<ForwardState>(token));
}

void register_rust_file_source(FileSourceType source_type, rust::Box<BoxedFileSource> source) {
    auto shared_source = std::make_shared<rust::Box<BoxedFileSource>>(std::move(source));
    mbgl::FileSourceManager::get()->registerFileSourceFactory(
        source_type,
        [shared_source](const mbgl::ResourceOptions& ro, const mbgl::ClientOptions& co)
            -> std::unique_ptr<mbgl::FileSource> {
            return std::make_unique<RustFileSource>(shared_source, ro.clone(), co.clone());
        });
}

} // namespace bridge
} // namespace mln
