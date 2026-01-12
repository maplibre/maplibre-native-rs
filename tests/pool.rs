#![cfg(feature = "pool")]

use std::path::PathBuf;

use insta::{assert_binary_snapshot, assert_debug_snapshot};
use maplibre_native::{Image, MultiThreadedRenderPool, SingleThreadedRenderPool};

fn image_to_png_bytes(image: &Image) -> Vec<u8> {
    let mut png_bytes = Vec::new();
    image
        .as_image()
        .write_to(
            &mut std::io::Cursor::new(&mut png_bytes),
            image::ImageFormat::Png,
        )
        .unwrap();
    png_bytes
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[tokio::test]
async fn sequential_errors_dont_break_pool() {
    let pool = SingleThreadedRenderPool::global_pool();

    for i in 0..3 {
        let path = PathBuf::from(format!("invalid_{i}.json"));
        let result = pool.render_tile(path, 0, i, 0).await;
        assert!(result.is_err());
    }
    let working_style = fixture_path("test-style.json");
    let result = pool
        .render_tile(working_style.clone(), 1, 0, 0)
        .await
        .unwrap();
    assert_binary_snapshot!(".png", image_to_png_bytes(&result));
}

#[tokio::test]
async fn large_coordinates_handled() {
    let pool = SingleThreadedRenderPool::global_pool();
    let style = fixture_path("test-style.json");

    let result = pool.render_tile(style, 1, 32767, 32767).await.unwrap();
    assert_binary_snapshot!(".png", image_to_png_bytes(&result));
}

#[tokio::test]
async fn io_errors() {
    let pool = SingleThreadedRenderPool::global_pool();

    let result = pool
        .render_tile(PathBuf::from(""), 0, 0, 0)
        .await
        .unwrap_err();
    assert_debug_snapshot!(result, @r#"
    IOError(
        Custom {
            kind: NotFound,
            error: "Path  is not a file",
        },
    )
    "#);

    let result = pool
        .render_tile(PathBuf::from("missing.json"), 0, 0, 0)
        .await
        .unwrap_err();
    assert_debug_snapshot!(result,@r#"
    IOError(
        Custom {
            kind: NotFound,
            error: "Path missing.json is not a file",
        },
    )
    "#);

    let result = pool
        .render_tile(PathBuf::from("/dev/null/style.json"), 0, 0, 0)
        .await
        .unwrap_err();
    assert_debug_snapshot!(result, @r#"
    IOError(
        Custom {
            kind: NotFound,
            error: "Path /dev/null/style.json is not a file",
        },
    )
    "#);
}

#[tokio::test]
async fn style_switching_() {
    let pool = SingleThreadedRenderPool::global_pool();
    let style1 = fixture_path("test-style.json");
    let style2 = fixture_path("test-style-alt.json");

    let result = pool.render_tile(style1.clone(), 1, 0, 0).await.unwrap();
    assert_binary_snapshot!(".png", image_to_png_bytes(&result));
    let result = pool.render_tile(style1.clone(), 1, 0, 1).await.unwrap();
    assert_binary_snapshot!(".png", image_to_png_bytes(&result));
    let result = pool.render_tile(style2.clone(), 1, 0, 0).await.unwrap();
    assert_binary_snapshot!(".png", image_to_png_bytes(&result));
}

#[tokio::test(flavor = "multi_thread")]
async fn concurrent_rendering_does_not_segfault() {
    let style_path = fixture_path("test-style.json");

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let path = style_path.clone();
            tokio::spawn(async move {
                let pool = SingleThreadedRenderPool::global_pool();
                pool.render_tile(path, 0, i, 0).await
            })
        })
        .collect();

    // All requests should complete without panic
    for handle in handles {
        let _ = handle.await.unwrap();
    }
}

#[tokio::test]
async fn various_zoom_levels() {
    let pool = SingleThreadedRenderPool::global_pool();
    let style_path = fixture_path("test-style.json");

    for zoom in [0, 5, 10, 15] {
        let result = pool.render_tile(style_path.clone(), zoom, 0, 0).await;
        // Should handle all zoom levels without crashing
        let _ = result;
    }
}

// MultiThreadedRenderPool tests

#[tokio::test]
async fn multi_threaded_basic_render() {
    // Skip if running as worker
    if MultiThreadedRenderPool::is_worker_process() {
        return;
    }

    let pool = MultiThreadedRenderPool::new(2).unwrap();
    let style_path = fixture_path("test-style.json");

    let result = pool.render_tile(style_path, 1, 0, 0).await.unwrap();
    assert_binary_snapshot!(".png", image_to_png_bytes(&result));
}

#[tokio::test(flavor = "multi_thread")]
async fn multi_threaded_concurrent_rendering() {
    // Skip if running as worker
    if MultiThreadedRenderPool::is_worker_process() {
        return;
    }

    let pool = MultiThreadedRenderPool::new(2).unwrap();
    let style_path = fixture_path("test-style.json");

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let path = style_path.clone();
            let pool = pool.clone();
            tokio::spawn(async move { pool.render_tile(path, 0, i, 0).await })
        })
        .collect();

    // All requests should complete without panic
    for handle in handles {
        let _ = handle.await.unwrap();
    }
}

#[tokio::test]
async fn multi_threaded_io_errors() {
    // Skip if running as worker
    if MultiThreadedRenderPool::is_worker_process() {
        return;
    }

    let pool = MultiThreadedRenderPool::new(1).unwrap();

    let result = pool
        .render_tile(PathBuf::from("missing.json"), 0, 0, 0)
        .await
        .unwrap_err();
    assert_debug_snapshot!(result, @r#"
    WorkerError(
        "Failed to load style: Path missing.json is not a file",
    )
    "#);
}

#[tokio::test]
async fn multi_threaded_parallel_workload() {
    // Skip if running as worker
    if MultiThreadedRenderPool::is_worker_process() {
        return;
    }

    let pool = MultiThreadedRenderPool::new(4).unwrap();
    let style_path = fixture_path("test-style.json");

    // Render multiple tiles in parallel
    let futures: Vec<_> = (0..8)
        .map(|i| {
            let path = style_path.clone();
            let pool = pool.clone();
            async move { pool.render_tile(path, 1, i, 0).await }
        })
        .collect();

    let results = futures::future::join_all(futures).await;

    // All should succeed
    for result in results {
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn multi_threaded_style_switching() {
    // Skip if running as worker
    if MultiThreadedRenderPool::is_worker_process() {
        return;
    }

    let pool = MultiThreadedRenderPool::new(1).unwrap();
    let style1 = fixture_path("test-style.json");
    let style2 = fixture_path("test-style-alt.json");

    let result = pool.render_tile(style1.clone(), 1, 0, 0).await.unwrap();
    assert_binary_snapshot!(".png", image_to_png_bytes(&result));

    let result = pool.render_tile(style2.clone(), 1, 0, 0).await.unwrap();
    assert_binary_snapshot!(".png", image_to_png_bytes(&result));

    let result = pool.render_tile(style1.clone(), 1, 0, 1).await.unwrap();
    assert_binary_snapshot!(".png", image_to_png_bytes(&result));
}
