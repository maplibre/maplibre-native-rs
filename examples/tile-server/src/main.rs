use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{Html, Response},
    routing::get,
    Router,
};
use maplibre_native::SingleThreadedRenderPool;
use std::path::PathBuf;

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
    Path((z, x, y)): Path<(u8, u32, u32)>,
) -> Result<Response, StatusCode> {
    let style = fixture_path("maplibre_demo.json");
    assert!(style.is_file());
    let image = SingleThreadedRenderPool::global_pool()
        .render_tile(style, z, x, y)
        .await
        .map_err(|e| dbg!(e))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut png_bytes = Vec::new();
    image
        .as_image()
        .write_to(
            &mut std::io::Cursor::new(&mut png_bytes),
            image::ImageFormat::Png,
        )
        .map_err(|e| dbg!(e))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let body = axum::body::Body::from(png_bytes);
    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "image/png")
        .body(body)
        .unwrap())
}

async fn index() -> Html<&'static str> {
    Html(include_str!("index.html"))
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:3000";
    println!("Server running on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let app = Router::new()
        .route("/", get(index))
        .route("/:z/:x/:y", get(rendered_style_tile));
    axum::serve(listener, app).await.unwrap();
}
