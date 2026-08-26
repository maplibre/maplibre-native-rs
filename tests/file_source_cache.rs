//! Tests cache forwarding, cache hits, and 304 revalidation with Rust sources.

use std::{
    collections::HashMap,
    future::Future,
    num::NonZeroU32,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    time::{Duration, SystemTime},
};

use maplibre_native::{
    file_source::{ErrorReason, Response},
    register_tokio_file_source, CameraUpdate, FileSourceType, ImageRendererBuilder, LatLng,
    ResourceKind, ResourceRequest, Size, TokioFileSource,
};

const INLINE_STYLE: &str = r#"{
    "version": 8,
    "name": "cache-test",
    "sources": {},
    "layers": [
        { "id": "bg", "type": "background", "paint": { "background-color": "rgb(255, 0, 0)" } }
    ]
}"#;

const ETAG: &str = "v1-etag";
const STYLE_URL: &str = "https://example.invalid/cache-style.json";

struct NetworkSource {
    hits: Arc<AtomicUsize>,
    revalidations: Arc<AtomicUsize>,
}

struct DatabaseSource {
    store: Arc<Mutex<HashMap<String, Response>>>,
    hits: Arc<AtomicUsize>,
    forwarded: mpsc::Sender<()>,
}

impl TokioFileSource for NetworkSource {
    fn can_request(&self, request: &ResourceRequest) -> bool {
        request.kind == ResourceKind::Style && request.url.ends_with("/cache-style.json")
    }

    async fn request(&self, request: ResourceRequest) -> Response {
        tokio::task::yield_now().await;
        self.hits.fetch_add(1, Ordering::SeqCst);
        if request.prior_data.is_some() {
            assert_eq!(request.prior_etag.as_deref(), Some(ETAG));
            self.revalidations.fetch_add(1, Ordering::SeqCst);
            Response::not_modified()
                .with_etag(ETAG)
                .with_must_revalidate(true)
                .with_expires(SystemTime::UNIX_EPOCH + Duration::from_secs(1))
        } else {
            Response::data(INLINE_STYLE.as_bytes().to_vec())
                .with_etag(ETAG)
                .with_must_revalidate(true)
                .with_expires(SystemTime::UNIX_EPOCH + Duration::from_secs(1))
        }
    }
}

impl TokioFileSource for DatabaseSource {
    fn can_request(&self, request: &ResourceRequest) -> bool {
        // Mirror MapLibre Native's own cache
        request.loading_methods.has_cache()
            && !request.url.starts_with("asset://")
            && !request.url.starts_with("file://")
    }

    fn request(&self, request: ResourceRequest) -> impl Future<Output = Response> {
        let cached = self.store.lock().unwrap().get(&request.url).cloned();
        std::future::ready(if let Some(response) = cached {
            self.hits.fetch_add(1, Ordering::SeqCst);
            response
        } else {
            // mbgl's DatabaseFileSource signals a cache miss with NotFound + noContent.
            let mut miss = Response::error(ErrorReason::NotFound, "cache miss");
            miss.no_content = true;
            miss
        })
    }

    async fn forward(&self, request: ResourceRequest, response: Response) {
        // Only cache successful bodies (skip errors and 304s).
        if response.error.is_none() && response.data.is_some() {
            self.store.lock().unwrap().insert(request.url, response);
        }
        let _ = self.forwarded.send(());
    }
}

/// Renders on a separate thread so a wedge can time out.
fn render_once(label: &'static str) {
    let (done_tx, done_rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let mut renderer = ImageRendererBuilder::new()
            .with_size(NonZeroU32::new(64).unwrap(), NonZeroU32::new(64).unwrap())
            .with_pixel_ratio(1.0)
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
        drop(renderer);
        let _ = done_tx.send(());
    });
    match done_rx.recv_timeout(Duration::from_secs(10)) {
        Ok(()) => handle.join().expect("renderer thread should not panic"),
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            handle.join().expect("renderer thread should not panic");
            panic!("{label}: renderer exited without reporting completion");
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!("{label}: render did not complete within the timeout")
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn database_cache_serves_and_revalidates_network_responses() {
    let network_hits = Arc::new(AtomicUsize::new(0));
    let revalidations = Arc::new(AtomicUsize::new(0));
    let cache_hits = Arc::new(AtomicUsize::new(0));
    let store: Arc<Mutex<HashMap<String, Response>>> = Arc::new(Mutex::new(HashMap::new()));
    let (forwarded_tx, forwarded_rx) = mpsc::channel();

    register_tokio_file_source(
        FileSourceType::Network,
        NetworkSource { hits: network_hits.clone(), revalidations: revalidations.clone() },
    );
    register_tokio_file_source(
        FileSourceType::Database,
        DatabaseSource { store: store.clone(), hits: cache_hits.clone(), forwarded: forwarded_tx },
    );

    render_once("initial load");

    // Wait for the cache write that mbgl forwards after the network response.
    forwarded_rx.recv_timeout(Duration::from_secs(5)).expect("the response should be forwarded");

    assert_eq!(
        network_hits.load(Ordering::SeqCst),
        1,
        "first render should fetch from the network"
    );
    assert_eq!(cache_hits.load(Ordering::SeqCst), 0, "first render is a cache miss");

    let stored = store.lock().unwrap().get(STYLE_URL).cloned();
    let stored = stored.expect("the network response should have been forwarded into the cache");
    assert_eq!(
        stored.data.as_deref().map(<[u8]>::len),
        Some(INLINE_STYLE.len()),
        "stored body should be the style bytes",
    );
    assert_eq!(stored.etag.as_deref(), Some(ETAG), "etag should round-trip through mbgl");
    assert!(stored.expires.is_some(), "expires should round-trip through mbgl");

    render_once("304 revalidation");
    forwarded_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("the revalidated response should be forwarded");

    assert!(
        cache_hits.load(Ordering::SeqCst) >= 1,
        "second render should revalidate the Database cache entry",
    );
    assert_eq!(revalidations.load(Ordering::SeqCst), 1);
    assert_eq!(network_hits.load(Ordering::SeqCst), 2);
}
