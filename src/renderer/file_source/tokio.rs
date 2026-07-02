//! Thin adapter for `tokio`-native async file sources.

use std::{future::Future, sync::Arc};

use super::{
    register_file_source, FileSource, FileSourceType, ForwardCompletion, RequestHandle,
    ResourceRequest, Responder, Response,
};

/// An async, `tokio`-native file source.
///
/// Prefer this trait for network-backed sources and other async resource loading.
/// The adapter spawns each request and aborts the task on cancellation.
pub trait TokioFileSource: Send + Sync + 'static {
    /// Whether this source can serve `request`.
    fn can_request(&self, request: &ResourceRequest) -> bool;

    /// Serve a request asynchronously.
    fn request(&self, request: ResourceRequest) -> impl Future<Output = Response> + Send;

    /// Store a response asynchronously (cache write).
    ///
    /// You only need to implement this when this source is registered as
    /// [`FileSourceType::Database`](super::FileSourceType::Database). MapLibre
    /// Native calls this on database sources to forward responses fetched from
    /// another source into the cache. The adapter completes MapLibre Native's
    /// cache-write callback after this future finishes.
    fn forward(
        &self,
        request: ResourceRequest,
        response: Response,
    ) -> impl Future<Output = ()> + Send {
        async move {
            let _ = (request, response);
        }
    }
}

struct Adapter<S> {
    source: Arc<S>,
    handle: ::tokio::runtime::Handle,
}

impl<S: TokioFileSource> FileSource for Adapter<S> {
    fn can_request(&self, request: &ResourceRequest) -> bool {
        self.source.can_request(request)
    }

    fn request(&self, request: ResourceRequest, responder: Responder) -> RequestHandle {
        let source = Arc::clone(&self.source);
        let abort = self
            .handle
            .spawn(async move {
                let response = source.request(request).await;
                responder.complete(response); // no-op if already cancelled
            })
            .abort_handle();
        RequestHandle::pending(move || abort.abort())
    }

    fn forward(&self, request: ResourceRequest, response: Response, completion: ForwardCompletion) {
        let source = Arc::clone(&self.source);
        self.handle.spawn(async move {
            source.forward(request, response).await;
            completion.complete();
        });
    }
}

/// Register a `tokio`-native file source using `handle`.
///
/// Keep the Tokio runtime alive while renderers may use this source.
/// Register before constructing renderers. Re-registering does not update
/// already-cached MapLibre Native file source instances.
pub fn register_tokio_file_source_with_handle<S: TokioFileSource>(
    source_type: FileSourceType,
    handle: ::tokio::runtime::Handle,
    source: S,
) {
    let adapter = Adapter { source: Arc::new(source), handle };
    register_file_source(source_type, adapter);
}

/// Register a `tokio`-native file source using the current runtime.
///
/// Keep the Tokio runtime alive while renderers may use this source.
/// Register before constructing renderers. Re-registering does not update
/// already-cached MapLibre Native file source instances.
///
/// # Panics
///
/// Panics if called outside a Tokio runtime.
pub fn register_tokio_file_source<S: TokioFileSource>(source_type: FileSourceType, source: S) {
    register_tokio_file_source_with_handle(
        source_type,
        ::tokio::runtime::Handle::current(),
        source,
    );
}
