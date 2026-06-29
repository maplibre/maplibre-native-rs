//! End-to-end test for the optional `tokio` file-source adapter.

use std::{
    num::NonZeroU32,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc,
    },
    thread,
    time::Duration,
};

use maplibre_native::{
    file_source::Response, register_tokio_file_source_with_handle, CameraUpdate, FileSourceType,
    ImageRendererBuilder, LatLng, ResourceKind, ResourceRequest, RunLoopHandle, Size,
    TokioFileSource,
};

const INLINE_STYLE: &str = r#"{
    "version": 8,
    "name": "tokio-test",
    "sources": {},
    "layers": [
        { "id": "bg", "type": "background", "paint": { "background-color": "rgb(0, 128, 255)" } }
    ]
}"#;

struct AsyncStyleSource {
    cancel_started_tx: mpsc::Sender<()>,
    abort_drop_count: Arc<AtomicUsize>,
    abort_drop_tx: mpsc::Sender<()>,
}

struct AbortDropCounter {
    abort_drop_count: Arc<AtomicUsize>,
    abort_drop_tx: mpsc::Sender<()>,
}

impl Drop for AbortDropCounter {
    fn drop(&mut self) {
        self.abort_drop_count.fetch_add(1, Ordering::SeqCst);
        let _ = self.abort_drop_tx.send(());
    }
}

impl TokioFileSource for AsyncStyleSource {
    fn can_request(&self, request: &ResourceRequest) -> bool {
        request.kind == ResourceKind::Style
            && (request.url.ends_with("/inline-style.json")
                || request.url.ends_with("/cancel-style.json"))
    }

    async fn request(&self, request: ResourceRequest) -> Response {
        // Force async scheduling before completion.
        tokio::task::yield_now().await;
        if request.url.ends_with("/inline-style.json") {
            Response::data(INLINE_STYLE.as_bytes().to_vec())
        } else if request.url.ends_with("/cancel-style.json") {
            let _guard = AbortDropCounter {
                abort_drop_count: Arc::clone(&self.abort_drop_count),
                abort_drop_tx: self.abort_drop_tx.clone(),
            };
            self.cancel_started_tx.send(()).expect("test receiver should be alive");
            std::future::pending().await
        } else {
            Response::no_content()
        }
    }
}

fn static_renderer() -> maplibre_native::ImageRenderer<maplibre_native::Static> {
    ImageRendererBuilder::new()
        .with_size(NonZeroU32::new(64).unwrap(), NonZeroU32::new(64).unwrap())
        .with_pixel_ratio(1.0)
        .build_static_renderer()
}

fn camera() -> CameraUpdate {
    CameraUpdate::new().center(LatLng { lat: 0.0, lng: 0.0 }).zoom(0.0).bearing(0.0).pitch(0.0)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tokio_file_source_adapter() {
    let (cancel_started_tx, cancel_started_rx) = mpsc::channel();
    let (abort_drop_tx, abort_drop_rx) = mpsc::channel();
    let abort_drop_count = Arc::new(AtomicUsize::new(0));

    register_tokio_file_source_with_handle(
        FileSourceType::Network,
        tokio::runtime::Handle::current(),
        AsyncStyleSource {
            cancel_started_tx,
            abort_drop_count: Arc::clone(&abort_drop_count),
            abort_drop_tx,
        },
    );

    let image = thread::spawn(|| {
        let mut renderer = static_renderer();

        let url: url::Url = "https://example.invalid/inline-style.json".parse().unwrap();
        renderer.load_style_from_url(&url);
        renderer.set_map_size(Size { width: 64, height: 64 });

        renderer.render_static(&camera())
    })
    .join()
    .expect("renderer thread should not panic")
    .expect("render should succeed");

    let buf = image.as_image();
    assert_eq!(buf.width(), 64);
    assert_eq!(buf.height(), 64);

    thread::spawn(move || {
        let mut renderer = static_renderer();

        let url: url::Url = "https://example.invalid/cancel-style.json".parse().unwrap();
        let _request = renderer.load_style_from_url(&url);

        let run_loop = RunLoopHandle::current();
        let mut started = false;
        for _ in 0..500 {
            run_loop.tick();
            if cancel_started_rx.try_recv().is_ok() {
                started = true;
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(started, "cancel-style request should start");
    })
    .join()
    .expect("renderer thread should not panic");

    tokio::task::spawn_blocking(move || abort_drop_rx.recv_timeout(Duration::from_secs(5)))
        .await
        .expect("abort wait task should not panic")
        .expect("tokio task future should be dropped");
    assert_eq!(
        abort_drop_count.load(Ordering::SeqCst),
        1,
        "dropping an in-flight tokio request should abort and drop its task future",
    );
}
