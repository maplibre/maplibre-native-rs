#![doc = include_str!("../README.md")]

mod renderer;
pub use renderer::*;
mod http_bridge;
pub use http_bridge::init_rust_http_bridge;
#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
pub use android::init_android_jvm;

#[cfg(feature = "pool")]
mod pool;
#[cfg(feature = "pool")]
pub use pool::{SingleThreadedRenderPool, SingleThreadedRenderPoolError};

#[cfg(feature = "wgpu")]
pub use binding_generator::*;
