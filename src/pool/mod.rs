/// This module provides a thread-safe pool of maplibre-native renderers.
/// Due to the nature of the library, it is not possible to create a multi-threaded pool.
/// Instead, we provide a single-threaded pool implementation and a multi threaded pool via inter process communication.

mod single_threaded;
pub use single_threaded::*;