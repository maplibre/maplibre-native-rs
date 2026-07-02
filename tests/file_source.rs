//! Smoke test for a synchronous Rust `FileSource`.

use std::{
    num::NonZeroU32,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use maplibre_native::{
    file_source::Response, register_file_source, CameraUpdate, FileSource, FileSourceType,
    ImageRendererBuilder, LatLng, RequestHandle, ResourceKind, ResourceRequest, Responder, Size,
};

const INLINE_STYLE: &str = r#"{
    "version": 8,
    "name": "file-source-test",
    "sources": {},
    "layers": [
        { "id": "bg", "type": "background", "paint": { "background-color": "rgb(255, 128, 0)" } }
    ]
}"#;

struct InlineStyleSource {
    call_count: Arc<AtomicUsize>,
}

impl FileSource for InlineStyleSource {
    fn can_request(&self, request: &ResourceRequest) -> bool {
        request.kind == ResourceKind::Style && request.url.ends_with("/inline-style.json")
    }

    fn request(&self, request: ResourceRequest, responder: Responder) -> RequestHandle {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        if request.url.ends_with("/inline-style.json") {
            responder.complete(Response::data(INLINE_STYLE.as_bytes().to_vec()));
        } else {
            responder.complete(Response::no_content());
        }
        RequestHandle::Done
    }
}

#[test]
fn sync_file_source_serves_inline_style() {
    let call_count = Arc::new(AtomicUsize::new(0));
    register_file_source(
        FileSourceType::Network,
        InlineStyleSource { call_count: call_count.clone() },
    );

    let mut renderer = ImageRendererBuilder::new()
        .with_size(NonZeroU32::new(64).unwrap(), NonZeroU32::new(64).unwrap())
        .with_pixel_ratio(1.0)
        .build_static_renderer();

    let url: url::Url = "https://example.invalid/inline-style.json".parse().unwrap();
    renderer.load_style_from_url(&url);
    renderer.set_map_size(Size { width: 64, height: 64 });

    renderer
        .render_static(
            &CameraUpdate::new()
                .center(LatLng { lat: 0.0, lng: 0.0 })
                .zoom(0.0)
                .bearing(0.0)
                .pitch(0.0),
        )
        .expect("render should succeed");

    assert!(
        call_count.load(Ordering::SeqCst) >= 1,
        "the Rust file source should have been invoked for the style",
    );
}
