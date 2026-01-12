use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use env_logger::Env;
use maplibre_native::MultiThreadedRenderPool;

/// Parallel tile rendering example using MapLibre Native's multi-process pool
#[derive(Parser, Debug)]
struct Args {
    /// Map stylesheet either as an URL or a path to a local file
    #[arg(
        short = 's',
        long = "style",
        default_value = "https://demotiles.maplibre.org/style.json"
    )]
    style: String,

    /// Output directory for rendered tiles
    #[arg(short = 'o', long = "output", default_value = "tiles")]
    output_dir: PathBuf,

    /// Number of worker processes to spawn
    #[arg(short = 'w', long = "workers", default_value_t = 4)]
    workers: usize,

    /// Zoom level
    #[arg(short = 'z', long = "zoom", default_value_t = 10)]
    zoom: u8,

    /// Starting X coordinate
    #[arg(long = "x-start", default_value_t = 0)]
    x_start: u32,

    /// Starting Y coordinate
    #[arg(long = "y-start", default_value_t = 0)]
    y_start: u32,

    /// Number of tiles in X direction
    #[arg(long = "x-count", default_value_t = 100)]
    x_count: u32,

    /// Number of tiles in Y direction
    #[arg(long = "y-count", default_value_t = 100)]
    y_count: u32,
}

#[tokio::main]
async fn main() {
    // Check if this is a worker process
    if MultiThreadedRenderPool::is_worker_process() {
        // Run as worker process - this will loop indefinitely processing requests
        if let Err(e) = MultiThreadedRenderPool::run_worker() {
            eprintln!("Worker process error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    // Main process logic
    env_logger::Builder::from_env(Env::new().default_filter_or("info")).init();
    let args = Args::parse();

    log::info!("Starting parallel tile renderer");
    log::info!("Configuration: {:#?}", args);

    // Resolve style path
    let style_path = if let Ok(url) = url::Url::parse(&args.style) {
        // For URLs, we would need to download and cache - for simplicity, just error
        log::error!("URL styles not yet supported in this example, please use a local file path");
        std::process::exit(1);
    } else {
        PathBuf::from(&args.style)
    };

    if !style_path.exists() {
        log::error!("Style file does not exist: {}", style_path.display());
        std::process::exit(1);
    }

    // Create output directory
    std::fs::create_dir_all(&args.output_dir).expect("Failed to create output directory");

    // Create the multi-threaded pool
    log::info!("Creating pool with {} workers", args.workers);
    let pool = MultiThreadedRenderPool::new(args.workers)
        .expect("Failed to create multi-threaded render pool");

    // Generate list of tiles to render
    let total_tiles = args.x_count * args.y_count;
    log::info!(
        "Rendering {} tiles ({}x{}) at zoom level {}",
        total_tiles,
        args.x_count,
        args.y_count,
        args.zoom
    );

    let start = Instant::now();

    // Create futures for all tile renders
    let mut render_tasks = Vec::new();
    for dy in 0..args.y_count {
        for dx in 0..args.x_count {
            let x = args.x_start + dx;
            let y = args.y_start + dy;
            let z = args.zoom;

            let style_path_clone = style_path.clone();
            let output_dir_clone = args.output_dir.clone();
            let pool_clone = pool.clone();

            // Spawn a task for each tile
            let task = tokio::spawn(async move {
                let tile_start = Instant::now();

                match pool_clone.render_tile(style_path_clone, z, x, y).await {
                    Ok(image) => {
                        // Save the tile
                        let output_path = output_dir_clone.join(format!("{}_{}_{}.png", z, x, y));
                        if let Err(e) = image.as_image().save(&output_path) {
                            log::error!("Failed to save tile {}/{}/{}: {}", z, x, y, e);
                            return Err(());
                        }

                        let elapsed = tile_start.elapsed();
                        log::info!(
                            "Rendered tile {}/{}/{} in {:?} -> {}",
                            z,
                            x,
                            y,
                            elapsed,
                            output_path.display()
                        );
                        Ok(())
                    }
                    Err(e) => {
                        log::error!("Failed to render tile {}/{}/{}: {}", z, x, y, e);
                        Err(())
                    }
                }
            });

            render_tasks.push(task);
        }
    }

    // Wait for all renders to complete
    log::info!(
        "Waiting for {} render tasks to complete...",
        render_tasks.len()
    );
    let results = futures::future::join_all(render_tasks).await;

    let elapsed = start.elapsed();
    let success_count = results
        .iter()
        .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
        .count();
    let failure_count = total_tiles as usize - success_count;

    log::info!("=====================================");
    log::info!("Rendering complete!");
    log::info!("Total time: {:?}", elapsed);
    log::info!("Average time per tile: {:?}", elapsed / total_tiles);
    log::info!("Successful renders: {}", success_count);
    log::info!("Failed renders: {}", failure_count);
    log::info!(
        "Tiles per second: {:.2}",
        total_tiles as f64 / elapsed.as_secs_f64()
    );
    log::info!("Output directory: {}", args.output_dir.display());
    log::info!("=====================================");

    if failure_count > 0 {
        std::process::exit(1);
    }
}
