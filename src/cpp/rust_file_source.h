#pragma once

// Rust-backed FileSource bridge.

#include "rust/cxx.h"
#include <mln/actor/scheduler.hpp>
#include <mln/storage/file_source.hpp>
#include <mln/storage/resource.hpp>
#include <mln/storage/response.hpp>

#include <atomic>
#include <functional>
#include <memory>
#include <mutex>
#include <optional>
#include <string>

namespace mln {
namespace bridge {

using ResourceKind = mln::Resource::Kind;
using ErrorReason = mln::Response::Error::Reason;
using FileSourceType = mln::FileSourceType;

// Opaque Rust type (defined in the Rust `file_source` module) and the cxx
// shared structs (defined in the generated bridge.rs.h)
struct BoxedFileSource;
struct RawResourceRequest;
struct RawResponse;

// Native state for one in-flight request
struct RequestState {
  mln::FileSource::Callback cb;
  std::function<void()> dispatch;
  std::mutex response_mutex;
  std::optional<mln::Response> response;
  std::atomic<bool> cancelled{false};
  // Cached body held until revalidation completes.
  std::shared_ptr<const std::string> withheld_prior_body;
};

// Holds a forward's (cache-write) completion callback until `forward_complete`.
struct ForwardState {
  std::function<void()> cb;
};

void register_rust_file_source(FileSourceType source_type,
                               rust::Box<BoxedFileSource> source);

bool has_rust_database_file_source() noexcept;

void responder_complete(std::shared_ptr<RequestState> state,
                        const RawResponse &response);

void responder_cancel(std::shared_ptr<RequestState> state);

RawResponse roundtrip_response_for_test(const RawResponse &response);

void forward_complete(std::shared_ptr<ForwardState> state);

} // namespace bridge
} // namespace mln
