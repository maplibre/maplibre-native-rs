//! MapLibre Native run loop handle.

use crate::bridge::ffi;
use std::marker::PhantomData;

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

    /// Ticks the current thread's run loop once.
    #[allow(clippy::unused_self, reason = "method syntax ties ticking to a run-loop handle")]
    pub fn tick(&self) {
        ffi::currentThreadRunLoopTick();
    }
}
