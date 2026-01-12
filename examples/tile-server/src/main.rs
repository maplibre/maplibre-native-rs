use axum::{
    extract::{Path, Query},
    http::{header, StatusCode},
    response::{Html, Response},
    routing::get,
    Router,
};
use clap::Parser;
use maplibre_native::{MultiThreadedRenderPool, SingleThreadedRenderPool};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;

/// Command-line arguments for the tile server
#[derive(Parser, Debug)]
struct Args {
    /// Use multi-threaded pool with multiple worker processes
    #[arg(short = 'm', long = "multi-threaded")]
    multi_threaded: bool,

    /// Number of worker processes (only used with --multi-threaded)
    #[arg(short = 'w', long = "workers", default_value_t = 4)]
    workers: usize,

    /// Server address to bind to
    #[arg(short = 'a', long = "addr", default_value = "127.0.0.1:3000")]
    addr: String,
}

/// Query parameters for tile requests
#[derive(Deserialize)]
struct TileQuery {
    /// Image format: "png" (default) or "webp"
    #[serde(default = "default_format")]
    format: String,
}

fn default_format() -> String {
    "png".to_string()
}

enum RenderPool {
    SingleThreaded,
    MultiThreaded(MultiThreadedRenderPool),
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join(name)
}
async fn rendered_style_tile(
    axum::extract::State(pool): axum::extract::State<Arc<RenderPool>>,
    Path((z, x, y)): Path<(u8, u32, u32)>,
    Query(query): Query<TileQuery>,
) -> Result<Response, StatusCode> {
    let style = fixture_path("maplibre_demo.json");
    assert!(style.is_file());

    let image = match pool.as_ref() {
        RenderPool::SingleThreaded => SingleThreadedRenderPool::global_pool()
            .render_tile(style, z + 1, x, y)
            .await
            .map_err(|e| {
                eprintln!("Render error: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?,
        RenderPool::MultiThreaded(multi_pool) => {
            multi_pool.render_tile(style, z, x, y).await.map_err(|e| {
                eprintln!("Render error: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
        }
    };

    // Determine output format and content type
    let (image_format, content_type) = match query.format.to_lowercase().as_str() {
        "webp" => (image::ImageFormat::WebP, "image/webp"),
        _ => (image::ImageFormat::Png, "image/png"), // Default to PNG
    };

    // Encode image in requested format
    let mut image_bytes = Vec::new();
    image
        .as_image()
        .write_to(&mut std::io::Cursor::new(&mut image_bytes), image_format)
        .map_err(|e| {
            eprintln!("Image encoding error ({:?}): {:?}", image_format, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let body = axum::body::Body::from(image_bytes);
    Ok(Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "max-age=3600")
        .body(body)
        .unwrap())
}

async fn index() -> Html<&'static str> {
    Html(include_str!("index.html"))
}

#[tokio::main]
async fn main() {
    // Check if this is a worker process
    if MultiThreadedRenderPool::is_worker_process() {
        // Run as worker process
        if let Err(e) = MultiThreadedRenderPool::run_worker() {
            eprintln!("Worker process error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    let args = Args::parse();

    let pool = if args.multi_threaded {
        println!("Using multi-threaded pool with {} workers", args.workers);
        let multi_pool = MultiThreadedRenderPool::new(args.workers)
            .expect("Failed to create multi-threaded pool");
        Arc::new(RenderPool::MultiThreaded(multi_pool))
    } else {
        println!("Using single-threaded pool");
        Arc::new(RenderPool::SingleThreaded)
    };

    let addr = &args.addr;
    println!("Server running on http://{addr}");
    println!("Usage:");
    println!("  - PNG tiles: http://{addr}/{{z}}/{{x}}/{{y}}");
    println!("  - WebP tiles: http://{addr}/{{z}}/{{x}}/{{y}}?format=webp");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let app = Router::new()
        .route("/", get(index))
        .route("/{z}/{x}/{y}", get(rendered_style_tile))
        .with_state(pool);
    axum::serve(listener, app).await.unwrap();
}
