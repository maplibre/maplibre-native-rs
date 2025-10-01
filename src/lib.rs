#![doc = include_str!("../README.md")]
#![warn(missing_debug_implementations)]

mod renderer;
pub use renderer::*;

#[cfg(feature = "pool")]
mod pool;
#[cfg(feature = "pool")]
pub use pool::{SingleThreadedRenderPool, SingleThreadedRenderPoolError};
