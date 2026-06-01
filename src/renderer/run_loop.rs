//! MapLibre Native run loop handle.

use std::marker::PhantomData;

use crate::bridge::ffi;

/// Handle to the current thread's MapLibre Native run loop.
///
/// Use [`tick`](Self::tick) to advance pending render requests on this thread.
#[derive(Debug)]
pub struct RunLoopHandle {
    // Makes this handle !Send and !Sync: MapLibre Native run loops are thread-affine.
    // This marker keeps the handle on the thread that created it.
    _not_send: PhantomData<*mut ()>,
}

impl RunLoopHandle {
    /// Returns a handle to the current thread's MapLibre Native run loop.
    #[must_use]
    pub fn current() -> Self {
        Self { _not_send: PhantomData }
    }

    /// Ticks the current thread's run loop once (non-blocking).
    #[allow(clippy::unused_self, reason = "method syntax ties ticking to a run-loop handle")]
    pub fn tick(&self) {
        ffi::currentThreadRunLoopTick();
    }

    /// Blocks the calling thread, advancing the run loop until it is woken by
    /// pending work (a render or style-load completion).
    ///
    /// Unlike repeatedly calling [`tick`](Self::tick), this parks the thread on
    /// the run loop instead of busy-polling, so it does not spin a CPU core while
    /// waiting for asynchronous work to complete. The underlying primitive depends
    /// on the run-loop backend: the libuv backend processes one event turn
    /// (`UV_RUN_ONCE`), while the CoreFoundation (Darwin) backend runs until a
    /// completion calls [`stop`](Self::stop).
    #[allow(clippy::unused_self, reason = "method syntax ties waiting to a run-loop handle")]
    pub(crate) fn wait_for_event(&self) {
        ffi::currentThreadRunLoopWait();
    }

    /// Wakes a thread blocked in [`wait_for_event`](Self::wait_for_event).
    ///
    /// Only the CoreFoundation (Darwin) backend needs this; on the libuv backend
    /// it is a no-op, since [`wait_for_event`](Self::wait_for_event) returns on its
    /// own after one event turn.
    #[allow(clippy::unused_self, reason = "method syntax ties stopping to a run-loop handle")]
    pub(crate) fn stop(&self) {
        ffi::currentThreadRunLoopStop();
    }
}
