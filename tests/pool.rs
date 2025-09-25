#![cfg(feature = "pool")]

use insta::{assert_binary_snapshot, assert_debug_snapshot};
use maplibre_native::pool::SingleThreadedRenderPool;
use std::path::PathBuf;

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
        let path = PathBuf::from(format!("invalid_{}.json", i));
        let result = pool.render_tile(path, 0, i, 0).await;
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn large_coordinates_handled() {
    let pool = SingleThreadedRenderPool::global_pool();
    let style = fixture_path("test-style.json");

    let result = pool.render_tile(style, 1, 32767, 32767).await;
    assert_debug_snapshot!(result.unwrap_err(), @r#"
    IOError(
        Custom {
            kind: NotFound,
            error: "Path missing.json is not a file",
        },
    )
    "#);
}

#[tokio::test]
async fn error_messages_are_consistent() {
    let pool = SingleThreadedRenderPool::global_pool();
    let result = pool
        .render_tile(PathBuf::from("missing.json"), 0, 0, 0)
        .await;

    assert_debug_snapshot!(result.unwrap_err(), @r#"
    IOError(
        Custom {
            kind: NotFound,
            error: "Path missing.json is not a file",
        },
    )
    "#);
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
    assert_binary_snapshot!(".png", result.as_slice().to_vec());
    let result = pool.render_tile(style1.clone(), 1, 0, 1).await.unwrap();
    assert_binary_snapshot!(".png", result.as_slice().to_vec());
    let result = pool.render_tile(style2.clone(), 1, 0, 0).await.unwrap();
    assert_binary_snapshot!(".png", result.as_slice().to_vec());
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
