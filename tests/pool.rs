//! Tests for SingleThreadedRenderPool

#![cfg(feature = "pool")]

use insta::assert_debug_snapshot;
use maplibre_native::pool::{PoolError, SingleThreadedRenderPool};
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn global_pool_is_singleton() {
    let pool1 = SingleThreadedRenderPool::global_pool();
    let pool2 = SingleThreadedRenderPool::global_pool();
    assert_eq!(pool1 as *const _, pool2 as *const _);
}

#[tokio::test]
async fn invalid_style_path_fails() {
    let pool = SingleThreadedRenderPool::global_pool();
    let result = pool
        .render_tile(PathBuf::from("/nonexistent.json"), 0, 0, 0)
        .await;

    assert!(result.is_err());
    assert!(matches!(result, Err(PoolError::IOError(_))));
}

#[tokio::test]
async fn concurrent_invalid_requests() {
    let invalid_path = PathBuf::from("missing.json");

    let handles: Vec<_> = (0..3)
        .map(|i| {
            let path = invalid_path.clone();
            tokio::spawn(async move {
                let pool = SingleThreadedRenderPool::global_pool();
                pool.render_tile(path, 0, i, 0).await.is_err()
            })
        })
        .collect();

    for handle in handles {
        assert!(handle.await.unwrap());
    }
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
    let path = PathBuf::from("test.json");

    let result = pool.render_tile(path, 15, 32767, 32767).await;
    assert!(result.is_err()); // Expected to fail gracefully
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
async fn different_path_errors() {
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
async fn basic_tile_rendering() {
    let pool = SingleThreadedRenderPool::global_pool();
    let style_path = fixture_path("test-style.json");

    let result = pool.render_tile(style_path, 0, 0, 0).await.unwrap();
    insta::assert_binary_snapshot!(".png", result.as_slice().to_vec());
}

#[tokio::test]
async fn style_switching() {
    let pool = SingleThreadedRenderPool::global_pool();
    let style1 = fixture_path("test-style.json");
    let style2 = fixture_path("test-style-alt.json");

    assert!(pool.render_tile(style1.clone(), 1, 0, 0).await.is_ok());
    assert!(pool.render_tile(style1.clone(), 1, 0, 1).await.is_ok());
    assert!(pool.render_tile(style2.clone(), 1, 0, 0).await.is_ok());
}

#[tokio::test(flavor = "multi_thread")]
async fn concurrent_rendering() {
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
