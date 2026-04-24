//! Rust-supplied FileSource callback.
//!
//! Pair with the C++ side defined in `src/cpp/rust_file_source.{h,cpp}`. The
//! Rust closure handed to
//! [`ImageRendererBuilder::with_file_source_callback`](crate::ImageRendererBuilder::with_file_source_callback)
//! serves every resource mbgl asks for — styles, tilesets, tiles, glyphs,
//! sprites, images. The URL scheme is not filtered on the C++ side, so the
//! callback owns scheme dispatch (e.g. `mbtiles://`, `file://`, custom).

use std::fmt::Debug;

use crate::renderer::callbacks::callback;

/// Kind of resource being requested. Mirrors `mbgl::Resource::Kind` from
/// `mbgl/storage/resource.hpp`; discriminant values are pinned byte-for-byte
/// to that enum by `static_assert`s in `rust_file_source.cpp`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ResourceKind {
    /// Unknown / unspecified resource kind.
    Unknown = 0,
    /// A style.json.
    Style = 1,
    /// A TileJSON / source descriptor.
    Source = 2,
    /// A single tile (vector or raster).
    Tile = 3,
    /// A glyph PBF range.
    Glyphs = 4,
    /// A sprite sheet PNG.
    SpriteImage = 5,
    /// A sprite sheet JSON.
    SpriteJSON = 6,
    /// A generic image resource.
    Image = 7,
}

impl ResourceKind {
    fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Style,
            2 => Self::Source,
            3 => Self::Tile,
            4 => Self::Glyphs,
            5 => Self::SpriteImage,
            6 => Self::SpriteJSON,
            7 => Self::Image,
            _ => Self::Unknown,
        }
    }
}

/// Error reason for a failed resource request. Mirrors
/// `mbgl::Response::Error::Reason`; discriminant values are pinned to that
/// enum by `static_assert`s in `rust_file_source.cpp`. The `0` discriminant
/// is intentionally reserved on the FFI side to mean "no error" —
/// `mbgl::Response::Error::Reason` starts at `Success = 1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FsErrorReason {
    /// Resource not found at the requested URL.
    NotFound = 2,
    /// Server-side error (5xx, etc.).
    Server = 3,
    /// Transport-level connection failure.
    Connection = 4,
    /// Rate-limit response.
    RateLimit = 5,
    /// Any other error.
    Other = 6,
}

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
    /// mbgl-internal retry/backoff logic still applies.
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
    kind: u8,
) -> crate::renderer::bridge::file_source::RustFsResponse {
    use crate::renderer::bridge::file_source::RustFsResponse;
    match (callback.0)(url, ResourceKind::from_u8(kind)) {
        FsResponse::Ok(bytes) => RustFsResponse {
            data: bytes,
            error_reason: 0,
            error_message: String::new(),
            no_content: false,
        },
        FsResponse::NoContent => RustFsResponse {
            data: Vec::new(),
            error_reason: 0,
            error_message: String::new(),
            no_content: true,
        },
        FsResponse::Error { reason, message } => RustFsResponse {
            data: Vec::new(),
            error_reason: reason as u8,
            error_message: message,
            no_content: false,
        },
    }
}
