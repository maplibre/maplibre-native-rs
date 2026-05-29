#![doc = include_str!("../README.md")]

pub(crate) mod bridge;
mod renderer;
mod style;
pub use renderer::*;
pub use style::*;

#[cfg(feature = "pool")]
mod pool;
#[cfg(feature = "pool")]
pub use pool::{SingleThreadedRenderPool, SingleThreadedRenderPoolError};

#[cfg(feature = "wgpu")]
pub use binding_generator::*;
