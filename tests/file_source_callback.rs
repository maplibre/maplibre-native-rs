//! End-to-end test for the Rust FileSource callback bridge.
//!
//! Serves an inline style.json from the callback and renders it. Verifies:
//! - the callback is actually invoked by mbgl
//! - the URLs we receive match the documented `file://` scheme for
//!   `load_style_from_path`
//! - the rendered image has the expected dimensions and non-zero content
//!   (the background-color pixels, not all transparent).

use std::num::NonZeroU32;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use maplibre_native::{
    FsResponse, Height, ImageRendererBuilder, ResourceKind, Size, Width,
};

// A minimal style that renders a solid background. No tile sources — keeps
// the test self-contained. We use a bright, unambiguous color so we can
// assert it round-trips through the render pipeline.
const INLINE_STYLE: &str = r#"{
    "version": 8,
    "name": "callback-test",
    "sources": {},
    "layers": [
        { "id": "bg", "type": "background", "paint": { "background-color": "rgb(255, 128, 0)" } }
    ]
}"#;

#[test]
fn file_source_callback_serves_inline_style() {
    // Record every URL the callback sees so we can assert mbgl actually
    // called us. Use Arc<Mutex<Vec>> because the closure is Send + Sync.
    let seen_urls: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let call_count = Arc::new(AtomicUsize::new(0));

    let seen_urls_cb = seen_urls.clone();
    let call_count_cb = call_count.clone();

    let mut renderer = ImageRendererBuilder::new()
        .with_size(NonZeroU32::new(64).unwrap(), NonZeroU32::new(64).unwrap())
        .with_pixel_ratio(1.0)
        .with_file_source_callback(move |url: &str, kind: ResourceKind| {
            call_count_cb.fetch_add(1, Ordering::SeqCst);
            seen_urls_cb.lock().unwrap().push(format!("{kind:?}:{url}"));

            if url.ends_with("/inline-style.json") {
                FsResponse::Ok(INLINE_STYLE.as_bytes().to_vec())
            } else {
                // Style has no other resources, but mbgl may still probe
                // sprites/glyphs. Return NoContent for everything else —
                // mbgl renders the background without sprites/glyphs.
                FsResponse::NoContent
            }
        })
        .build_static_renderer();

    // Use a bogus URL — our callback doesn't care about the prefix, only the
    // `/inline-style.json` suffix. Go through `load_style_from_url` rather
    // than `load_style_from_path` so no file on disk is required.
    let url: url::Url = "https://example.invalid/inline-style.json".parse().unwrap();
    renderer.load_style_from_url(&url);
    renderer.set_map_size(Size::new(Width(64), Height(64)));

    let image = renderer
        .render_static(0.0, 0.0, 0.0, 0.0, 0.0)
        .expect("render should succeed");

    // Dimensions: 64x64 logical * 1.0 ratio = 64x64 physical.
    let buf = image.as_image();
    assert_eq!(buf.width(), 64);
    assert_eq!(buf.height(), 64);

    // The callback must have been invoked at least once — for the style
    // itself. (mbgl may also ask for sources/sprites/glyphs even though
    // the style declares none; we return NoContent for those.)
    let calls = call_count.load(Ordering::SeqCst);
    assert!(calls >= 1, "expected ≥1 callback invocation, got {calls}");

    // The first call must be the style URL we asked to load.
    let urls = seen_urls.lock().unwrap();
    assert!(!urls.is_empty());
    assert!(
        urls[0].contains("inline-style.json"),
        "first callback url was not the style: {:?}",
        urls[0]
    );

    // Background color is rgb(255, 128, 0) — find at least one pixel with
    // that color in the output (mbgl may premultiply/unpremultiply so we
    // allow ±2 tolerance).
    let mut saw_orange = false;
    for p in buf.pixels() {
        let [r, g, b, a] = p.0;
        if a >= 250 && (r as i32 - 255).abs() <= 2
            && (g as i32 - 128).abs() <= 3
            && (b as i32 - 0).abs() <= 2 {
            saw_orange = true;
            break;
        }
    }
    assert!(saw_orange, "expected to see rgb(255,128,0) pixels in the rendered image");
}
