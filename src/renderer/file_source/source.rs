use std::fmt;
use std::sync::{Arc, Mutex};

use cxx::SharedPtr;

use crate::bridge::file_source::{
    forward_complete, register_rust_file_source, responder_cancel, responder_complete, ErrorReason,
    ForwardState, RawResourceRequest, RawResponse, RequestState,
};

use super::{FileSourceType, ResourceRequest, Response};

// SAFETY: These are opaque native handles. Rust only moves/clones the `SharedPtr`
// and hands it back to C++; it never dereferences the native state.
unsafe impl Send for RequestState {}
unsafe impl Sync for RequestState {}
unsafe impl Send for ForwardState {}
unsafe impl Sync for ForwardState {}

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
    /// You only need to implement this when this source is registered as
    /// [`FileSourceType::Database`](super::FileSourceType::Database). MapLibre
    /// Native calls this on database sources to forward responses fetched from
    /// another source into the cache.
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
    state: Arc<NativeRequest>,
}

impl Responder {
    fn new(state: Arc<NativeRequest>) -> Self {
        Self { state }
    }

    /// Deliver `response`. No-op if the request was already cancelled.
    pub fn complete(self, response: Response) {
        if let Some(state) = self.state.take() {
            responder_complete(state, &response.into_ffi());
        }
    }
}

impl Drop for Responder {
    fn drop(&mut self) {
        if let Some(state) = self.state.take() {
            let error = Response::error(
                ErrorReason::Other,
                "Rust FileSource responder dropped without completing",
            );
            responder_complete(state, &error.into_ffi());
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
    state: Option<SharedPtr<ForwardState>>,
}

impl ForwardCompletion {
    fn new(state: SharedPtr<ForwardState>) -> Self {
        Self { state: Some(state) }
    }

    /// Notify MapLibre Native that the cache write finished.
    pub fn complete(mut self) {
        if let Some(state) = self.state.take() {
            forward_complete(state);
        }
    }
}

impl Drop for ForwardCompletion {
    fn drop(&mut self) {
        if let Some(state) = self.state.take() {
            forward_complete(state);
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

/// Rust-side handle to one native (C++) request, shared by its [`Responder`] and
/// [`RequestHandleFfi`].
struct NativeRequest {
    state: Mutex<Option<SharedPtr<RequestState>>>,
}

impl NativeRequest {
    fn new(state: SharedPtr<RequestState>) -> Self {
        Self { state: Mutex::new(Some(state)) }
    }

    #[cfg(test)]
    fn empty_for_test() -> Self {
        Self { state: Mutex::new(None) }
    }

    /// Take the native state, unless already completed or cancelled.
    fn take(&self) -> Option<SharedPtr<RequestState>> {
        self.state.lock().expect("native request mutex should not be poisoned").take()
    }
}

impl fmt::Debug for NativeRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeRequest").finish_non_exhaustive()
    }
}

/// Opaque carrier for a [`RequestHandle`] passed back to C++.
pub(crate) struct RequestHandleFfi {
    state: Arc<NativeRequest>,
    cancel_hook: Mutex<Option<CancelHook>>,
}

impl RequestHandleFfi {
    fn new(handle: RequestHandle, state: Arc<NativeRequest>) -> Self {
        let cancel_hook = match handle {
            RequestHandle::Done => None,
            RequestHandle::Pending(hook) => Some(hook),
        };
        Self { state, cancel_hook: Mutex::new(cancel_hook) }
    }

    #[cfg(test)]
    fn is_pending(&self) -> bool {
        self.cancel_hook.lock().expect("cancel hook mutex should not be poisoned").is_some()
    }

    pub(crate) fn cancel(&self) {
        let Some(state) = self.state.take() else {
            return;
        };
        responder_cancel(state);
        if let Some(hook) =
            self.cancel_hook.lock().expect("cancel hook mutex should not be poisoned").take()
        {
            hook();
        }
    }
}

impl fmt::Debug for RequestHandleFfi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RequestHandleFfi").finish_non_exhaustive()
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
        native_state: SharedPtr<RequestState>,
    ) -> Box<RequestHandleFfi> {
        let request = ResourceRequest::from_ffi(request);
        let state = Arc::new(NativeRequest::new(native_state));
        let responder = Responder::new(Arc::clone(&state));
        Box::new(RequestHandleFfi::new(self.0.request(request, responder), state))
    }

    pub(crate) fn forward(
        &self,
        request: &RawResourceRequest,
        response: &RawResponse,
        completion: SharedPtr<ForwardState>,
    ) {
        let request = ResourceRequest::from_ffi(request);
        let completion = ForwardCompletion::new(completion);
        self.0.forward(request, Response::from_ffi(response), completion);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use super::{NativeRequest, RequestHandle, RequestHandleFfi};

    // A `NativeRequest` whose native state is already taken (completed/cancelled), so
    // `cancel` must neither call into C++ nor run the hook.
    fn consumed_request() -> Arc<NativeRequest> {
        Arc::new(NativeRequest::empty_for_test())
    }

    #[test]
    fn cancel_without_live_state_does_not_run_hook() {
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_hook = Arc::clone(&calls);
        let handle = RequestHandleFfi::new(
            RequestHandle::pending(move || {
                calls_hook.fetch_add(1, Ordering::SeqCst);
            }),
            consumed_request(),
        );
        assert!(handle.is_pending());
        handle.cancel();
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn done_request_handle_has_no_cancel_hook() {
        let handle = RequestHandleFfi::new(RequestHandle::Done, consumed_request());
        assert!(!handle.is_pending());
        handle.cancel();
    }
}
