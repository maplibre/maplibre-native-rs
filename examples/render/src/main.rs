//! Command-line tool for rendering map tiles using mapLibre Native.
//!
//! This example demonstrates how to use the different rendering options
//! including different map styles, zoom levels, and output formats.
//!
//! For exapmle create a image of a specific tile with `cargo run -- -m tile -z 3 -x 4 -y 2`
//! or of a specific area (uses lat,lon and zoom) `cargo run -- --zoom 3.9 --lat 17.209 --lon -87.41`

use std::num::NonZeroU32;
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use env_logger::Env;
use maplibre_native::{Image, ImageRenderer, ImageRendererBuilder, MapDebugOptions, Static, Tile};

/// Command-line tool to render a map via [`mapLibre-native`](https://github.com/maplibre/maplibre-native)
#[derive(Parser, Debug)]
struct Args {
    /// API key
    #[arg(short = 't', long = "apikey", env = "MLN_API_KEY")]
    apikey: Option<String>,

    /// Map stylesheet either as an URL or a path to a local file
    #[arg(
        short = 's',
        long = "style",
        default_value = "https://demotiles.maplibre.org/style.json"
    )]
    style: String,

    /// Output file name
    #[arg(short = 'o', long = "output", default_value = "out.png")]
    output: PathBuf,

    /// Cache database file name
    #[arg(short = 'c', long = "cache", default_value = "cache.sqlite")]
    cache: PathBuf,

    /// Directory to which `asset://` URLs will resolve
    #[arg(short = 'a', long = "assets", default_value = ".")]
    asset_root: PathBuf,

    /// Adds an debug overlay
    #[arg(long)]
    debug: Option<DebugMode>,

    /// Image scale factor
    #[arg(short = 'r', long = "ratio", default_value_t = 1.0)]
    ratio: f32,

    /// Zoom level (distinct)
    #[arg(short = 'z', long = "z", default_value_t = 0)]
    z: u8,

    /// x coordinate
    #[arg(short = 'x', long = "x", default_value_t = 0)]
    x: u32,

    /// y coordiante
    #[arg(short = 'y', long = "y", default_value_t = 0)]
    y: u32,

    /// Latitude in degrees [-90..90]
    #[arg(long, value_parser = clap::value_parser!(f64), default_value_t = 0.0)]
    lat: f64,

    /// Longitude in degrees [-90..90]
    #[arg(long, value_parser = clap::value_parser!(f64), allow_hyphen_values(true), default_value_t = 0.0)]
    lon: f64,

    /// Zoom level
    #[arg(long, value_parser = clap::value_parser!(f64), default_value_t = 0.0)]
    zoom: f64,

    /// Bearing
    #[arg(short = 'b', long = "bearing", default_value_t = 0.0)]
    bearing: f64,

    /// Pitch
    #[arg(short = 'p', long = "pitch", default_value_t = 0.0)]
    pitch: f64,

    /// Image width
    #[arg(long = "width", default_value_t = NonZeroU32::new(512).unwrap())]
    width: NonZeroU32,

    /// Image height
    #[arg(long = "height", default_value_t = NonZeroU32::new(512).unwrap())]
    height: NonZeroU32,

    /// Map mode
    #[arg(short = 'm', long = "mode", default_value = "static")]
    mode: Mode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
enum Mode {
    #[default]
    /// Once-off still image of an arbitrary viewport
    Static,
    /// Once-off still image of a single tile
    Tile,
    /// Continually updating map
    Continuous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum DebugMode {
    /// Edges of tile boundaries are shown as thick, red lines.
    ///
    /// Can help diagnose tile clipping issues.
    TileBorders,
    ParseStatus,
    /// Each tile shows a timestamp indicating when it was loaded.
    Timestamps,
    /// Edges of glyphs and symbols are shown as faint, green lines.
    ///
    /// Can help diagnose collision and label placement issues.
    Collision,
    /// Each drawing operation is replaced by a translucent fill.
    ///
    /// Overlapping drawing operations appear more prominent to help diagnose overdrawing.
    Overdraw,
    /// The stencil buffer is shown instead of the color buffer.
    ///
    /// Note: This option does nothing in Release builds of the SDK.
    StencilClip,
    /// The depth buffer is shown instead of the color buffer.
    ///
    /// Note: This option does nothing in Release builds of the SDK
    DepthBuffer,
}

impl From<DebugMode> for MapDebugOptions {
    fn from(value: DebugMode) -> Self {
        match value {
            DebugMode::TileBorders => MapDebugOptions::TileBorders,
            DebugMode::ParseStatus => MapDebugOptions::ParseStatus,
            DebugMode::Timestamps => MapDebugOptions::Timestamps,
            DebugMode::Collision => MapDebugOptions::Collision,
            DebugMode::Overdraw => MapDebugOptions::Overdraw,
            DebugMode::StencilClip => MapDebugOptions::StencilClip,
            DebugMode::DepthBuffer => MapDebugOptions::DepthBuffer,
        }
    }
}

impl Args {
    fn load(self) -> Renderer {
        let map = ImageRendererBuilder::new()
            .with_api_key(self.apikey.unwrap_or_default())
            .with_cache_path(self.cache)
            .with_asset_root(self.asset_root)
            .with_pixel_ratio(self.ratio)
            .with_size(self.width, self.height);

        match self.mode {
            Mode::Static => {
                if !(-90.0..=90.0).contains(&self.lat) {
                    panic!("lat must be between -90 and 90")
                }
                if !(-180.0..=180.0).contains(&self.lon) {
                    panic!("lon must be between -180 and 180")
                }

                let mut map = map.build_static_renderer();
                if let Some(debug) = self.debug {
                    map.set_debug_flags(debug.into());
                }
                if let Ok(url) = url::Url::parse(&self.style) {
                    map.load_style_from_url(&url);
                } else {
                    map.load_style_from_path(self.style)
                        .expect("the path to be valid");
                }
                Renderer::Static {
                    map,
                    lat: self.lat,
                    lon: self.lon,
                    zoom: self.zoom,
                    bearing: self.bearing,
                    pitch: self.pitch,
                }
            }
            Mode::Tile => {
                if self.bearing != 0.0 {
                    println!("Warning: nonzero bearing is ignored in tile-mode");
                }
                if self.pitch != 0.0 {
                    println!("Warning: nonzero pitch is ignored in tile-mode");
                }
                let mut map = map.build_tile_renderer();
                if let Ok(url) = url::Url::parse(&self.style) {
                    map.load_style_from_url(&url);
                } else {
                    map.load_style_from_path(self.style)
                        .expect("the path to be valid");
                }
                if let Some(debug) = self.debug {
                    map.set_debug_flags(debug.into());
                }
                Renderer::Tiled {
                    map,
                    x: self.x,
                    y: self.y,
                    z: self.z,
                }
            }
            Mode::Continuous => {
                todo!("not yet implemented in the wrapper")
            }
        }
    }
}

enum Renderer {
    Static {
        map: ImageRenderer<Static>,
        lat: f64,
        lon: f64,
        zoom: f64,
        bearing: f64,
        pitch: f64,
    },
    Tiled {
        map: ImageRenderer<Tile>,
        x: u32,
        y: u32,
        z: u8,
    },
}
impl Renderer {
    fn render(&mut self) -> Image {
        match self {
            Renderer::Static {
                map,
                lat,
                lon,
                zoom,
                bearing,
                pitch,
            } => map
                .render_static(*lat, *lon, *zoom, *bearing, *pitch)
                .expect("could not render image"),
            Renderer::Tiled { map, x, y, z } => {
                map.render_tile(*z, *x, *y).expect("could not render image")
            }
        }
    }
}
fn main() {
    env_logger::Builder::from_env(Env::new().default_filter_or("trace")).init();
    log::info!("Starting MapLibre Native renderer with logging enabled");

    let args = Args::parse();
    println!("Rendering arguments: {args:#?}");
    let output = args.output.clone();

    let before_initalisation = Instant::now();
    let mut renderer = args.load();
    println!("intialisation took {:?}", before_initalisation.elapsed());
    let before_render1 = Instant::now();
    let data = renderer.render();
    println!(
        "Rendering successfull in {:?}, writing result to {}",
        before_render1.elapsed(),
        output.display()
    );
    println!("Tip: Future renders using the same instance would be faster due to amortized initialization");
    data.as_image().save(&output).unwrap_or_else(|e| {
        panic!(
            "Failed to write rendered map to {} because of {e:?}",
            output.display()
        )
    });
    let before_second_render = Instant::now();
    let data = renderer.render();
    assert!(data.as_image().dimensions() != (0, 0));
    println!("A second render took {:?}", before_second_render.elapsed());
}

#[cfg(test)]
mod tests {
    use std::num::NonZero;

    use super::*;

    #[test]
    fn test_rendering() {
        let args = Args {
            width: NonZero::new(32).unwrap(),
            height: NonZero::new(32).unwrap(),
            mode: Mode::Static,
            ..Args::parse()
        };
        let mut renderer = args.load();
        let image = renderer.render();

        // Test image properties
        let img_buffer = image.as_image();
        assert_eq!(img_buffer.width(), 32);
        assert_eq!(img_buffer.height(), 32);
        assert_eq!(img_buffer.dimensions(), (32, 32));
        assert!(!img_buffer.as_raw().is_empty());
        assert_eq!(img_buffer.as_raw().len(), 32 * 32 * 4); // RGBA

        let args = Args {
            width: NonZero::new(64).unwrap(),
            height: NonZero::new(64).unwrap(),
            mode: Mode::Tile,
            ..Args::parse()
        };
        let mut renderer = args.load();
        let image = renderer.render();

        // Test tile rendering
        let img_buffer = image.as_image();
        assert_eq!(img_buffer.width(), 64);
        assert_eq!(img_buffer.height(), 64);
        assert!(!img_buffer.as_raw().is_empty());
        assert_eq!(img_buffer.as_raw().len(), 64 * 64 * 4); // RGBA
    }
}
