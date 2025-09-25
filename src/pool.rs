//! Simple rendering pool for thread-safe [MapLibre Native](https://maplibre.org/projects/native/) rendering.
//!
//! This module provides a minimal thread-safe rendering pool that prevents
//! segmentation faults when used concurrently.
//!
//! # Example
//!
//! ```no_run
//! # async fn example() {
//! use maplibre_native::pool::SingleThreadedRenderPool;
//! use std::path::PathBuf;
//!
//! // Get the global pool instance
//! let pool = SingleThreadedRenderPool::global_pool();
//!
//! // Render a tile with a MapLibre style
//! let style_path = PathBuf::from("path/to/style.json");
//! let image = pool.render_tile(style_path.clone(), 10, 512, 384).await.unwrap();
//!
//! // The pool automatically handles style caching - subsequent renders
//! // with the same style will be faster
//! let another_tile = pool.render_tile(style_path.clone(), 10, 513, 384).await.unwrap();
//! # }
//! ```

use std::path::PathBuf;
use std::sync::{mpsc, LazyLock};
use std::thread;

use tokio::sync::oneshot;

use crate::renderer::{Image, ImageRenderer, ImageRendererOptions, RenderingError, Tile};

/// Rendering request sent to the pool.
struct RenderRequest {
    style_path: PathBuf,
    z: u8,
    x: u32,
    y: u32,
    response: oneshot::Sender<Result<Image, PoolError>>,
}

/// A thread-safe rendering pool that serializes [MapLibre Native](https://maplibre.org/projects/native/) tile rendering
/// operations through a single worker thread.
///
/// Prevents segmentation faults by ensuring all rendering operations are handled
/// sequentially. Automatically loads and caches styles as needed.
///
/// Use [`SingleThreadedRenderPool::global_pool`] to access the shared instance.
pub struct SingleThreadedRenderPool {
    rendering_requests: mpsc::Sender<RenderRequest>,
}

impl SingleThreadedRenderPool {
    /// Create a new rendering pool
    ///
    /// Purposely not public to prevent accidental misuse.
    pub(crate) fn new() -> Self {
        let (tx, rx) = mpsc::channel::<RenderRequest>();

        thread::spawn(move || {
            let mut renderer = ImageRendererOptions::default().build_tile_renderer();
            let mut current_style: Option<PathBuf> = None;

            while let Ok(request) = rx.recv() {
                // Load style if different from current
                if current_style.as_ref() != Some(&request.style_path) {
                    if let Err(e) = renderer.load_style_from_path(&request.style_path) {
                        let _ = request.response.send(Err(PoolError::IOError(e)));
                        continue;
                    }
                    current_style = Some(request.style_path.clone());
                }
                // TODO: handle style on disk changing content^

                // Render the tile
                let result = renderer
                    .render_tile(request.z, request.x, request.y)
                    .map_err(PoolError::RenderingError);
                let _ = request.response.send(result);
            }
        });

        Self {
            rendering_requests: tx,
        }
    }

    /// Render an encoded tile [`Image`] asynchronously in a centralised pool
    ///
    /// # Errors
    ///
    /// If the rendering fails, the response channel is dropped, or the request fails to send.
    pub async fn render_tile(
        &self,
        style_path: PathBuf,
        z: u8,
        x: u32,
        y: u32,
    ) -> Result<Image, PoolError> {
        let (response_tx, response_rx) = oneshot::channel();

        self.rendering_requests
            .send(RenderRequest {
                style_path,
                z,
                x,
                y,
                response: response_tx,
            })
            .map_err(|_| PoolError::FailedToSendRequest)?;

        response_rx
            .await
            .map_err(|_| PoolError::FailedToReceiveResponse)?
    }

    /// Get the global rendering pool instance.
    #[must_use]
    pub fn global_pool() -> &'static SingleThreadedRenderPool {
        static GLOBAL_POOL: LazyLock<SingleThreadedRenderPool> =
            LazyLock::new(SingleThreadedRenderPool::new);

        &GLOBAL_POOL
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PoolError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    RenderingError(#[from] RenderingError),

    #[error("Failed to send request to rendering thread")]
    FailedToSendRequest,

    #[error("Failed to receive response from rendering thread")]
    FailedToReceiveResponse,
}
