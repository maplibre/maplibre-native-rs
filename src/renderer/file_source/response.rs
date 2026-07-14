//! The response a [`FileSource`](super::FileSource) delivers, mirroring
//! `mbgl::Response`.

use std::time::SystemTime;

use crate::bridge::file_source::{ErrorReason, RawResponse};

use super::{from_epoch, to_epoch};

/// An error result for a request (mirrors `mbgl::Response::Error`).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Error {
    /// Category of failure. Use [`ErrorReason::Other`] if none fits.
    pub reason: ErrorReason,
    /// Human-readable message for logging.
    pub message: String,
    /// Retry-after time for rate-limited responses, if known.
    pub retry_after: Option<SystemTime>,
}

/// The response a [`FileSource`](super::FileSource) delivers for a request.
///
/// Mirrors `mbgl::Response`: body bytes, error state, and HTTP cache metadata.
/// Construct with [`Response::data`], [`Response::no_content`],
/// [`Response::not_modified`], or [`Response::error`].
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Response {
    /// Present when the request failed. `None` means success.
    pub error: Option<Error>,
    /// Empty successful response. Used for HTTP 204 and missing tiles.
    pub no_content: bool,
    /// HTTP 304: the cached copy is still current.
    pub not_modified: bool,
    /// `Cache-Control: must-revalidate`.
    pub must_revalidate: bool,
    /// Body bytes. `Some(Vec::new())` means an explicit zero-length body.
    pub data: Option<Vec<u8>>,
    /// `Last-Modified`, if known.
    pub modified: Option<SystemTime>,
    /// `Expires`, if known.
    pub expires: Option<SystemTime>,
    /// `ETag`, if known.
    pub etag: Option<String>,
}

impl Response {
    /// A successful response carrying `data`.
    #[must_use]
    pub fn data(data: Vec<u8>) -> Self {
        Self { data: Some(data), ..Self::default() }
    }

    /// An empty successful response (e.g. HTTP 204 or a missing tile).
    #[must_use]
    pub fn no_content() -> Self {
        Self { no_content: true, ..Self::default() }
    }

    /// HTTP 304: the previously cached copy is current.
    #[must_use]
    pub fn not_modified() -> Self {
        Self { not_modified: true, ..Self::default() }
    }

    /// A failed response. Don't pass [`ErrorReason::Success`] — it is coerced to
    /// [`ErrorReason::Other`].
    #[must_use]
    pub fn error(reason: ErrorReason, message: impl Into<String>) -> Self {
        let reason = if reason == ErrorReason::Success { ErrorReason::Other } else { reason };
        Self {
            error: Some(Error { reason, message: message.into(), retry_after: None }),
            ..Self::default()
        }
    }

    /// Set the retry-after time on an error response.
    #[must_use]
    pub fn with_retry_after(mut self, retry_after: SystemTime) -> Self {
        if let Some(error) = &mut self.error {
            error.retry_after = Some(retry_after);
        }
        self
    }

    /// Set the `Expires` time.
    #[must_use]
    pub fn with_expires(mut self, expires: SystemTime) -> Self {
        self.expires = Some(expires);
        self
    }

    /// Set the `Last-Modified` time.
    #[must_use]
    pub fn with_modified(mut self, modified: SystemTime) -> Self {
        self.modified = Some(modified);
        self
    }

    /// Set the `ETag`.
    #[must_use]
    pub fn with_etag(mut self, etag: impl Into<String>) -> Self {
        self.etag = Some(etag.into());
        self
    }

    /// Set the `must-revalidate` flag.
    #[must_use]
    pub fn with_must_revalidate(mut self, must_revalidate: bool) -> Self {
        self.must_revalidate = must_revalidate;
        self
    }

    /// Convert to the flat FFI shape.
    pub(super) fn into_ffi(self) -> RawResponse {
        let (has_error, error_reason, error_message, has_retry_after, retry_after_epoch_s) =
            match self.error {
                Some(Error { reason, message, retry_after }) => {
                    let reason =
                        if reason == ErrorReason::Success { ErrorReason::Other } else { reason };
                    let (has_retry_after, retry_after_epoch_s) = to_epoch(retry_after);
                    (true, reason, message, has_retry_after, retry_after_epoch_s)
                }
                None => (false, ErrorReason::Success, String::new(), false, 0),
            };
        let (has_data, data) = match self.data {
            Some(data) => (true, data),
            None => (false, Vec::new()),
        };
        let (has_modified, modified_epoch_s) = to_epoch(self.modified);
        let (has_expires, expires_epoch_s) = to_epoch(self.expires);
        let (has_etag, etag) = match self.etag {
            Some(etag) => (true, etag),
            None => (false, String::new()),
        };
        RawResponse {
            has_error,
            error_reason,
            error_message,
            has_retry_after,
            retry_after_epoch_s,
            no_content: self.no_content,
            not_modified: self.not_modified,
            must_revalidate: self.must_revalidate,
            has_data,
            data,
            has_modified,
            modified_epoch_s,
            has_expires,
            expires_epoch_s,
            has_etag,
            etag,
        }
    }

    /// Reconstruct from the flat FFI shape (for `forward`).
    pub(super) fn from_ffi(ffi: &RawResponse) -> Self {
        Self {
            error: ffi.has_error.then(|| Error {
                reason: ffi.error_reason,
                message: ffi.error_message.clone(),
                retry_after: from_epoch(ffi.has_retry_after, ffi.retry_after_epoch_s),
            }),
            no_content: ffi.no_content,
            not_modified: ffi.not_modified,
            must_revalidate: ffi.must_revalidate,
            data: ffi.has_data.then(|| ffi.data.clone()),
            modified: from_epoch(ffi.has_modified, ffi.modified_epoch_s),
            expires: from_epoch(ffi.has_expires, ffi.expires_epoch_s),
            etag: ffi.has_etag.then(|| ffi.etag.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use super::{ErrorReason, Response};

    fn roundtrip(response: Response) -> Response {
        Response::from_ffi(&response.into_ffi())
    }

    #[test]
    fn data_response_roundtrips_all_metadata() {
        let modified = SystemTime::UNIX_EPOCH + Duration::from_secs(500_000);
        let expires = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);

        let back = roundtrip(
            Response::data(b"hello".to_vec())
                .with_etag("etag-1")
                .with_modified(modified)
                .with_expires(expires)
                .with_must_revalidate(true),
        );

        assert!(back.error.is_none());
        assert_eq!(back.data.as_deref(), Some(b"hello".as_slice()));
        assert_eq!(back.etag.as_deref(), Some("etag-1"));
        assert_eq!(back.modified, Some(modified));
        assert_eq!(back.expires, Some(expires));
        assert!(back.must_revalidate);
        assert!(!back.no_content);
        assert!(!back.not_modified);
    }

    #[test]
    fn no_content_and_not_modified_roundtrip() {
        let back = roundtrip(Response::no_content());
        assert!(back.no_content);
        assert!(back.data.is_none());
        assert!(back.error.is_none());

        let back = roundtrip(Response::not_modified());
        assert!(back.not_modified);
        assert!(back.data.is_none());
    }

    #[test]
    fn empty_body_stays_present_with_absent_metadata() {
        let back = roundtrip(Response::data(Vec::new()));
        assert_eq!(back.data.as_deref(), Some([].as_slice()));
        assert!(back.modified.is_none());
        assert!(back.expires.is_none());
        assert!(back.etag.is_none());
        assert!(!back.must_revalidate);
    }

    #[test]
    fn pre_unix_epoch_timestamps_roundtrip() {
        let modified = SystemTime::UNIX_EPOCH - Duration::from_secs(1);
        let back = roundtrip(Response::data(Vec::new()).with_modified(modified));

        assert_eq!(back.modified, Some(modified));
    }

    #[test]
    fn error_roundtrips_and_success_is_coerced() {
        let retry_after = SystemTime::UNIX_EPOCH + Duration::from_secs(123_456);
        let back = roundtrip(
            Response::error(ErrorReason::RateLimit, "retry later").with_retry_after(retry_after),
        );
        let error = back.error.expect("error should be preserved");
        assert_eq!(error.reason, ErrorReason::RateLimit);
        assert_eq!(error.message, "retry later");
        assert_eq!(error.retry_after, Some(retry_after));
        assert!(back.data.is_none());

        let back = roundtrip(Response::error(ErrorReason::Success, "oops"));
        assert_eq!(back.error.expect("error should be preserved").reason, ErrorReason::Other);
    }
}
