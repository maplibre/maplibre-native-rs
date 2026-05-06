#![doc = include_str!("../README.md")]

mod renderer;
pub use renderer::*;

#[cfg(feature = "pool")]
mod pool;
#[cfg(feature = "pool")]
pub use pool::{SingleThreadedRenderPool, SingleThreadedRenderPoolError};

#[cfg(feature = "wgpu")]
pub use binding_generator::*;
