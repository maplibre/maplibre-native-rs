//! Integration tests for image renderer request and run loop behavior.

use maplibre_native::{
    CameraUpdate, EdgeInsets, ImageRendererBuilder, LatLng, LatLngBounds, RunLoopHandle,
};
use std::{
    num::NonZeroU32,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

const RENDER_TIMEOUT: Duration = Duration::from_secs(5);

fn test_camera() -> CameraUpdate {
    CameraUpdate::new().center(LatLng { lat: 0.0, lng: 0.0 }).zoom(0.0).bearing(0.0).pitch(0.0)
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(name)
}

fn tick_until_ready(mut ready: impl FnMut() -> bool) {
    let run_loop = RunLoopHandle::current();
    let deadline = Instant::now() + RENDER_TIMEOUT;
    while !ready() {
        assert!(
            Instant::now() < deadline,
            "render request did not complete within {RENDER_TIMEOUT:?}"
        );
        run_loop.tick();
    }
}

#[test]
fn thread_run_loop_supports_worker_threads() {
    let handles = (0..2).map(|_| {
        thread::spawn(|| {
            let mut renderer = ImageRendererBuilder::new()
                .with_size(NonZeroU32::new(128).unwrap(), NonZeroU32::new(128).unwrap())
                .with_pixel_ratio(1.0)
                .build_static_renderer();

            renderer
                .load_style_from_path(fixture_path("test-style.json"))
                .expect("test style should load");
            let image =
                renderer.render_static(&test_camera()).expect("thread run loop should render");

            assert_eq!(image.as_image().width(), 128);
            assert_eq!(image.as_image().height(), 128);
        })
    });

    for handle in handles {
        handle.join().expect("render thread should not panic");
    }
}

#[test]
fn multiple_renderers_render_on_single_thread() {
    let mut first = ImageRendererBuilder::new()
        .with_size(NonZeroU32::new(128).unwrap(), NonZeroU32::new(128).unwrap())
        .with_pixel_ratio(1.0)
        .build_static_renderer();
    let mut second = ImageRendererBuilder::new()
        .with_size(NonZeroU32::new(128).unwrap(), NonZeroU32::new(128).unwrap())
        .with_pixel_ratio(1.0)
        .build_static_renderer();

    first.load_style_from_path(fixture_path("test-style.json")).expect("test style should load");
    second.load_style_from_path(fixture_path("test-style.json")).expect("test style should load");

    let first_request =
        first.submit_render_static(&test_camera()).expect("first render should submit");
    let second_request =
        second.submit_render_static(&test_camera()).expect("second render should submit");

    // Both requests are driven from the same thread-local run loop.
    tick_until_ready(|| first_request.is_ready() && second_request.is_ready());

    let first_image = first_request.finish().expect("first renderer should render");
    let second_image = second_request.finish().expect("second renderer should render");

    assert_eq!(first_image.as_image().width(), 128);
    assert_eq!(first_image.as_image().height(), 128);
    assert_eq!(second_image.as_image().width(), 128);
    assert_eq!(second_image.as_image().height(), 128);
}

#[test]
fn load_style_from_json_renders() {
    let mut renderer = ImageRendererBuilder::new()
        .with_size(NonZeroU32::new(128).unwrap(), NonZeroU32::new(128).unwrap())
        .with_pixel_ratio(1.0)
        .build_static_renderer();

    renderer.load_style_from_json(include_str!("fixtures/test-style.json"));

    let request =
        renderer.submit_render_static(&test_camera()).expect("JSON style render should submit");
    tick_until_ready(|| request.is_ready());

    let image = request.finish().expect("JSON style should render");
    assert_eq!(image.as_image().width(), 128);
    assert_eq!(image.as_image().height(), 128);
}

#[test]
fn tile_render_request_renders() {
    let mut renderer = ImageRendererBuilder::new()
        .with_size(NonZeroU32::new(128).unwrap(), NonZeroU32::new(128).unwrap())
        .with_pixel_ratio(1.0)
        .build_tile_renderer();

    renderer.load_style_from_path(fixture_path("test-style.json")).expect("test style should load");

    let request = renderer.submit_render_tile(0, 0, 0).expect("tile render should submit");

    tick_until_ready(|| request.is_ready());

    let image = request.finish().expect("tile renderer should render");
    assert_eq!(image.as_image().width(), 128);
    assert_eq!(image.as_image().height(), 128);
}

#[test]
fn camera_for_bounds_renders() {
    let mut renderer = ImageRendererBuilder::new()
        .with_size(NonZeroU32::new(128).unwrap(), NonZeroU32::new(128).unwrap())
        .with_pixel_ratio(1.0)
        .build_static_renderer();

    renderer.load_style_from_path(fixture_path("test-style.json")).expect("test style should load");

    let bounds = LatLngBounds {
        southwest: LatLng { lat: -10.0, lng: -10.0 },
        northeast: LatLng { lat: 10.0, lng: 10.0 },
    };
    let camera = renderer.camera_for_bounds(bounds, Some(EdgeInsets::all(8.0)), 0.0, 0.0);
    let request = renderer.submit_render_static(&camera).expect("bounds-fit render should submit");
    tick_until_ready(|| request.is_ready());

    let image = request.finish().expect("bounds-fit renderer should render");
    assert_eq!(image.as_image().width(), 128);
    assert_eq!(image.as_image().height(), 128);
}

#[test]
fn dropping_render_request_before_renderer_is_safe() {
    let mut renderer = ImageRendererBuilder::new()
        .with_size(NonZeroU32::new(128).unwrap(), NonZeroU32::new(128).unwrap())
        .with_pixel_ratio(1.0)
        .build_static_renderer();

    renderer.load_style_from_path(fixture_path("test-style.json")).expect("test style should load");

    let request = renderer.submit_render_static(&test_camera()).expect("render should submit");
    drop(request);
    drop(renderer);
}
