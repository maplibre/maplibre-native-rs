use std::fmt;

use crate::bridge::file_source::{
    forward_complete, register_rust_file_source, responder_complete, ErrorReason,
    RawResourceRequest, RawResponse,
};

use super::{FileSourceType, ResourceRequest, Response};

/// A custom resource provider registered for one [`FileSourceType`].
pub trait FileSource: Send + Sync + 'static {
    /// Whether this source can serve `request`.
    fn can_request(&self, request: &ResourceRequest) -> bool;

    /// Begin serving `request`.
    ///
    /// Deliver the [`Response`] through `responder` — inline, or later from
    /// another thread. Return [`RequestHandle::Done`] when completed inline, or
    /// [`RequestHandle::pending`] with a cancel hook while work is in flight.
    fn request(&self, request: ResourceRequest, responder: Responder) -> RequestHandle;

    /// Store `response` for `request` (cache write).
    ///
    /// MapLibre Native calls this on [`FileSourceType::Database`] sources. Complete
    /// `completion` when the write is done. The default ignores writes and
    /// completes immediately.
    fn forward(&self, request: ResourceRequest, response: Response, completion: ForwardCompletion) {
        let _ = (request, response);
        completion.complete();
    }
}

/// A cancellation hook for an in-flight request.
///
/// MapLibre Native invokes it when it drops the request before completion. It must be
/// idempotent and safe to race with [`Responder::complete`].
pub type CancelHook = Box<dyn Fn() + Send + Sync + 'static>;

/// Request lifetime handle returned from [`FileSource::request`].
/// Describes whether a [`FileSource::request`] is complete or cancellable.
pub enum RequestHandle {
    /// The request completed synchronously; no cancellation hook is needed.
    Done,
    /// The request is in flight; run the hook if MapLibre Native cancels it.
    Pending(CancelHook),
}

impl RequestHandle {
    /// Create a pending request from a cancellation hook.
    pub fn pending(cancel: impl Fn() + Send + Sync + 'static) -> Self {
        Self::Pending(Box::new(cancel))
    }
}

impl fmt::Debug for RequestHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Done => f.write_str("RequestHandle::Done"),
            Self::Pending(_) => f.write_str("RequestHandle::Pending(..)"),
        }
    }
}

/// A one-shot handle for delivering a [`Response`] to MapLibre Native.
///
/// Call [`complete`](Self::complete) at most once. Dropping without completing
/// reports an error unless the request was already cancelled.
#[must_use = "complete() the Responder, or MapLibre Native receives a dropped-responder error"]
pub struct Responder {
    token: Option<usize>,
}

impl Responder {
    fn new(token: usize) -> Self {
        Self { token: Some(token) }
    }

    /// Deliver `response`. No-op if the request was already cancelled.
    pub fn complete(mut self, response: Response) {
        let Some(token) = self.token.take() else {
            return;
        };
        responder_complete(token, &response.into_ffi());
    }
}

impl Drop for Responder {
    fn drop(&mut self) {
        if let Some(token) = self.token.take() {
            let error = Response::error(
                ErrorReason::Other,
                "Rust FileSource responder dropped without completing",
            );
            responder_complete(token, &error.into_ffi());
        }
    }
}

impl fmt::Debug for Responder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Responder").finish_non_exhaustive()
    }
}

/// A one-shot handle for finishing a cache write.
///
/// MapLibre Native waits for this before treating [`FileSource::forward`] as complete.
/// Dropping it also completes the write.
pub struct ForwardCompletion {
    token: Option<usize>,
}

impl ForwardCompletion {
    fn new(token: usize) -> Self {
        Self { token: Some(token) }
    }

    /// Notify MapLibre Native that the cache write finished.
    pub fn complete(mut self) {
        let Some(token) = self.token.take() else {
            return;
        };
        forward_complete(token);
    }
}

impl Drop for ForwardCompletion {
    fn drop(&mut self) {
        if let Some(token) = self.token.take() {
            forward_complete(token);
        }
    }
}

impl fmt::Debug for ForwardCompletion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ForwardCompletion").finish_non_exhaustive()
    }
}

/// Register `source` as the process-global implementation of `source_type`.
///
/// Register before constructing renderers. Re-registering does not update
/// already-cached MapLibre Native file source instances.
pub fn register_file_source<S: FileSource>(source_type: FileSourceType, source: S) {
    register_rust_file_source(source_type, Box::new(BoxedFileSource(Box::new(source))));
}

/// Opaque carrier for a [`RequestHandle`] passed back to C++.
pub(crate) struct RequestHandleFfi(RequestHandle);

impl RequestHandleFfi {
    fn new(handle: RequestHandle) -> Self {
        Self(handle)
    }

    #[cfg(test)]
    fn is_pending(&self) -> bool {
        matches!(self.0, RequestHandle::Pending(_))
    }

    pub(crate) fn cancel(&self) {
        if let RequestHandle::Pending(hook) = &self.0 {
            hook();
        }
    }
}

impl fmt::Debug for RequestHandleFfi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

// cxx bridge glue (not called directly by user code)

/// Opaque wrapper handed to C++.
pub(crate) struct BoxedFileSource(Box<dyn FileSource>);

impl fmt::Debug for BoxedFileSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoxedFileSource").finish_non_exhaustive()
    }
}

impl BoxedFileSource {
    pub(crate) fn can_request(&self, request: &RawResourceRequest) -> bool {
        let request = ResourceRequest::from_ffi(request);
        self.0.can_request(&request)
    }

    #[allow(clippy::unnecessary_box_returns)]
    pub(crate) fn request(
        &self,
        request: &RawResourceRequest,
        responder_token: usize,
    ) -> Box<RequestHandleFfi> {
        let request = ResourceRequest::from_ffi(request);
        let responder = Responder::new(responder_token);
        Box::new(RequestHandleFfi::new(self.0.request(request, responder)))
    }

    pub(crate) fn forward(
        &self,
        request: &RawResourceRequest,
        response: &RawResponse,
        forward_token: usize,
    ) {
        let request = ResourceRequest::from_ffi(request);
        let completion = ForwardCompletion::new(forward_token);
        self.0.forward(request, Response::from_ffi(response), completion);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use super::{RequestHandle, RequestHandleFfi};

    #[test]
    fn pending_request_handle_runs_cancel_hook() {
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_hook = Arc::clone(&calls);
        let handle = RequestHandleFfi::new(RequestHandle::pending(move || {
            calls_hook.fetch_add(1, Ordering::SeqCst);
        }));
        assert!(handle.is_pending());
        handle.cancel();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn done_request_handle_has_no_cancel_hook() {
        let handle = RequestHandleFfi::new(RequestHandle::Done);
        assert!(!handle.is_pending());
        handle.cancel();
    }
}
