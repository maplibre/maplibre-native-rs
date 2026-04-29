//! Rust-supplied `FileSource` callback.
//!
//! Pair with the C++ side defined in `src/cpp/rust_file_source.{h,cpp}`. The
//! Rust closure handed to
//! [`ImageRendererBuilder::with_file_source_callback`](crate::ImageRendererBuilder::with_file_source_callback)
//! serves every resource mbgl asks for — styles, tilesets, tiles, glyphs,
//! sprites, images. The URL scheme is not filtered on the C++ side, so the
//! callback owns scheme dispatch (e.g. `mbtiles://`, `file://`, custom).

use std::fmt::Debug;

use crate::renderer::bridge::file_source::{FsErrorReason, ResourceKind};
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
