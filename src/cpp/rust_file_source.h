#pragma once

// Rust-backed FileSource bridge.

#include "rust/cxx.h"
#include <mbgl/actor/scheduler.hpp>
#include <mbgl/storage/file_source.hpp>
#include <mbgl/storage/resource.hpp>
#include <mbgl/storage/response.hpp>

#include <atomic>
#include <functional>
#include <memory>
#include <mutex>
#include <optional>

namespace mln {
namespace bridge {

using ResourceKind = mbgl::Resource::Kind;
using ErrorReason = mbgl::Response::Error::Reason;
using FileSourceType = mbgl::FileSourceType;

// Opaque Rust type (defined in the Rust `file_source` module) and the cxx
// shared structs (defined in the generated bridge.rs.h)
struct BoxedFileSource;
struct RawResourceRequest;
struct RawResponse;

// Native state for one in-flight request
struct RequestState {
  mbgl::FileSource::Callback cb;
  std::function<void()> dispatch;
  std::mutex response_mutex;
  std::optional<mbgl::Response> response;
  std::atomic<bool> cancelled{false};
};

// Holds a forward's (cache-write) completion callback until `forward_complete`.
struct ForwardState {
  std::function<void()> cb;
};

void register_rust_file_source(FileSourceType source_type,
                               rust::Box<BoxedFileSource> source);

void responder_complete(std::shared_ptr<RequestState> state,
                        const RawResponse &response);

void responder_cancel(std::shared_ptr<RequestState> state);

RawResponse roundtrip_response_for_test(const RawResponse &response);

void forward_complete(std::shared_ptr<ForwardState> state);

} // namespace bridge
} // namespace mln
