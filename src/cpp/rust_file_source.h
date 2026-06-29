#pragma once

// Rust-backed FileSource bridge.

#include "rust/cxx.h"
#include <mbgl/storage/file_source.hpp>
#include <mbgl/storage/resource.hpp>
#include <mbgl/storage/response.hpp>

#include <cstddef>

namespace mln {
namespace bridge {

using ResourceKind = mbgl::Resource::Kind;
using ErrorReason = mbgl::Response::Error::Reason;
using FileSourceType = mbgl::FileSourceType;

// Opaque Rust type (defined in the Rust `file_source` module) and the cxx shared
// structs (defined in the generated bridge.rs.h)
struct BoxedFileSource;
struct RawResourceRequest;
struct RawResponse;

void register_rust_file_source(FileSourceType source_type,
                               rust::Box<BoxedFileSource> source);

void responder_complete(std::size_t token, const RawResponse &response);

void forward_complete(std::size_t token);

} // namespace bridge
} // namespace mln
