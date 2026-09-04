#include "rust_file_source.h"
#include "maplibre_native/src/bridge.rs.h"

#include <mln/actor/scheduler.hpp>
#include <mln/storage/file_source.hpp>
#include <mln/storage/file_source_manager.hpp>
#include <mln/storage/resource.hpp>
#include <mln/storage/resource_options.hpp>
#include <mln/storage/response.hpp>
#include <mln/util/async_request.hpp>
#include <mln/util/chrono.hpp>
#include <mln/util/client_options.hpp>

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

mln::Response buildResponse(const RawResponse& r) {
    mln::Response response;
    if (r.has_error) {
        std::optional<mln::Timestamp> retry_after;
        if (r.has_retry_after) {
            retry_after = mln::Timestamp(mln::Seconds(r.retry_after_epoch_s));
        }
        response.error = std::make_unique<mln::Response::Error>(
            r.error_reason, std::string(r.error_message), retry_after);
    }
    response.noContent = r.no_content;
    response.notModified = r.not_modified;
    response.mustRevalidate = r.must_revalidate;
    if (r.has_data) {
        response.data = std::make_shared<std::string>(
            reinterpret_cast<const char*>(r.data.data()), r.data.size());
    }
    if (r.has_modified) {
        response.modified = mln::Timestamp(mln::Seconds(r.modified_epoch_s));
    }
    if (r.has_expires) {
        response.expires = mln::Timestamp(mln::Seconds(r.expires_epoch_s));
    }
    if (r.has_etag) {
        response.etag = std::string(r.etag);
    }
    return response;
}

RawResponse toRustResponse(const mln::Response& response) {
    RawResponse out{};
    out.error_reason = ErrorReason::Success;
    if (response.error) {
        out.has_error = true;
        out.error_reason = response.error->reason;
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
        out.etag = rust::String::lossy(*response.etag);
    }
    return out;
}

RawResourceRequest toRustResourceRequest(const mln::Resource& resource, bool include_prior_data) {
    RawResourceRequest out{};
    out.url = rust::String::lossy(resource.url);
    out.kind = resource.kind;
    out.loading_methods = static_cast<std::uint8_t>(resource.loadingMethod);
    out.is_volatile = resource.storagePolicy == mln::Resource::StoragePolicy::Volatile;
    out.is_low_priority = resource.priority == mln::Resource::Priority::Low;
    out.is_offline = resource.usage == mln::Resource::Usage::Offline;

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
        std::chrono::duration_cast<mln::Milliseconds>(resource.minimumUpdateInterval).count();

    return out;
}

void completeState(const std::shared_ptr<RequestState>& state, mln::Response response) {
    if (state->cancelled.load()) {
        return;
    }

    {
        std::scoped_lock lock(state->response_mutex);
        // Match OnlineFileSource: priorData means native withheld the cached
        // representation, so a 304 must deliver it to complete the load.
        if (response.notModified && state->withheld_prior_body) {
            response.data = std::move(state->withheld_prior_body);
            response.notModified = false;
        }
        state->response.emplace(std::move(response));
    }
    state->dispatch();
}

void completeForwardState(const std::shared_ptr<ForwardState>& state) {
    if (state->cb) {
        state->cb();
    }
}

class RustAsyncRequest final : public mln::AsyncRequest {
public:
    RustAsyncRequest(rust::Box<RequestHandleFfi> handle, std::shared_ptr<RequestState> state)
        : handle_(std::move(handle)), state_(std::move(state)) {}

    ~RustAsyncRequest() override {
        state_->cancelled.store(true);
        handle_->cancel();
    }

private:
    rust::Box<RequestHandleFfi> handle_;
    std::shared_ptr<RequestState> state_;
};

class RustFileSource final : public mln::FileSource {
public:
    RustFileSource(std::shared_ptr<rust::Box<BoxedFileSource>> source,
                   bool materializeNotModified,
                   mln::ResourceOptions resourceOpts,
                   mln::ClientOptions clientOpts)
        : source_(std::move(source)),
          materializeNotModified_(materializeNotModified),
          resourceOpts_(std::move(resourceOpts)),
          clientOpts_(std::move(clientOpts)) {}

    std::unique_ptr<mln::AsyncRequest> request(const mln::Resource& resource,
                                                Callback cb) override {
        auto state = std::make_shared<RequestState>();
        state->cb = std::move(cb);
        if (materializeNotModified_) {
            state->withheld_prior_body = resource.priorData;
        }

        // FileSource callbacks must run on the thread that issued request().
        // Capture the scheduler's weak binding while that scheduler is known to
        // be alive. A late completion becomes a no-op after scheduler teardown.
        std::weak_ptr<RequestState> weakState = state;
        state->dispatch = mln::Scheduler::GetCurrent()->bindOnce([weakState]() mutable {
            auto state = weakState.lock();
            if (!state || state->cancelled.load()) {
                return;
            }

            std::optional<mln::Response> response;
            {
                std::scoped_lock lock(state->response_mutex);
                response = std::move(state->response);
                state->response.reset();
            }
            if (response && !state->cancelled.load()) {
                state->cb(std::move(*response));
            }
        });

        auto request = toRustResourceRequest(resource, true);
        rust::Box<RequestHandleFfi> handle = (**source_).request(request, state);
        return std::make_unique<RustAsyncRequest>(std::move(handle), std::move(state));
    }

    bool canRequest(const mln::Resource& resource) const override {
        auto request = toRustResourceRequest(resource, false);
        return (**source_).can_request(request);
    }

    void forward(const mln::Resource& resource,
                 const mln::Response& response,
                 std::function<void()> callback) override {
        auto state = std::make_shared<ForwardState>();
        if (callback) {
            state->cb = mln::Scheduler::GetCurrent()->bindOnce(std::move(callback));
        }

        auto request = toRustResourceRequest(resource, true);
        (**source_).forward(request, toRustResponse(response), std::move(state));
    }

    void setResourceOptions(mln::ResourceOptions options) override {
        resourceOpts_ = std::move(options);
    }
    mln::ResourceOptions getResourceOptions() override {
        return resourceOpts_.clone();
    }

    void setClientOptions(mln::ClientOptions options) override {
        clientOpts_ = std::move(options);
    }
    mln::ClientOptions getClientOptions() override {
        return clientOpts_.clone();
    }

private:
    std::shared_ptr<rust::Box<BoxedFileSource>> source_;
    bool materializeNotModified_;
    mln::ResourceOptions resourceOpts_;
    mln::ClientOptions clientOpts_;
};

} // namespace

void responder_complete(std::shared_ptr<RequestState> state, const RawResponse& response) {
    completeState(state, buildResponse(response));
}

void responder_cancel(std::shared_ptr<RequestState> state) {
    state->cancelled.store(true);
}

RawResponse roundtrip_response_for_test(const RawResponse& response) {
    return toRustResponse(buildResponse(response));
}

void forward_complete(std::shared_ptr<ForwardState> state) {
    completeForwardState(state);
}

namespace {
std::atomic<bool> rust_database_source_registered{false};
} // namespace

bool has_rust_database_file_source() noexcept {
    return rust_database_source_registered.load(std::memory_order_acquire);
}

void register_rust_file_source(FileSourceType source_type, rust::Box<BoxedFileSource> source) {
    if (source_type == FileSourceType::Database) {
        rust_database_source_registered.store(true, std::memory_order_release);
    }
    auto shared_source = std::make_shared<rust::Box<BoxedFileSource>>(std::move(source));
    const bool materialize_not_modified = source_type == FileSourceType::Network;
    mln::FileSourceManager::get()->registerFileSourceFactory(
        source_type,
        [shared_source, materialize_not_modified](const mln::ResourceOptions& ro,
                                                  const mln::ClientOptions& co)
            -> std::unique_ptr<mln::FileSource> {
            return std::make_unique<RustFileSource>(
                shared_source, materialize_not_modified, ro.clone(), co.clone());
        });
}

} // namespace bridge
} // namespace mln
