use log::{debug, info, warn};
use maplibre_native::ImageRendererOptions;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Barrier;
use std::thread;

fn fixture_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(name);
    assert!(path.is_file());
    path
}

#[test]
fn simple_two_thread_concurrent_rendering() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();
    let style_paths = vec![
        fixture_path("test-style.json"),
        fixture_path("test-style-alt.json"),
    ];

    let thread_count = 10;
    info!("Starting simple {thread_count}-thread concurrent test");

    let mut handles = Vec::new();
    let rendering_barrier = Arc::new(Barrier::new(thread_count));
    for i in 0..thread_count {
        let style_path = style_paths[i % style_paths.len()].clone();
        let rendering_barrier = rendering_barrier.clone();
        handles.push(thread::spawn(move || {
            info!("Thread {i}: Creating renderer");

            let mut renderer = ImageRendererOptions::default();
            renderer.with_cache_path(format!("cache{i}.sqlite"));
            let mut renderer = renderer.build_tile_renderer();

            debug!("Thread {i}: Loading style");
            if let Err(e) = renderer.load_style_from_path(&style_path) {
                panic!("Thread {i}: Failed to load style: {e}");
            }

            debug!("Thread {i}: Rendering tiles");
            rendering_barrier.wait();
            for j in 1_u32..1000 {
                log::trace!("Thread {i}: Rendering tile {j}");
                renderer.render_tile(10, j, 384).unwrap();
            }
        }));
    }

    for handle in handles {
        handle.join().expect("Thread should not panic");
        warn!("Joined thread");
    }

    info!("Both threads completed successfully!");
}
