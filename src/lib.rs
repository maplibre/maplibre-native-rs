#![doc = include_str!("../README.md")]
#![deny(warnings)]

mod renderer;
pub use renderer::*;

#[cfg(feature = "pool")]
mod pool;
#[cfg(feature = "pool")]
pub use pool::{SingleThreadedRenderPool, SingleThreadedRenderPoolError};
