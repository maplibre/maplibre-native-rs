//! Integration tests for image renderer request and run loop behavior.

use maplibre_native::{ImageRendererBuilder, RunLoopHandle};
use std::{num::NonZeroU32, path::PathBuf, thread};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(name)
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

    let run_loop = RunLoopHandle::current();
    let first_request =
        first.submit_render_static(0.0, 0.0, 0.0, 0.0, 0.0).expect("first render should submit");
    let second_request =
        second.submit_render_static(0.0, 0.0, 0.0, 0.0, 0.0).expect("second render should submit");

    while !first_request.is_ready() || !second_request.is_ready() {
        run_loop.tick();
    }

    let first_image = first_request.finish().expect("first renderer should render");
    let second_image = second_request.finish().expect("second renderer should render");

    assert_eq!(first_image.as_image().width(), 128);
    assert_eq!(first_image.as_image().height(), 128);
    assert_eq!(second_image.as_image().width(), 128);
    assert_eq!(second_image.as_image().height(), 128);
}

#[test]
fn tile_render_request_renders() {
    let mut renderer = ImageRendererBuilder::new()
        .with_size(NonZeroU32::new(128).unwrap(), NonZeroU32::new(128).unwrap())
        .with_pixel_ratio(1.0)
        .build_tile_renderer();

    renderer.load_style_from_path(fixture_path("test-style.json")).expect("test style should load");

    let run_loop = RunLoopHandle::current();
    let request = renderer.submit_render_tile(0, 0, 0).expect("tile render should submit");

    while !request.is_ready() {
        run_loop.tick();
    }

    let image = request.finish().expect("tile renderer should render");
    assert_eq!(image.as_image().width(), 128);
    assert_eq!(image.as_image().height(), 128);
}
