#![expect(missing_docs)]

use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use maplibre_native::SingleThreadedRenderPool;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(name)
}

const TILES: [(u8, u32, u32); 16] = [
    (2, 0, 0),
    (2, 1, 0),
    (2, 2, 0),
    (2, 3, 0),
    (2, 0, 1),
    (2, 1, 1),
    (2, 2, 1),
    (2, 3, 1),
    (2, 0, 2),
    (2, 1, 2),
    (2, 2, 2),
    (2, 3, 2),
    (2, 0, 3),
    (2, 1, 3),
    (2, 2, 3),
    (2, 3, 3),
];

fn pool_throughput(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let style = fixture_path("maplibre_demo_live.json");

    if !style.is_file() {
        eprintln!(
            "Skipping: {} not found.\n  \
             curl -sfL https://demotiles.maplibre.org/style.json -o {}",
            style.display(),
            style.display()
        );
        return;
    }

    let pool = rt.block_on(SingleThreadedRenderPool::global_pool()).unwrap();

    // Warmup: triggers HTTP fetches, populates cache, warms MapLibre (not measured)
    rt.block_on(async {
        for &(z, x, y) in &TILES {
            pool.render_tile(style.clone(), z, x, y).await.unwrap();
        }
    });

    let mut group = c.benchmark_group("pool_render_tile");
    group.throughput(Throughput::Elements(TILES.len() as u64));
    group.sample_size(20);
    group.warm_up_time(std::time::Duration::from_secs(1));

    // Steady-state: all tiles already cached + MapLibre warmed.
    // Single block_on per iteration to avoid per-tile runtime enter/exit overhead.
    group.bench_function(BenchmarkId::new("steady_state", TILES.len()), |b| {
        b.iter(|| {
            rt.block_on(async {
                for &(z, x, y) in &TILES {
                    pool.render_tile(style.clone(), z, x, y).await.unwrap();
                }
            });
        });
    });

    group.finish();
}

criterion_group!(benches, pool_throughput);
criterion_main!(benches);
