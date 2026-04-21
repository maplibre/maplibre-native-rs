use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{raw_sql, Row, SqlitePool};

/// Schema version expected by `MapLibre` Native.
const SCHEMA_VERSION: i64 = 6;

/// The full `MapLibre` offline cache schema (v6).
const SCHEMA_SQL: &str = include_str!("cache_schema.sql");

/// Errors that can occur when working with the tile cache.
#[derive(thiserror::Error, Debug)]
pub enum CacheError {
    /// A `SQLite` error occurred.
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    /// The cache database has an incompatible schema version.
    #[error("Incompatible cache schema version: expected {SCHEMA_VERSION}, found {found}")]
    IncompatibleSchema {
        /// The schema version found in the database.
        found: i64,
    },

    /// Failed to fetch a resource during precaching.
    #[error("Failed to fetch {url}: {source}")]
    Fetch {
        /// The URL that failed to fetch.
        url: String,
        /// The underlying error.
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Failed to parse a style JSON document.
    #[error("Failed to parse style JSON: {0}")]
    StyleParse(#[from] serde_json::Error),
}

/// HTTP response metadata forwarded into the cache.
///
/// Maps directly to the `expires`, `etag`, `modified`, `compressed`, and
/// `must_revalidate` columns in `MapLibre`'s cache schema.
#[derive(Clone, Debug, Default)]
pub struct ResponseMeta {
    /// Unix timestamp (seconds) after which the data should be re-fetched.
    pub expires: Option<i64>,
    /// HTTP `ETag` header value for conditional revalidation.
    pub etag: Option<String>,
    /// Unix timestamp (seconds) from the `Last-Modified` header.
    pub modified: Option<i64>,
    /// Whether the response body is deflate-compressed.
    pub compressed: bool,
    /// If `true`, the cached entry must be revalidated before use.
    pub must_revalidate: bool,
}

/// Response from an HTTP fetch: body bytes plus cache metadata.
#[derive(Clone, Debug)]
pub struct FetchResponse {
    /// The response body.
    pub data: Vec<u8>,
    /// HTTP cache metadata to store alongside the data.
    pub meta: ResponseMeta,
}

/// The kind of resource stored in the cache.
///
/// These values match `MapLibre` Native's `Resource::Kind` enumeration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
#[allow(dead_code)]
pub enum ResourceKind {
    /// Map style JSON.
    Style = 1,
    /// Tile source metadata (e.g. `TileJSON`).
    Source = 2,
    /// A map tile (vector or raster) stored in the resources table.
    Tile = 3,
    /// Font glyph ranges (PBF).
    Glyphs = 4,
    /// Sprite sheet image (PNG).
    SpriteImage = 5,
    /// Sprite sheet metadata (JSON).
    SpriteJSON = 6,
    /// A generic image resource.
    Image = 7,
}

/// A handle to `MapLibre`'s `SQLite` tile cache for pre-populating data.
#[derive(Debug)]
pub struct TileCache {
    pool: SqlitePool,
}

impl TileCache {
    /// Open (or create) the cache database at the given path.
    ///
    /// # Errors
    ///
    /// Returns [`CacheError::Sqlx`] if the database cannot be opened or
    /// schema initialization fails, or [`CacheError::IncompatibleSchema`] if
    /// the existing database has an unsupported schema version.
    pub async fn new(cache_path: impl AsRef<Path>) -> Result<Self, CacheError> {
        let options = SqliteConnectOptions::new()
            .filename(cache_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal);

        let pool = SqlitePoolOptions::new().max_connections(4).connect_with(options).await?;

        let row = sqlx::query("PRAGMA user_version").fetch_one(&pool).await?;
        let version: i64 = row.get(0);

        if version == 0 {
            raw_sql(SCHEMA_SQL).execute(&pool).await?;
            raw_sql(&format!("PRAGMA user_version = {SCHEMA_VERSION}")).execute(&pool).await?;
        } else if version != SCHEMA_VERSION {
            return Err(CacheError::IncompatibleSchema { found: version });
        }

        Ok(Self { pool })
    }

    /// Insert a tile into the cache.
    ///
    /// # Errors
    ///
    /// Returns [`CacheError::Sqlx`] if the insert fails.
    #[allow(clippy::too_many_arguments)]
    pub async fn put_tile(
        &self,
        url_template: &str,
        pixel_ratio: i32,
        z: i32,
        x: i32,
        y: i32,
        data: &[u8],
        meta: &ResponseMeta,
    ) -> Result<(), CacheError> {
        let now = now_unix_secs();

        sqlx::query(
            "INSERT OR REPLACE INTO tiles
               (url_template, pixel_ratio, x, y, z, modified, must_revalidate, etag, expires, accessed, data, compressed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        )
        .bind(url_template)
        .bind(pixel_ratio)
        .bind(x)
        .bind(y)
        .bind(z)
        .bind(meta.modified)
        .bind(i32::from(meta.must_revalidate))
        .bind(&meta.etag)
        .bind(meta.expires)
        .bind(now)
        .bind(data)
        .bind(i32::from(meta.compressed))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Insert a resource (style, glyphs, sprites, etc.) into the cache by URL.
    ///
    /// # Errors
    ///
    /// Returns [`CacheError::Sqlx`] if the insert fails.
    pub async fn put_resource(
        &self,
        url: &str,
        kind: ResourceKind,
        data: &[u8],
        meta: &ResponseMeta,
    ) -> Result<(), CacheError> {
        let now = now_unix_secs();

        sqlx::query(
            "INSERT OR REPLACE INTO resources
               (url, kind, etag, expires, must_revalidate, modified, accessed, data, compressed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .bind(url)
        .bind(kind as i32)
        .bind(&meta.etag)
        .bind(meta.expires)
        .bind(i32::from(meta.must_revalidate))
        .bind(meta.modified)
        .bind(now)
        .bind(data)
        .bind(i32::from(meta.compressed))
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Style parsing — extract tile URL templates from a MapLibre style JSON
// ---------------------------------------------------------------------------

/// Resolved tile source: a URL template and the source name it belongs to.
#[derive(Clone, Debug)]
pub(crate) struct TileSource {
    pub url_template: String,
}

/// Parsed style information needed for prefetching.
#[derive(Clone, Debug)]
pub(crate) struct StyleInfo {
    /// Tile sources with resolved URL templates (after `TileJSON` fetch).
    pub sources: Vec<TileSource>,
    /// Sprite base URL (if any). Fetch `{base}.png` and `{base}.json`.
    pub sprite_base: Option<String>,
}

impl StyleInfo {
    /// Parse a style JSON and resolve `TileJSON` URLs using the given HTTP client.
    pub async fn from_style_json(
        style_bytes: &[u8],
        client: &reqwest::Client,
    ) -> Result<Self, CacheError> {
        let style: serde_json::Value = serde_json::from_slice(style_bytes)?;
        let _layers = style.get("layers").and_then(|v| v.as_array());
        let sprite_base = style.get("sprite").and_then(|v| v.as_str()).map(String::from);

        let mut sources = Vec::new();

        if let Some(style_sources) = style.get("sources").and_then(|v| v.as_object()) {
            for (source_name, source) in style_sources {
                // Collect inline tile templates
                if let Some(tiles) = source.get("tiles").and_then(|v| v.as_array()) {
                    for t in tiles.iter().filter_map(|v| v.as_str()) {
                        sources.push(TileSource { url_template: t.to_string() });
                    }
                }

                // Fetch TileJSON to discover templates
                if let Some(tilejson_url) = source.get("url").and_then(|v| v.as_str()) {
                    if let Ok(resp) = client.get(tilejson_url).send().await {
                        if let Ok(body) = resp.bytes().await {
                            if let Ok(tj) = serde_json::from_slice::<serde_json::Value>(&body) {
                                if let Some(tiles) = tj.get("tiles").and_then(|v| v.as_array()) {
                                    for t in tiles.iter().filter_map(|v| v.as_str()) {
                                        sources.push(TileSource { url_template: t.to_string() });
                                    }
                                }
                            }
                        }
                    }
                }

                let _ = source_name; // used only for zoom filtering below
            }
        }

        Ok(Self { sources, sprite_base })
    }

    /// Collect all fetch URLs needed for a single tile coordinate.
    pub fn tile_urls(&self, z: i32, x: i32, y: i32) -> Vec<(String, String)> {
        self.sources
            .iter()
            .map(|s| {
                let url = resolve_tile_url(&s.url_template, z, x, y);
                (s.url_template.clone(), url)
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

/// Fetch a URL and return body + cache metadata.
pub(crate) async fn http_fetch(
    client: &reqwest::Client,
    url: &str,
) -> Result<FetchResponse, Box<dyn std::error::Error + Send + Sync>> {
    let response = client.get(url).send().await?.error_for_status()?;
    let meta = response_meta_from_headers(response.headers());
    let data = response.bytes().await?.to_vec();
    Ok(FetchResponse { data, meta })
}

/// Extract cache metadata from HTTP response headers.
fn response_meta_from_headers(headers: &reqwest::header::HeaderMap) -> ResponseMeta {
    let etag = headers.get(reqwest::header::ETAG).and_then(|v| v.to_str().ok()).map(String::from);

    let expires = headers
        .get(reqwest::header::EXPIRES)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| httpdate::parse_http_date(v).ok())
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .and_then(|d| i64::try_from(d.as_secs()).ok());

    let modified = headers
        .get(reqwest::header::LAST_MODIFIED)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| httpdate::parse_http_date(v).ok())
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .and_then(|d| i64::try_from(d.as_secs()).ok());

    let must_revalidate = headers
        .get(reqwest::header::CACHE_CONTROL)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains("must-revalidate"));

    let compressed = headers
        .get(reqwest::header::CONTENT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains("deflate"));

    ResponseMeta { expires, etag, modified, compressed, must_revalidate }
}

/// Substitute `{z}`, `{x}`, `{y}` placeholders in a tile URL template.
fn resolve_tile_url(template: &str, z: i32, x: i32, y: i32) -> String {
    template
        .replace("{z}", &z.to_string())
        .replace("{x}", &x.to_string())
        .replace("{y}", &y.to_string())
}

fn now_unix_secs() -> i64 {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_secs(),
    )
    .expect("current time exceeds i64 range")
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_cache() -> (TileCache, tempfile::TempDir) {
        let dir = tempfile::TempDir::new().unwrap();
        let cache = TileCache::new(dir.path().join("test.sqlite")).await.unwrap();
        (cache, dir)
    }

    #[tokio::test]
    async fn test_new_creates_schema() {
        let (cache, _dir) = test_cache().await;

        let row = sqlx::query("PRAGMA user_version").fetch_one(&cache.pool).await.unwrap();
        let version: i64 = row.get(0);
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[tokio::test]
    async fn test_reopen_existing_db() {
        let dir = tempfile::TempDir::new().unwrap();
        let db_path = dir.path().join("test.sqlite");

        let cache = TileCache::new(&db_path).await.unwrap();
        drop(cache);

        let _cache = TileCache::new(&db_path).await.unwrap();
    }

    #[tokio::test]
    async fn test_put_and_read_tile() {
        let (cache, _dir) = test_cache().await;
        let meta = ResponseMeta::default();

        cache
            .put_tile("https://example.com/{z}/{x}/{y}.pbf", 1, 5, 10, 20, b"fake tile data", &meta)
            .await
            .unwrap();

        let row = sqlx::query(
            "SELECT data, compressed, must_revalidate
             FROM tiles WHERE url_template = ?1 AND pixel_ratio = ?2 AND z = ?3 AND x = ?4 AND y = ?5",
        )
        .bind("https://example.com/{z}/{x}/{y}.pbf")
        .bind(1)
        .bind(5)
        .bind(10)
        .bind(20)
        .fetch_one(&cache.pool)
        .await
        .unwrap();

        assert_eq!(row.get::<Vec<u8>, _>("data"), b"fake tile data");
        assert_eq!(row.get::<i32, _>("compressed"), 0);
        assert_eq!(row.get::<i32, _>("must_revalidate"), 0);
    }

    #[tokio::test]
    async fn test_put_tile_replaces_existing() {
        let (cache, _dir) = test_cache().await;
        let meta = ResponseMeta::default();

        cache
            .put_tile("https://example.com/{z}/{x}/{y}.pbf", 1, 0, 0, 0, b"old", &meta)
            .await
            .unwrap();
        cache
            .put_tile("https://example.com/{z}/{x}/{y}.pbf", 1, 0, 0, 0, b"new", &meta)
            .await
            .unwrap();

        let rows = sqlx::query("SELECT data FROM tiles").fetch_all(&cache.pool).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get::<Vec<u8>, _>("data"), b"new");
    }

    #[tokio::test]
    async fn test_put_and_read_resource() {
        let (cache, _dir) = test_cache().await;
        let meta = ResponseMeta::default();

        cache
            .put_resource(
                "https://example.com/style.json",
                ResourceKind::Style,
                br#"{"version": 8}"#,
                &meta,
            )
            .await
            .unwrap();

        let row = sqlx::query("SELECT data, kind FROM resources WHERE url = ?1")
            .bind("https://example.com/style.json")
            .fetch_one(&cache.pool)
            .await
            .unwrap();

        assert_eq!(row.get::<Vec<u8>, _>("data"), br#"{"version": 8}"#);
        assert_eq!(row.get::<i32, _>("kind"), ResourceKind::Style as i32);
    }

    #[tokio::test]
    async fn test_incompatible_schema_version() {
        let dir = tempfile::TempDir::new().unwrap();
        let db_path = dir.path().join("test.sqlite");

        let options = SqliteConnectOptions::new().filename(&db_path).create_if_missing(true);
        let pool = SqlitePoolOptions::new().connect_with(options).await.unwrap();
        sqlx::query("PRAGMA user_version = 99").execute(&pool).await.unwrap();
        drop(pool);

        let result = TileCache::new(&db_path).await;
        assert!(matches!(result.unwrap_err(), CacheError::IncompatibleSchema { found: 99 }));
    }
}
