//! Integration tests for image renderer request and run loop behavior.

use maplibre_native::{
    ImageRenderer, ImageRendererBuilder, MapLoadError, RunLoopHandle, Static, Tile,
};
use std::{
    cell::Cell,
    num::NonZeroU32,
    path::PathBuf,
    rc::Rc,
    thread,
    time::{Duration, Instant},
};

const RENDER_TIMEOUT: Duration = Duration::from_secs(5);

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(name)
}

fn static_renderer_with_size(size: u32) -> ImageRenderer<Static> {
    ImageRendererBuilder::new()
        .with_size(NonZeroU32::new(size).unwrap(), NonZeroU32::new(size).unwrap())
        .with_pixel_ratio(1.0)
        .build_static_renderer()
}

fn static_renderer() -> ImageRenderer<Static> {
    static_renderer_with_size(128)
}

fn tile_renderer() -> ImageRenderer<Tile> {
    ImageRendererBuilder::new()
        .with_size(NonZeroU32::new(128).unwrap(), NonZeroU32::new(128).unwrap())
        .with_pixel_ratio(1.0)
        .build_tile_renderer()
}

fn tick_until_ready(mut ready: impl FnMut() -> bool) {
    let run_loop = RunLoopHandle::current();
    let deadline = Instant::now() + RENDER_TIMEOUT;
    while !ready() {
        assert!(Instant::now() < deadline, "request did not complete within {RENDER_TIMEOUT:?}");
        run_loop.tick();
    }
}

#[test]
fn thread_run_loop_supports_worker_threads() {
    let handles = (0..2).map(|_| {
        thread::spawn(|| {
            let mut renderer = static_renderer();

            renderer
                .load_style_from_path(fixture_path("test-style.json"))
                .expect("test style path should be valid");
            let image = renderer
                .render_static(0.0, 0.0, 0.0, 0.0, 0.0)
                .expect("thread run loop should render");

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
    let mut first = static_renderer();
    let mut second = static_renderer();

    first
        .load_style_from_path(fixture_path("test-style.json"))
        .expect("test style path should be valid");
    second
        .load_style_from_path(fixture_path("test-style.json"))
        .expect("test style path should be valid");

    let first_request =
        first.submit_render_static(0.0, 0.0, 0.0, 0.0, 0.0).expect("first render should submit");
    let second_request =
        second.submit_render_static(0.0, 0.0, 0.0, 0.0, 0.0).expect("second render should submit");

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
fn load_style_from_json_str_renders() {
    let mut renderer = static_renderer();

    renderer.load_style_from_json_str(include_str!("fixtures/test-style.json"));

    let request = renderer
        .submit_render_static(0.0, 0.0, 0.0, 0.0, 0.0)
        .expect("JSON style render should submit");
    tick_until_ready(|| request.is_ready());

    let image = request.finish().expect("JSON style should render");
    assert_eq!(image.as_image().width(), 128);
    assert_eq!(image.as_image().height(), 128);
}

#[cfg(feature = "json")]
#[test]
fn load_style_from_json_value_loads() {
    let mut renderer = static_renderer();
    let value: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/test-style.json")).expect("valid JSON style");

    renderer
        .load_style_from_json_value(&value)
        .expect("style JSON should serialize")
        .wait()
        .expect("JSON style should load");
}

#[test]
fn style_load_request_polls_to_completion() {
    let mut renderer = static_renderer();

    let request = renderer
        .load_style_from_path(fixture_path("test-style.json"))
        .expect("test style path should be valid");
    tick_until_ready(|| request.is_ready());
    request.finish().expect("style should load");
}

#[test]
fn tile_render_request_renders() {
    let mut renderer = tile_renderer();

    renderer
        .load_style_from_path(fixture_path("test-style.json"))
        .expect("test style path should be valid");

    let request = renderer.submit_render_tile(0, 0, 0).expect("tile render should submit");

    tick_until_ready(|| request.is_ready());

    let image = request.finish().expect("tile renderer should render");
    assert_eq!(image.as_image().width(), 128);
    assert_eq!(image.as_image().height(), 128);
}

#[test]
fn style_load_request_reports_failure() {
    let mut renderer = static_renderer_with_size(64);

    let did_fail = Rc::new(Cell::new(false));
    renderer.map_observer().set_did_fail_loading_map_callback({
        let did_fail = Rc::clone(&did_fail);
        move |_error, _what| did_fail.set(true)
    });

    // A top-level JSON array is valid JSON but not a valid style document
    // (the style root must be an object), so MapLibre Native reports a load
    // failure through the observer rather than succeeding.
    let result = renderer.load_style_from_json_str("[]").wait();
    assert!(matches!(result, Err(MapLoadError::StyleParseError)), "unexpected result: {result:?}");
    assert!(did_fail.get(), "user style load failure callback should still be called");
}
