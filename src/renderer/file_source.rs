//! Rust-supplied `FileSource` callback.
//!
//! Pair with the C++ side defined in `src/cpp/rust_file_source.{h,cpp}`. The
//! Rust closure handed to [`register_file_source_callback`] serves every
//! resource mbgl asks for — styles, tilesets, tiles, glyphs, sprites,
//! images. The URL scheme is not filtered on the C++ side, so the callback
//! owns scheme dispatch (e.g. `mbtiles://`, `file://`, custom).

// Required by the `callback!` macro expansion: it emits
// `impl Debug for $callback_name`, so `Debug` must be in scope as a
// trait, not just a derive macro.
use std::fmt::Debug;

use crate::renderer::bridge::file_source::{
    register_rust_file_source_factory, FsErrorReason, ResourceKind,
};
use crate::renderer::callbacks::callback;

/// Return value for a resource request callback.
#[derive(Debug)]
pub enum FsResponse {
    /// Request succeeded; bytes are the raw resource body.
    Ok(Vec<u8>),
    /// Request was a well-formed miss (e.g. mbtiles tile not present at
    /// this z/x/y). mbgl treats this as a 204-equivalent — no error, no
    /// body. Overzoomed tiles should use this rather than [`FsResponse::Error`].
    NoContent,
    /// Request failed. The reason maps directly to the mbgl error enum so
    /// mbgl-internal retry/backoff logic still applies. Don't use
    /// [`FsErrorReason::Success`] here — pick `Other` if no other variant
    /// fits.
    Error {
        /// Category of failure.
        reason: FsErrorReason,
        /// Human-readable message for logging.
        message: String,
    },
}

// Send + Sync are load-bearing: `mbgl::FileSourceManager` is a singleton, so
// the same callback is captured by every `RustFileSource` instance that the
// factory produces. If a consumer constructs more than one `ImageRenderer`
// (or mbgl ever spawns a second file-source-owning thread upstream), the
// callback will be invoked from multiple threads and must be thread-safe.
callback!(FileSourceRequestCallback,
          Fn(&str, ResourceKind) -> FsResponse,
          Send, Sync);

/// Install a Rust closure as the `ResourceLoader` file-source callback for
/// every subsequently constructed `mbgl::Map`.
///
/// The closure is invoked for every resource mbgl needs to render the
/// style — tiles, glyphs, sprites, source manifests, etc. It replaces the
/// mbgl default `ResourceLoader` entirely, so the closure owns URL-scheme
/// dispatch (`mbtiles://`, `file://`, `https://`, custom).
///
/// # Process-global, with caching subtleties
///
/// `mbgl::FileSourceManager` is a process-wide singleton and this call
/// mutates global state. There are two layers to be aware of:
///
/// 1. **Factory replacement.** A subsequent call replaces the registered
///    factory; the prior closure is dropped once no `RustFileSource`
///    instance still references it.
/// 2. **Instance cache.** `FileSourceManager` *also* caches `FileSource`
///    instances keyed by `(type, ResourceOptions)`. `getFileSource` returns
///    the cached instance via `weak_ptr::lock()` whenever it's still alive,
///    bypassing the factory. So replacing the callback while any prior
///    renderer (built with the same `ResourceOptions`) is still in scope
///    has no effect on either the prior renderer *or* on new renderers
///    that share its `ResourceOptions` — they all keep using the original
///    closure until the cached instance is dropped.
///
/// In practice: install the callback once, before constructing any
/// renderer, and keep it for the process lifetime. Two concurrent
/// renderers requiring different callbacks are not supported by this
/// layer.
///
/// `Send + Sync` are required because mbgl may invoke the same closure
/// from multiple renderer threads concurrently.
pub fn register_file_source_callback<F>(callback: F)
where
    F: Fn(&str, ResourceKind) -> FsResponse + Send + Sync + 'static,
{
    register_rust_file_source_factory(Box::new(FileSourceRequestCallback::new(callback)));
}

/// Bridge function invoked by C++ for every resource request. Not called
/// directly from user code — exposed via the cxx bridge in
/// `src/renderer/bridge.rs`.
pub(crate) fn fs_request_callback(
    callback: &FileSourceRequestCallback,
    url: &str,
    kind: ResourceKind,
) -> crate::renderer::bridge::file_source::RustFsResponse {
    use crate::renderer::bridge::file_source::RustFsResponse;
    #[cfg(feature = "log")]
    log::debug!("rust-fs request kind={kind:?} url={url}");
    match (callback.0)(url, kind) {
        FsResponse::Ok(bytes) => RustFsResponse {
            data: bytes,
            error_reason: FsErrorReason::Success,
            error_message: String::new(),
            no_content: false,
        },
        FsResponse::NoContent => RustFsResponse {
            data: Vec::new(),
            error_reason: FsErrorReason::Success,
            error_message: String::new(),
            no_content: true,
        },
        FsResponse::Error { reason, message } => RustFsResponse {
            data: Vec::new(),
            error_reason: reason,
            error_message: message,
            no_content: false,
        },
    }
}
