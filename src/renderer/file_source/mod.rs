//! Custom MapLibre Native file sources.
//!
//! Implement [`FileSource`] and register it for a [`FileSourceType`] — for
//! example, [`Network`](FileSourceType::Network) to fetch resources or
//! [`Database`](FileSourceType::Database) to cache them.
//!
//! For network-based or otherwise async sources, prefer `TokioFileSource`.
//! Use [`FileSource`] directly for synchronous responses, custom runtimes,
//! or explicit cancellation control.

mod request;
mod response;
mod source;
#[cfg(feature = "tokio")]
mod tokio;

use std::time::{Duration, SystemTime};

pub use crate::bridge::file_source::{ErrorReason, FileSourceType, ResourceKind};
pub use request::{LoadingMethods, Priority, ResourceRequest, StoragePolicy, TileRequest, Usage};
pub use response::{Error, Response};
pub use source::{
    register_file_source, CancelHook, FileSource, ForwardCompletion, RequestHandle, Responder,
};
pub(crate) use source::{BoxedFileSource, RequestHandleFfi};
#[cfg(feature = "tokio")]
pub use tokio::{
    register_tokio_file_source, register_tokio_file_source_with_handle, TokioFileSource,
};

// Epoch-seconds conversions shared by the `Response` and `ResourceRequest`
fn to_epoch(time: Option<SystemTime>) -> (bool, i64) {
    let Some(time) = time else {
        return (false, 0);
    };
    let seconds = match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => i64::try_from(duration.as_secs()).unwrap_or(i64::MAX),
        Err(error) => -i64::try_from(error.duration().as_secs()).unwrap_or(i64::MAX),
    };
    (true, seconds)
}

fn from_epoch(present: bool, epoch_s: i64) -> Option<SystemTime> {
    if !present {
        return None;
    }
    let duration = Duration::from_secs(epoch_s.unsigned_abs());
    if epoch_s >= 0 {
        SystemTime::UNIX_EPOCH.checked_add(duration)
    } else {
        SystemTime::UNIX_EPOCH.checked_sub(duration)
    }
}
