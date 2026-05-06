#pragma once

// Rust-backed FileSource bridge.
//
// Installs an `mbgl::FileSource` factory for `FileSourceType::ResourceLoader`
// that delegates every resource request to a Rust closure. This replaces the
// default ResourceLoader (which composes Asset/Database/Network/Mbtiles/Pmtiles
// sources) with a single Rust-supplied handler — letting callers serve
// mbtiles://, file://, and custom schemes from Rust without running a sidecar
// HTTP server or pre-extracting tiles.
//
// Factory registration is process-global (mbgl::FileSourceManager is a
// singleton). Call `register_rust_file_source_factory` once before any
// `mbgl::Map` is constructed.
//
// `FileSourceManager` *also* caches FileSource instances by `(type,
// ResourceOptions)`; `registerFileSourceFactory` only swaps the factory
// slot and never evicts the cache. A subsequent call therefore takes
// effect only for `Map`s constructed with `ResourceOptions` that don't
// already have a live cached FileSource — in practice, only after every
// previously built renderer has been dropped, or by varying
// `ResourceOptions` (e.g. a unique `platformContext`) per renderer. Two
// concurrent renderers with different callbacks are not supported by
// this layer.

#include "rust/cxx.h"
#include <mbgl/storage/resource.hpp>
#include <mbgl/storage/response.hpp>

namespace mln {
namespace bridge {

// cxx doesn't support nested enums directly, so flatten mbgl's
// `Resource::Kind` and `Response::Error::Reason` into top-level aliases
// the cxx::bridge can pick up. Same pattern as `MapObserverCameraChangeMode`
// in src/cpp/map_observer.h. Names + discriminants must stay aligned with
// the Rust-side enum declarations in src/renderer/bridge.rs; cxx codegen
// validates the match at compile time.
using ResourceKind = mbgl::Resource::Kind;
using FsErrorReason = mbgl::Response::Error::Reason;

// Opaque Rust type — defined in src/renderer/file_source.rs.
struct FileSourceRequestCallback;

// Implementation in rust_file_source.cpp.
void register_rust_file_source_factory(rust::Box<FileSourceRequestCallback> callback);

} // namespace bridge
} // namespace mln
