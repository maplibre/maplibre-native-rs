// FIXME: Remove this before merging
#![allow(unused)]

mod renderer;
pub use renderer::*;

#[cfg(feature = "pool")]
pub mod pool;
#[cfg(feature = "pool")]
pub use pool::SingleThreadedRenderPool;
