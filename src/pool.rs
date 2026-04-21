//! Rendering pool with transparent tile prefetching.
//!
//! The pool serializes all `MapLibre` Native rendering on a single worker
//! thread (required for GPU safety) while prefetching tile data via `reqwest`
//! on the async side. Fetched tiles are written to a temporary `SQLite` cache
//! that `MapLibre`'s `DatabaseFileSource` reads from, eliminating network
//! latency from the render path.
//!
//! # Example
//!
//! ```no_run
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use maplibre_native::SingleThreadedRenderPool;
//! use std::path::PathBuf;
//!
//! let pool = SingleThreadedRenderPool::global_pool().await?;
//! let style = PathBuf::from("path/to/style.json");
//! let image = pool.render_tile(style, 2, 1, 1).await?;
//! image.as_image().save("tile.png")?;
//! # Ok(())
//! # }
//! ```

use std::path::PathBuf;
use std::sync::{mpsc, Arc};
use std::thread;

use tokio::sync::{oneshot, Mutex, OnceCell};

use crate::renderer::cache::{self, CacheError, ResourceKind, ResponseMeta, StyleInfo, TileCache};
use crate::renderer::{Image, ImageRendererBuilder, RenderingError};
use crate::ResourceOptions;

/// Expected number of unique tiles the bloom filter is sized for.
const BLOOM_EXPECTED_ITEMS: usize = 100_000;

/// Rendering request sent to the pool.
struct RenderRequest {
    style_path: PathBuf,
    z: u8,
    x: u32,
    y: u32,
    response: oneshot::Sender<Result<Image, SingleThreadedRenderPoolError>>,
}

/// Cached state for a loaded style: parsed source info + prefetched sprites.
#[derive(Debug)]
struct LoadedStyle {
    style_path: PathBuf,
    info: StyleInfo,
}

/// A thread-safe rendering pool with transparent tile prefetching.
///
/// Prevents segmentation faults by ensuring all rendering operations are handled
/// sequentially. Automatically loads and caches styles as needed.
///
/// Use [`SingleThreadedRenderPool::global_pool`] to access the shared instance.
#[derive(Debug, Clone)]
pub struct SingleThreadedRenderPool {
    rendering_requests: mpsc::Sender<RenderRequest>,
    cache: Arc<TileCache>,
    client: reqwest::Client,
    loaded_style: Arc<Mutex<Option<LoadedStyle>>>,
    /// Bloom filter tracking tiles already written to cache.
    /// False positives (skipped prefetch) are harmless - `MapLibre` falls back to network.
    fetched_tiles: Arc<fastbloom::AtomicBloomFilter>,
    /// Kept alive so the temp directory isn't deleted.
    _cache_dir: Arc<tempfile::TempDir>,
}

impl SingleThreadedRenderPool {
    /// Create a new rendering pool
    ///
    /// Purposely not public to prevent accidental misuse.
    pub(crate) async fn new() -> Result<Self, SingleThreadedRenderPoolError> {
        let cache_dir = tempfile::TempDir::with_prefix("mln-pool-")
            .map_err(SingleThreadedRenderPoolError::IOError)?;
        let cache_path = cache_dir.path().join("cache.sqlite");

        let cache = Arc::new(
            TileCache::new(&cache_path).await.map_err(SingleThreadedRenderPoolError::CacheError)?,
        );

        let path_for_thread = cache_path.clone();
        let (tx, rx) = mpsc::channel::<RenderRequest>();

        thread::spawn(move || {
            let resource_options = ResourceOptions::default().with_cache_path(path_for_thread);
            let mut renderer = ImageRendererBuilder::new()
                .with_resource_options(resource_options)
                .build_tile_renderer();
            let mut current_style: Option<PathBuf> = None;

            while let Ok(request) = rx.recv() {
                if current_style.as_ref() != Some(&request.style_path) {
                    if let Err(e) = renderer.load_style_from_path(&request.style_path) {
                        let _ =
                            request.response.send(Err(SingleThreadedRenderPoolError::IOError(e)));
                        continue;
                    }
                    current_style = Some(request.style_path.clone());
                }

                let result = renderer
                    .render_tile(request.z, request.x, request.y)
                    .map_err(SingleThreadedRenderPoolError::RenderingError);
                let _ = request.response.send(result);
            }
        });

        Ok(Self {
            rendering_requests: tx,
            cache,
            client: reqwest::Client::new(),
            loaded_style: Arc::new(Mutex::new(None)),
            fetched_tiles: Arc::new(
                fastbloom::AtomicBloomFilter::with_false_pos(0.001)
                    .expected_items(BLOOM_EXPECTED_ITEMS),
            ),
            _cache_dir: Arc::new(cache_dir),
        })
    }

    /// Render a tile, transparently prefetching tile data from the network.
    ///
    /// On the first call for a given style, the style JSON is parsed to
    /// discover tile source URLs and sprites. Sprites are fetched once and
    /// cached. On every call, the tile data for `(z, x, y)` is fetched via
    /// `reqwest` and written to the `SQLite` cache before the render request
    /// is sent to the worker thread.
    ///
    /// # Errors
    ///
    /// Returns an error if prefetching, cache access, or rendering fails.
    ///
    /// # Panics
    ///
    /// Panics if `x` or `y` exceeds `i32::MAX`.
    pub async fn render_tile(
        &self,
        style_path: PathBuf,
        z: u8,
        x: u32,
        y: u32,
    ) -> Result<Image, SingleThreadedRenderPoolError> {
        // --- Prefetch phase (async, on tokio) ---
        let zi = i32::from(z);
        let xi = i32::try_from(x).expect("tile x out of range");
        let yi = i32::try_from(y).expect("tile y out of range");

        // Fast path: bloom filter check is lock-free and O(1).
        // False positives skip a prefetch — harmless, `MapLibre` falls back to network.
        if !self.fetched_tiles.contains(&(zi, xi, yi)) {
            let style_info = self.ensure_style_loaded(&style_path).await?;
            let to_fetch = style_info.tile_urls(zi, xi, yi);

            if !to_fetch.is_empty() {
                let fetches: Vec<_> =
                    to_fetch.iter().map(|(_, url)| cache::http_fetch(&self.client, url)).collect();
                let results = futures::future::join_all(fetches).await;

                for ((url_template, _), result) in to_fetch.iter().zip(results) {
                    if let Ok(resp) = result {
                        let _ = self
                            .cache
                            .put_tile(url_template, 1, zi, xi, yi, &resp.data, &resp.meta)
                            .await;
                    }
                }
            }

            self.fetched_tiles.insert(&(zi, xi, yi));
        }

        // --- Render phase (sync, on worker thread) ---
        let (response_tx, response_rx) = oneshot::channel();

        self.rendering_requests
            .send(RenderRequest { style_path, z, x, y, response: response_tx })
            .map_err(|_| SingleThreadedRenderPoolError::FailedToSendRequest)?;

        response_rx.await.map_err(|_| SingleThreadedRenderPoolError::FailedToReceiveResponse)?
    }

    /// Ensure the style is parsed and its sources resolved.
    /// Returns the cached `StyleInfo`, fetching `TileJSON` endpoints if needed.
    async fn ensure_style_loaded(
        &self,
        style_path: &PathBuf,
    ) -> Result<StyleInfo, SingleThreadedRenderPoolError> {
        let mut guard = self.loaded_style.lock().await;

        if let Some(loaded) = guard.as_ref() {
            if &loaded.style_path == style_path {
                return Ok(loaded.info.clone());
            }
        }

        // Read and parse the style JSON
        let style_bytes =
            tokio::fs::read(style_path).await.map_err(SingleThreadedRenderPoolError::IOError)?;

        // Cache the style itself (as a file:// resource)
        if let Some(path_str) = style_path.to_str() {
            let url = format!("file://{path_str}");
            let _ = self
                .cache
                .put_resource(&url, ResourceKind::Style, &style_bytes, &ResponseMeta::default())
                .await;
        }

        let info = StyleInfo::from_style_json(&style_bytes, &self.client)
            .await
            .map_err(SingleThreadedRenderPoolError::CacheError)?;

        // Prefetch sprites once
        if let Some(sprite_base) = &info.sprite_base {
            let png_url = format!("{sprite_base}.png");
            let json_url = format!("{sprite_base}.json");

            let (png_res, json_res) = futures::future::join(
                cache::http_fetch(&self.client, &png_url),
                cache::http_fetch(&self.client, &json_url),
            )
            .await;

            if let Ok(resp) = png_res {
                let _ = self
                    .cache
                    .put_resource(&png_url, ResourceKind::SpriteImage, &resp.data, &resp.meta)
                    .await;
            }
            if let Ok(resp) = json_res {
                let _ = self
                    .cache
                    .put_resource(&json_url, ResourceKind::SpriteJSON, &resp.data, &resp.meta)
                    .await;
            }
        }

        *guard = Some(LoadedStyle { style_path: style_path.clone(), info: info.clone() });

        Ok(info)
    }

    /// Get the global rendering pool instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the pool has not been initialized and
    /// initialization fails.
    pub async fn global_pool(
    ) -> Result<&'static SingleThreadedRenderPool, SingleThreadedRenderPoolError> {
        static GLOBAL_POOL: OnceCell<SingleThreadedRenderPool> = OnceCell::const_new();

        GLOBAL_POOL.get_or_try_init(SingleThreadedRenderPool::new).await
    }
}

/// Errors that can occur in the single-threaded render pool.
#[derive(thiserror::Error, Debug)]
pub enum SingleThreadedRenderPoolError {
    /// An I/O error occurred during rendering operations.
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    /// A rendering error occurred during map rendering.
    #[error(transparent)]
    RenderingError(#[from] RenderingError),

    /// A cache operation failed.
    #[error(transparent)]
    CacheError(#[from] CacheError),

    /// Failed to send a rendering request to the worker thread.
    #[error("Failed to send request to rendering thread")]
    FailedToSendRequest,

    /// Failed to receive a response from the worker thread.
    #[error("Failed to receive response from rendering thread")]
    FailedToReceiveResponse,
}
