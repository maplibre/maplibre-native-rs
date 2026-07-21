#![doc = include_str!("../README.md")]

pub(crate) mod bridge;
mod renderer;
mod style;
pub use renderer::*;
pub use style::*;
#[cfg(feature = "wgpu")]
pub use webgpu_shim::*;

#[cfg(feature = "ureq_http")]
mod http_bridge;
#[cfg(feature = "ureq_http")]
pub use http_bridge::init_rust_http_bridge;
#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
pub use android::init_android;
