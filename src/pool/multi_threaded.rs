//! Multi-process rendering pool for parallel [MapLibre Native](https://maplibre.org/projects/native/) rendering.
//!
//! This module provides a process-based rendering pool that enables true parallelism
//! by spawning multiple worker processes. Each process handles rendering independently,
//! avoiding thread-safety issues in the underlying MapLibre Native library.
//!
//! # Example
//!
//! ```no_run
//! # async fn example() {
//! use maplibre_native::MultiThreadedRenderPool;
//! use std::path::PathBuf;
//!
//! // Create a pool with 4 worker processes
//! let pool = MultiThreadedRenderPool::new(4).unwrap();
//!
//! // Render multiple tiles concurrently
//! let style_path = PathBuf::from("path/to/style.json");
//! let futures: Vec<_> = (0..10)
//!     .map(|i| pool.render_tile(style_path.clone(), 10, 512 + i, 384))
//!     .collect();
//!
//! // All tiles will be rendered in parallel across worker processes
//! let results = futures::future::join_all(futures).await;
//! # }
//! ```

use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};

#[cfg(feature = "log")]
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use crate::renderer::{Image, ImageRendererBuilder, RenderingError};

/// Size of the length prefix for binary messages (4 bytes for u32)
const LENGTH_PREFIX_SIZE: usize = 4;

/// Message sent from the main process to a worker process.
#[derive(Debug, Serialize, Deserialize)]
struct WorkerRequest {
    /// Unique request ID for matching responses
    id: u64,
    /// Path to the MapLibre style JSON file
    style_path: PathBuf,
    /// Tile zoom level
    z: u8,
    /// Tile X coordinate
    x: u32,
    /// Tile Y coordinate
    y: u32,
}

/// Message sent from a worker process back to the main process.
#[derive(Debug, Serialize, Deserialize)]
struct WorkerResponse {
    /// Request ID this response corresponds to
    id: u64,
    /// Result of the rendering operation (raw RGBA bytes with dimension header)
    result: Result<Vec<u8>, String>,
}

/// Represents a single worker process in the pool.
struct Worker {
    /// Child process handle
    _process: Child,
    /// Standard input stream for sending requests
    stdin: ChildStdin,
    /// Pending responses waiting to be fulfilled
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Image, MultiThreadedRenderPoolError>>>>>,
}

impl Worker {
    /// Spawn a new worker process.
    fn spawn() -> Result<Self, MultiThreadedRenderPoolError> {
        let mut process = Command::new(std::env::current_exe()?)
            .arg("--maplibre-worker")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let stdin = process.stdin.take().ok_or_else(|| {
            MultiThreadedRenderPoolError::WorkerSpawnError("Failed to capture stdin".to_string())
        })?;

        let stdout = process.stdout.take().ok_or_else(|| {
            MultiThreadedRenderPoolError::WorkerSpawnError("Failed to capture stdout".to_string())
        })?;

        let pending = Arc::new(Mutex::new(HashMap::new()));
        let pending_clone = Arc::clone(&pending);

        // Spawn a thread to read responses from this worker
        std::thread::spawn(move || {
            Self::read_responses(stdout, pending_clone);
        });

        Ok(Self {
            _process: process,
            stdin,
            pending,
        })
    }

    /// Send a render request to this worker.
    fn send_request(
        &mut self,
        id: u64,
        style_path: PathBuf,
        z: u8,
        x: u32,
        y: u32,
        response_tx: oneshot::Sender<Result<Image, MultiThreadedRenderPoolError>>,
    ) -> Result<(), MultiThreadedRenderPoolError> {
        #[cfg(feature = "log")]
        let start = Instant::now();

        // Register the pending response
        self.pending.lock().unwrap().insert(id, response_tx);

        // Serialize and send the request using bincode
        let request = WorkerRequest {
            id,
            style_path,
            z,
            x,
            y,
        };

        let encoded = bincode::serialize(&request)
            .map_err(|e| MultiThreadedRenderPoolError::SerializationError(e.to_string()))?;

        // Send length prefix followed by data
        let len = encoded.len() as u32;
        self.stdin.write_all(&len.to_le_bytes())?;
        self.stdin.write_all(&encoded)?;
        self.stdin.flush()?;

        #[cfg(feature = "log")]
        log::trace!(
            "Sent request {} ({}bytes) in {:?}",
            id,
            encoded.len(),
            start.elapsed()
        );

        Ok(())
    }

    /// Read and process responses from a worker process.
    fn read_responses(
        mut stdout: ChildStdout,
        pending: Arc<
            Mutex<HashMap<u64, oneshot::Sender<Result<Image, MultiThreadedRenderPoolError>>>>,
        >,
    ) {
        use std::io::Read;

        loop {
            // Read length prefix (4 bytes)
            let mut len_bytes = [0u8; LENGTH_PREFIX_SIZE];
            if stdout.read_exact(&mut len_bytes).is_err() {
                break;
            }
            let len = u32::from_le_bytes(len_bytes) as usize;

            // Read message data
            let mut buffer = vec![0u8; len];
            if stdout.read_exact(&mut buffer).is_err() {
                break;
            }

            // Deserialize response using bincode
            let response: WorkerResponse = match bincode::deserialize(&buffer) {
                Ok(r) => r,
                Err(_) => continue,
            };

            #[cfg(feature = "log")]
            log::trace!("Received response {} ({}bytes)", response.id, buffer.len());

            // Find the pending response channel
            let sender = pending.lock().unwrap().remove(&response.id);

            if let Some(sender) = sender {
                #[cfg(feature = "log")]
                let decode_start = Instant::now();

                let result = response.result.map_or_else(
                    |e| Err(MultiThreadedRenderPoolError::WorkerError(e)),
                    |data| {
                        Image::from_raw_bytes(&data).ok_or_else(|| {
                            MultiThreadedRenderPoolError::ImageDecodeError(
                                "Failed to decode raw image data".to_string(),
                            )
                        })
                    },
                );

                #[cfg(feature = "log")]
                if result.is_ok() {
                    log::trace!("Decoded image in {:?}", decode_start.elapsed());
                }

                let _ = sender.send(result);
            }
        }
    }
}

/// A multi-process rendering pool that distributes [MapLibre Native](https://maplibre.org/projects/native/)
/// tile rendering operations across multiple worker processes.
///
/// This enables true parallel rendering by isolating each renderer in its own process,
/// avoiding thread-safety issues in the underlying C++ library.
///
/// # Process Management
///
/// The pool spawns worker processes as separate instances of the current executable.
/// Workers are identified by the `--maplibre-worker` command-line argument.
/// Communication happens via JSON-encoded messages over stdin/stdout pipes.
#[derive(Clone)]
pub struct MultiThreadedRenderPool {
    workers: Arc<Mutex<Vec<Worker>>>,
    next_request_id: Arc<Mutex<u64>>,
    next_worker_idx: Arc<Mutex<usize>>,
}

impl MultiThreadedRenderPool {
    /// Create a new multi-process rendering pool with the specified number of workers.
    ///
    /// # Arguments
    ///
    /// * `num_workers` - Number of worker processes to spawn. Should typically match
    ///   the number of CPU cores for optimal performance.
    ///
    /// # Errors
    ///
    /// Returns an error if any worker process fails to spawn.
    pub fn new(num_workers: usize) -> Result<Self, MultiThreadedRenderPoolError> {
        let mut workers = Vec::with_capacity(num_workers);

        for _ in 0..num_workers {
            workers.push(Worker::spawn()?);
        }

        Ok(Self {
            workers: Arc::new(Mutex::new(workers)),
            next_request_id: Arc::new(Mutex::new(0)),
            next_worker_idx: Arc::new(Mutex::new(0)),
        })
    }

    /// Render an encoded tile [`Image`] asynchronously using the worker pool.
    ///
    /// Requests are distributed to workers in a round-robin fashion. Multiple
    /// concurrent requests will be processed in parallel across different workers.
    ///
    /// # Arguments
    ///
    /// * `style_path` - Path to the MapLibre style JSON file
    /// * `z` - Tile zoom level
    /// * `x` - Tile X coordinate
    /// * `y` - Tile Y coordinate
    ///
    /// # Errors
    ///
    /// Returns an error if rendering fails, the worker process crashes, or
    /// communication with the worker fails.
    pub async fn render_tile(
        &self,
        style_path: PathBuf,
        z: u8,
        x: u32,
        y: u32,
    ) -> Result<Image, MultiThreadedRenderPoolError> {
        let (response_tx, response_rx) = oneshot::channel();

        // Get the next request ID
        let request_id = {
            let mut id = self.next_request_id.lock().unwrap();
            let current = *id;
            *id = id.wrapping_add(1);
            current
        };

        // Select the next worker in round-robin fashion
        let worker_idx = {
            let mut idx = self.next_worker_idx.lock().unwrap();
            let current = *idx;
            let num_workers = self.workers.lock().unwrap().len();
            *idx = (*idx + 1) % num_workers;
            current
        };

        // Send the request to the selected worker
        {
            let mut workers = self.workers.lock().unwrap();
            workers[worker_idx].send_request(request_id, style_path, z, x, y, response_tx)?;
        }

        // Wait for the response
        response_rx
            .await
            .map_err(|_| MultiThreadedRenderPoolError::FailedToReceiveResponse)?
    }

    /// Check if the current process is running as a worker.
    ///
    /// This function checks command-line arguments for the `--maplibre-worker` flag.
    #[must_use]
    pub fn is_worker_process() -> bool {
        std::env::args().any(|arg| arg == "--maplibre-worker")
    }

    /// Run the worker event loop.
    ///
    /// This function should be called when the process is started with the
    /// `--maplibre-worker` flag. It will run indefinitely, processing render
    /// requests from the main process.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails or if I/O errors occur.
    pub fn run_worker() -> Result<(), MultiThreadedRenderPoolError> {
        use std::io::Read;

        #[cfg(feature = "log")]
        log::debug!("Worker process started");

        let mut renderer = ImageRendererBuilder::default().build_tile_renderer();
        let mut current_style: Option<PathBuf> = None;

        let mut stdin = std::io::stdin();

        loop {
            // Read length prefix (4 bytes)
            let mut len_bytes = [0u8; LENGTH_PREFIX_SIZE];
            if stdin.read_exact(&mut len_bytes).is_err() {
                break;
            }
            let len = u32::from_le_bytes(len_bytes) as usize;

            // Read message data
            let mut buffer = vec![0u8; len];
            if stdin.read_exact(&mut buffer).is_err() {
                break;
            }

            // Deserialize request using bincode
            let request: WorkerRequest = match bincode::deserialize(&buffer) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Worker: Failed to parse request: {}", e);
                    continue;
                }
            };

            #[cfg(feature = "log")]
            let request_start = Instant::now();

            // Load style if it's different from current
            if current_style.as_ref() != Some(&request.style_path) {
                #[cfg(feature = "log")]
                let style_load_start = Instant::now();

                if let Err(e) = renderer.load_style_from_path(&request.style_path) {
                    let response = WorkerResponse {
                        id: request.id,
                        result: Err(format!("Failed to load style: {}", e)),
                    };
                    Self::send_response(&response)?;
                    continue;
                }
                current_style = Some(request.style_path);

                #[cfg(feature = "log")]
                log::debug!("Loaded style in {:?}", style_load_start.elapsed());
            }

            #[cfg(feature = "log")]
            let render_start = Instant::now();

            // Render the tile
            let result = match renderer.render_tile(request.z, request.x, request.y) {
                Ok(image) => {
                    #[cfg(feature = "log")]
                    log::trace!(
                        "Rendered tile {}/{}/{} in {:?}",
                        request.z,
                        request.x,
                        request.y,
                        render_start.elapsed()
                    );

                    #[cfg(feature = "log")]
                    let encode_start = Instant::now();

                    let bytes = image.to_raw_bytes();

                    #[cfg(feature = "log")]
                    log::trace!(
                        "Encoded to {} bytes in {:?}",
                        bytes.len(),
                        encode_start.elapsed()
                    );

                    Ok(bytes)
                }
                Err(e) => Err(format!("Rendering error: {}", e)),
            };

            let response = WorkerResponse {
                id: request.id,
                result,
            };

            Self::send_response(&response)?;

            #[cfg(feature = "log")]
            log::trace!(
                "Total request {} processed in {:?}",
                request.id,
                request_start.elapsed()
            );
        }

        Ok(())
    }

    /// Send a response from the worker to the main process.
    fn send_response(response: &WorkerResponse) -> Result<(), MultiThreadedRenderPoolError> {
        use std::io::Write;

        let encoded = bincode::serialize(response)
            .map_err(|e| MultiThreadedRenderPoolError::SerializationError(e.to_string()))?;

        // Send length prefix followed by data
        let len = encoded.len() as u32;
        let mut stdout = std::io::stdout();
        stdout.write_all(&len.to_le_bytes())?;
        stdout.write_all(&encoded)?;
        stdout.flush()?;

        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum MultiThreadedRenderPoolError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    RenderingError(#[from] RenderingError),

    #[error("Worker spawn error: {0}")]
    WorkerSpawnError(String),

    #[error("Worker error: {0}")]
    WorkerError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Image decode error: {0}")]
    ImageDecodeError(String),

    #[error("Failed to receive response from worker")]
    FailedToReceiveResponse,
}
