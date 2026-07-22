//! Integration tests for image renderer request and run loop behavior.

use std::cell::Cell;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};

use maplibre_native::{
    CameraUpdate, Color, Continuous, EdgeInsets, FillLayer, GeoJson, GeoJsonSource, ImageRenderer,
    ImageRendererBuilder, LatLng, LatLngBounds, MapLoadErrorKind, RunLoopHandle, Static, Tile,
};

const RENDER_TIMEOUT: Duration = Duration::from_secs(5);

fn test_camera() -> CameraUpdate {
    CameraUpdate::new().center(LatLng { lat: 0.0, lng: 0.0 }).zoom(0.0).bearing(0.0).pitch(0.0)
}

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
fn continuous_renderer_requests_frame_after_update() {
    let mut renderer: ImageRenderer<Continuous> = ImageRendererBuilder::new()
        .with_size(NonZeroU32::new(128).unwrap(), NonZeroU32::new(128).unwrap())
        .with_pixel_ratio(1.0)
        .build_continuous_renderer();
    let requested = Rc::new(Cell::new(false));
    renderer.set_render_requested_callback({
        let requested = requested.clone();
        move || requested.set(true)
    });

    renderer.update_camera(&CameraUpdate::new().zoom(1.0));
    tick_until_ready(|| requested.get());
}

#[test]
fn thread_run_loop_supports_worker_threads() {
    let handles = (0..2).map(|_| {
        thread::spawn(|| {
            let mut renderer = static_renderer();

            renderer
                .load_style_from_path(fixture_path("test-style.json"))
                .expect("test style path should be valid");
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
    let mut first = static_renderer();
    let mut second = static_renderer();

    first
        .load_style_from_path(fixture_path("test-style.json"))
        .expect("test style path should be valid");
    second
        .load_style_from_path(fixture_path("test-style.json"))
        .expect("test style path should be valid");

    let first_request =
        first.submit_render_static(&test_camera()).expect("first render should submit");
    let second_request =
        second.submit_render_static(&test_camera()).expect("second render should submit");

    // Both requests are already submitted before waiting, so they are driven
    // from the same thread-local run loop.
    let first_image = first_request.wait().expect("first renderer should render");
    let second_image = second_request.wait().expect("second renderer should render");

    assert_eq!(first_image.as_image().width(), 128);
    assert_eq!(first_image.as_image().height(), 128);
    assert_eq!(second_image.as_image().width(), 128);
    assert_eq!(second_image.as_image().height(), 128);
}

#[test]
fn load_style_from_json_str_renders() {
    let mut renderer = static_renderer();

    renderer.load_style_from_json_str(include_str!("fixtures/test-style.json"));

    let request =
        renderer.submit_render_static(&test_camera()).expect("JSON style render should submit");

    let image = request.wait().expect("JSON style should render");
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

    let image = request.wait().expect("tile renderer should render");
    assert_eq!(image.as_image().width(), 128);
    assert_eq!(image.as_image().height(), 128);
}

#[test]
fn camera_for_bounds_renders() {
    let mut renderer = static_renderer();

    renderer
        .load_style_from_path(fixture_path("test-style.json"))
        .expect("test style path should be valid");

    let bounds = LatLngBounds {
        southwest: LatLng { lat: -10.0, lng: -10.0 },
        northeast: LatLng { lat: 10.0, lng: 10.0 },
    };
    let camera = renderer.camera_for_bounds(bounds, Some(EdgeInsets::all(8.0)), 0.0, 0.0);
    let request = renderer.submit_render_static(&camera).expect("bounds-fit render should submit");

    let image = request.wait().expect("bounds-fit renderer should render");
    assert_eq!(image.as_image().width(), 128);
    assert_eq!(image.as_image().height(), 128);
}

#[test]
fn camera_for_lat_lngs_renders() {
    let mut renderer = static_renderer();

    renderer
        .load_style_from_path(fixture_path("test-style.json"))
        .expect("test style path should be valid");

    let lat_lngs = [
        LatLng { lat: -10.0, lng: -10.0 },
        LatLng { lat: -10.0, lng: 10.0 },
        LatLng { lat: 10.0, lng: 10.0 },
        LatLng { lat: 10.0, lng: -10.0 },
    ];
    let camera = renderer
        .camera_for_lat_lngs(&lat_lngs, Some(EdgeInsets::all(8.0)), 0.0, 0.0)
        .expect("non-empty coordinates should produce a camera");
    let request =
        renderer.submit_render_static(&camera).expect("coordinates-fit render should submit");

    let image = request.wait().expect("coordinates-fit renderer should render");
    assert_eq!(image.as_image().width(), 128);
    assert_eq!(image.as_image().height(), 128);
}

#[test]
fn camera_for_lat_lngs_returns_none_for_empty_input() {
    let mut renderer = static_renderer();
    renderer
        .load_style_from_path(fixture_path("test-style.json"))
        .expect("test style path should be valid");

    assert!(renderer.camera_for_lat_lngs(&[], None, 0.0, 0.0).is_none());
}

#[test]
fn camera_for_geojson_renders() {
    let mut renderer = static_renderer();

    renderer
        .load_style_from_path(fixture_path("test-style.json"))
        .expect("test style path should be valid");

    let geojson = r#"{
        "type": "Polygon",
        "coordinates": [[[-10.0, -10.0], [10.0, -10.0], [10.0, 10.0], [-10.0, 10.0], [-10.0, -10.0]]]
    }"#
    .parse::<GeoJson>()
    .expect("inline GeoJSON should parse");

    let camera = renderer
        .camera_for_geojson(&geojson, Some(EdgeInsets::all(8.0)), 0.0, 0.0)
        .expect("non-empty geometry should produce a camera");
    let request =
        renderer.submit_render_static(&camera).expect("geometry-fit render should submit");

    let image = request.wait().expect("geometry-fit renderer should render");
    assert_eq!(image.as_image().width(), 128);
    assert_eq!(image.as_image().height(), 128);
}

#[test]
fn camera_for_geojson_returns_none_for_empty_geometry() {
    let mut renderer = static_renderer();
    renderer
        .load_style_from_path(fixture_path("test-style.json"))
        .expect("test style path should be valid");

    let empty = r#"{ "type": "FeatureCollection", "features": [] }"#
        .parse::<GeoJson>()
        .expect("inline GeoJSON should parse");

    assert!(renderer.camera_for_geojson(&empty, None, 0.0, 0.0).is_none());
}

#[test]
fn camera_for_geojson_matches_bounds() {
    let mut renderer = static_renderer();
    renderer
        .load_style_from_path(fixture_path("test-style.json"))
        .expect("test style path should be valid");

    let polygon = r#"{
        "type": "Polygon",
        "coordinates": [[[-10.0, -10.0], [10.0, -10.0], [10.0, 10.0], [-10.0, 10.0], [-10.0, -10.0]]]
    }"#
    .parse::<GeoJson>()
    .expect("inline GeoJSON should parse");

    // Draw the polygon so the rendered output actually depends on the camera.
    {
        let mut style = renderer.style();
        let mut source = GeoJsonSource::new("poly");
        source.set_geojson(&polygon);
        let source_id = style.add_source(source).expect("source should be added");
        let mut fill = FillLayer::new("poly-fill", &source_id);
        fill.set_fill_color(Color::rgb(1.0, 0.0, 0.0));
        style.add_layer(fill).expect("fill layer should be added");
    }

    let render = |renderer: &mut ImageRenderer<Static>, camera: &CameraUpdate| {
        let request = renderer.submit_render_static(camera).expect("render should submit");
        request.wait().expect("render should succeed").as_image().clone()
    };

    let geo_camera = renderer
        .camera_for_geojson(&polygon, Some(EdgeInsets::all(8.0)), 0.0, 0.0)
        .expect("non-empty geometry should produce a camera");
    let geo_image = render(&mut renderer, &geo_camera);

    let bounds = LatLngBounds {
        southwest: LatLng { lat: -10.0, lng: -10.0 },
        northeast: LatLng { lat: 10.0, lng: 10.0 },
    };
    let bounds_camera = renderer.camera_for_bounds(bounds, Some(EdgeInsets::all(8.0)), 0.0, 0.0);
    let bounds_image = render(&mut renderer, &bounds_camera);

    // Fitting an axis-aligned rectangle by geometry must match fitting its bounds.
    assert_eq!(geo_image, bounds_image);
}

#[test]
fn style_load_request_reports_failure() {
    let mut renderer = static_renderer_with_size(64);

    let did_fail = Rc::new(Cell::new(false));
    renderer.map_observer().set_did_fail_loading_map_callback({
        let did_fail = Rc::clone(&did_fail);
        move |_error| did_fail.set(true)
    });

    // A top-level JSON array is valid JSON but not a valid style document
    // (the style root must be an object), so MapLibre Native reports a load
    // failure through the observer rather than succeeding.
    let result = renderer.load_style_from_json_str("[]").wait();
    let error = result.expect_err("invalid style should fail to load");
    assert_eq!(error.kind, MapLoadErrorKind::StyleParse);
    assert!(!error.message.is_empty(), "style load error should include a message");
    assert!(did_fail.get(), "user style load failure callback should still be called");
}

#[test]
fn dropping_render_request_before_renderer_is_safe() {
    let mut renderer = static_renderer();

    renderer
        .load_style_from_path(fixture_path("test-style.json"))
        .expect("test style path should be valid");

    let request = renderer.submit_render_static(&test_camera()).expect("render should submit");
    drop(request);
    drop(renderer);
}
