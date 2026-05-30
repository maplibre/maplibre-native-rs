//! Tile rendering server example.
//!
//! Serves rendered tiles from a MapLibre style over HTTP. It shows one simple
//! way to use [`ImageRenderer`](maplibre_native::ImageRenderer) from an async,
//! multi-threaded server.
//!
//! Run with `cargo run -p tile-server`, then open <http://127.0.0.1:3000>.

use std::{
    io::Cursor,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, Response},
    routing::get,
    Router,
};
use clap::Parser;
use maplibre_native::{ImageRenderer, ImageRendererBuilder, RenderingError, Tile};
use tokio::sync::{mpsc, oneshot};

const WORKER_QUEUE_SIZE: usize = 128;
const DEFAULT_STYLE_URL: &str = "https://demotiles.maplibre.org/style.json";

#[derive(Parser, Debug)]
struct Args {
    /// Style JSON URL to render.
    #[arg(long, default_value = DEFAULT_STYLE_URL)]
    style: url::Url,
}

/// A render job: a tile coordinate plus a channel to return the encoded PNG.
struct RenderJob {
    z: u8,
    x: u32,
    y: u32,
    response: oneshot::Sender<Result<Vec<u8>, RenderError>>,
}

struct SimpleRenderPool {
    workers: Vec<mpsc::Sender<RenderJob>>,
    next_worker: AtomicUsize,
}

impl SimpleRenderPool {
    fn new(style_url: url::Url, worker_count: usize) -> Self {
        assert!(worker_count > 0, "render pool must have at least one worker");

        let workers = (0..worker_count)
            .map(|_| {
                let (tx, rx) = mpsc::channel(WORKER_QUEUE_SIZE);
                let style_url = style_url.clone();
                thread::spawn(move || render_worker(&style_url, rx));
                tx
            })
            .collect();

        Self { workers, next_worker: AtomicUsize::new(0) }
    }

    async fn render_tile(&self, z: u8, x: u32, y: u32) -> Result<Vec<u8>, RenderError> {
        let (response_tx, response_rx) = oneshot::channel();

        // round-robin
        let worker = self.next_worker.fetch_add(1, Ordering::Relaxed) % self.workers.len();

        self.workers[worker]
            .send(RenderJob { z, x, y, response: response_tx })
            .await
            .map_err(|_| RenderError::WorkerUnavailable)?;

        response_rx.await.map_err(|_| RenderError::WorkerUnavailable)?
    }
}

fn render_worker(style_url: &url::Url, mut rx: mpsc::Receiver<RenderJob>) {
    let mut renderer = ImageRendererBuilder::default().with_pixel_ratio(2.0).build_tile_renderer();
    renderer.load_style_from_url(style_url);

    while let Some(job) = rx.blocking_recv() {
        let result = render_and_encode(&mut renderer, &job);
        let _ = job.response.send(result);
    }
}

fn render_and_encode(
    renderer: &mut ImageRenderer<Tile>,
    job: &RenderJob,
) -> Result<Vec<u8>, RenderError> {
    let image = renderer.render_tile(job.z, job.x, job.y)?;
    let mut png = Vec::new();
    image.as_image().write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)?;
    Ok(png)
}

#[derive(thiserror::Error, Debug)]
enum RenderError {
    #[error(transparent)]
    Rendering(#[from] RenderingError),
    #[error(transparent)]
    Encode(#[from] image::ImageError),
    #[error("render worker is unavailable")]
    WorkerUnavailable,
}

async fn rendered_style_tile(
    State(pool): State<Arc<SimpleRenderPool>>,
    Path((z, x, y)): Path<(u8, u32, u32)>,
) -> Result<Response, StatusCode> {
    let png = pool.render_tile(z, x, y).await.map_err(|e| {
        eprintln!("failed to render tile {z}/{x}/{y}: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "image/png")
        .header(header::CACHE_CONTROL, "max-age=3600")
        .body(Body::from(png))
        .expect("valid response"))
}

async fn index() -> Html<&'static str> {
    Html(include_str!("index.html"))
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let worker_count = thread::available_parallelism().map_or(1, usize::from);
    let pool = Arc::new(SimpleRenderPool::new(args.style.clone(), worker_count));

    let addr = "127.0.0.1:3000";
    println!("Server running on http://{addr}");
    println!("Rendering {} with {worker_count} workers", args.style);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let app = Router::new()
        .route("/", get(index))
        .route("/{z}/{x}/{y}", get(rendered_style_tile))
        .with_state(pool);
    axum::serve(listener, app).await.unwrap();
}
