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
// `mbgl::Map` is constructed. A subsequent call replaces the previous
// callback but leaves existing RustFileSource instances alive until their
// owning Map is destroyed.

#include "rust/cxx.h"

namespace mln {
namespace bridge {

// Opaque Rust type — defined in src/renderer/file_source.rs.
struct FileSourceRequestCallback;

// Implementation in rust_file_source.cpp.
void register_rust_file_source_factory(rust::Box<FileSourceRequestCallback> callback);

} // namespace bridge
} // namespace mln
