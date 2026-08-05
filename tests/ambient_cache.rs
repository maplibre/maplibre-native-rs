//! Tests that `with_maximum_cache_size` reaches MapLibre Native's ambient cache.

use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use maplibre_native::file_source::Response;
use maplibre_native::{
    register_tokio_file_source, CameraUpdate, FileSourceType, ImageRendererBuilder, LatLng,
    ResourceKind, ResourceOptions, ResourceRequest, Size, TokioFileSource,
};

const INLINE_STYLE: &str = r#"{
    "version": 8,
    "name": "ambient-cache-test",
    "sources": {},
    "layers": [
        { "id": "bg", "type": "background", "paint": { "background-color": "rgb(0, 255, 0)" } }
    ]
}"#;

const ETAG: &str = "v1-etag";
const STYLE_URL: &str = "https://example.invalid/ambient-cache-style.json";

struct NetworkSource {
    plain_requests: Arc<AtomicUsize>,
    revalidations: Arc<AtomicUsize>,
}

impl TokioFileSource for NetworkSource {
    fn can_request(&self, request: &ResourceRequest) -> bool {
        request.kind == ResourceKind::Style && request.url.ends_with("/ambient-cache-style.json")
    }

    async fn request(&self, request: ResourceRequest) -> Response {
        tokio::task::yield_now().await;
        if request.prior_etag.as_deref() == Some(ETAG) {
            self.revalidations.fetch_add(1, Ordering::SeqCst);
        } else {
            self.plain_requests.fetch_add(1, Ordering::SeqCst);
        }
        Response::data(INLINE_STYLE.as_bytes().to_vec())
            .with_etag(ETAG)
            .with_expires(SystemTime::UNIX_EPOCH + Duration::from_secs(4_000_000_000))
    }
}

fn render_once(cache_path: PathBuf, maximum_cache_size: u64) {
    thread::spawn(move || {
        let options = ResourceOptions::default()
            .with_cache_path(cache_path)
            .with_maximum_cache_size(maximum_cache_size);
        let mut renderer = ImageRendererBuilder::new()
            .with_size(NonZeroU32::new(64).unwrap(), NonZeroU32::new(64).unwrap())
            .with_pixel_ratio(1.0)
            .with_resource_options(options)
            .build_static_renderer();

        let url: url::Url = STYLE_URL.parse().unwrap();
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
    })
    .join()
    .expect("renderer thread should not panic");
}

fn temp_cache_db(name: &str) -> PathBuf {
    let path = std::env::temp_dir()
        .join(format!("mln-ambient-cache-{}-{name}.sqlite", std::process::id()));
    let _ = std::fs::remove_file(&path);
    path
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn maximum_cache_size_reaches_the_ambient_cache() {
    let plain_requests = Arc::new(AtomicUsize::new(0));
    let revalidations = Arc::new(AtomicUsize::new(0));
    register_tokio_file_source(
        FileSourceType::Network,
        NetworkSource {
            plain_requests: plain_requests.clone(),
            revalidations: revalidations.clone(),
        },
    );

    let disabled = temp_cache_db("disabled");
    for _ in 0..3 {
        render_once(disabled.clone(), 0);
        thread::sleep(Duration::from_millis(300));
    }
    assert_eq!(
        revalidations.load(Ordering::SeqCst),
        0,
        "a disabled ambient cache must never serve a response"
    );
    assert_eq!(
        plain_requests.load(Ordering::SeqCst),
        3,
        "every render against a disabled cache should fetch from the network"
    );

    let enabled = temp_cache_db("enabled");
    let deadline = Instant::now() + Duration::from_secs(10);
    while revalidations.load(Ordering::SeqCst) == 0 {
        render_once(enabled.clone(), 10 * 1024 * 1024);
        assert!(Instant::now() < deadline, "the ambient cache never served the style");
        thread::sleep(Duration::from_millis(200));
    }
}
