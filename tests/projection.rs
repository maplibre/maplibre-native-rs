//! Integration tests for native projection behavior in static rendering mode.

use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use maplibre_native::{ImageRendererBuilder, MapProjectionType};

fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .expect("projection test lock should be available")
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn alpha_at_corners(image: &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>) -> [u8; 4] {
    let max_x = image.width().saturating_sub(1);
    let max_y = image.height().saturating_sub(1);
    [
        image.get_pixel(0, 0)[3],
        image.get_pixel(max_x, 0)[3],
        image.get_pixel(0, max_y)[3],
        image.get_pixel(max_x, max_y)[3],
    ]
}

#[test]
fn projection_setter_roundtrips() {
    let mut renderer = ImageRendererBuilder::new()
        .with_size(
            NonZeroU32::new(128).expect("constant non-zero width"),
            NonZeroU32::new(128).expect("constant non-zero height"),
        )
        .with_projection(MapProjectionType::Mercator)
        .build_static_renderer();

    assert_eq!(renderer.projection(), MapProjectionType::Mercator);
    renderer.set_projection(MapProjectionType::Globe);
    assert_eq!(renderer.projection(), MapProjectionType::Globe);
    renderer.set_projection(MapProjectionType::Mercator);
    assert_eq!(renderer.projection(), MapProjectionType::Mercator);
}

#[test]
fn projection_mode_changes_render_output() {
    let _guard = test_lock();

    let mut renderer = ImageRendererBuilder::new()
        .with_size(
            NonZeroU32::new(128).expect("constant non-zero width"),
            NonZeroU32::new(128).expect("constant non-zero height"),
        )
        .with_projection(MapProjectionType::Mercator)
        .build_static_renderer();
    renderer
        .load_style_from_path(fixture_path("test-style.json"))
        .expect("fixture style should load");

    renderer.set_projection(MapProjectionType::Mercator);
    let _ = renderer
        .render_static(0.0, 0.0, 1.5, 0.0, 0.0)
        .expect("warmup render should succeed");
    let mercator = renderer
        .render_static(0.0, 0.0, 1.5, 0.0, 0.0)
        .expect("mercator render should succeed");
    let mercator_corners = alpha_at_corners(mercator.as_image());

    renderer.set_projection(MapProjectionType::Globe);
    let _ = renderer
        .render_static(0.0, 0.0, 1.5, 0.0, 0.0)
        .expect("warmup globe render should succeed");
    let globe = renderer
        .render_static(0.0, 0.0, 1.5, 0.0, 0.0)
        .expect("globe render should succeed");
    let globe_corners = alpha_at_corners(globe.as_image());

    assert_ne!(mercator_corners, globe_corners);
    assert!(globe_corners
        .iter()
        .zip(mercator_corners.iter())
        .any(|(globe_alpha, mercator_alpha)| globe_alpha < mercator_alpha));
}
