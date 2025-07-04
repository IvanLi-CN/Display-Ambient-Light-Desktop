use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::io::Cursor;

use anyhow::Result;
use image::{ImageFormat, RgbaImage};
use tokio::sync::{broadcast, RwLock};
use tokio::time::sleep;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

use crate::screenshot::Screenshot;
use crate::screenshot_manager::ScreenshotManager;

#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub display_id: u32,
    pub target_width: u32,
    pub target_height: u32,
    pub quality: u8,  // JPEG quality 1-100
    pub max_fps: u8,  // Maximum frames per second
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            display_id: 0,
            target_width: 320,  // Reduced from 400 for better performance
            target_height: 180, // Reduced from 225 for better performance
            quality: 50,  // Reduced from 75 for faster compression
            max_fps: 15,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StreamFrame {
    pub display_id: u32,
    pub timestamp: Instant,
    pub jpeg_data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub struct ScreenStreamManager {
    streams: Arc<RwLock<HashMap<u32, Arc<RwLock<StreamState>>>>>,
}

struct StreamState {
    config: StreamConfig,
    subscribers: Vec<broadcast::Sender<StreamFrame>>,
    last_frame: Option<StreamFrame>,
    last_screenshot_hash: Option<u64>,
    last_force_send: Instant,
    is_running: bool,
}

impl ScreenStreamManager {
    pub fn new() -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_stream(&self, config: StreamConfig) -> Result<broadcast::Receiver<StreamFrame>> {
        let display_id = config.display_id;
        let mut streams = self.streams.write().await;

        if let Some(stream_state) = streams.get(&display_id) {
            // Stream already exists, just add a new subscriber
            let mut state = stream_state.write().await;
            let (tx, rx) = broadcast::channel(10);
            state.subscribers.push(tx);
            return Ok(rx);
        }

        // Create new stream
        let (tx, rx) = broadcast::channel(10);
        let stream_state = Arc::new(RwLock::new(StreamState {
            config: config.clone(),
            subscribers: vec![tx],
            last_frame: None,
            last_screenshot_hash: None,
            last_force_send: Instant::now(),
            is_running: false,
        }));

        streams.insert(display_id, stream_state.clone());
        drop(streams);

        // Start the stream processing task
        let streams_ref = self.streams.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::run_stream(display_id, streams_ref).await {
                log::error!("Stream {} error: {}", display_id, e);
            }
        });

        Ok(rx)
    }

    async fn run_stream(display_id: u32, streams: Arc<RwLock<HashMap<u32, Arc<RwLock<StreamState>>>>>) -> Result<()> {
        log::info!("Starting stream for display_id: {}", display_id);

        let screenshot_manager = ScreenshotManager::global().await;

        // If display_id is 0, try to get the first available display
        let actual_display_id = if display_id == 0 {
            // Get available displays and use the first one
            let displays = display_info::DisplayInfo::all().map_err(|e| anyhow::anyhow!("Failed to get displays: {}", e))?;
            if displays.is_empty() {
                return Err(anyhow::anyhow!("No displays available"));
            }
            log::info!("Using first available display: {}", displays[0].id);
            displays[0].id
        } else {
            display_id
        };

        log::info!("Attempting to subscribe to display_id: {}", actual_display_id);
        let screenshot_rx = match screenshot_manager.subscribe_by_display_id(actual_display_id).await {
            Ok(rx) => {
                log::info!("Successfully subscribed to display_id: {}", actual_display_id);
                rx
            }
            Err(e) => {
                log::error!("Failed to subscribe to display_id {}: {}", actual_display_id, e);
                return Err(e);
            }
        };
        let mut screenshot_rx = screenshot_rx;

        // Mark stream as running
        {
            let streams_lock = streams.read().await;
            if let Some(stream_state) = streams_lock.get(&display_id) {
                let mut state = stream_state.write().await;
                state.is_running = true;
            }
        }

        let mut last_process_time = Instant::now();

        loop {
            // Check if stream still has subscribers
            let should_continue = {
                let streams_lock = streams.read().await;
                if let Some(stream_state) = streams_lock.get(&display_id) {
                    let state = stream_state.read().await;
                    !state.subscribers.is_empty()
                } else {
                    false
                }
            };

            if !should_continue {
                break;
            }

            // Wait for new screenshot
            if let Ok(_) = screenshot_rx.changed().await {
                let screenshot = screenshot_rx.borrow().clone();
                
                // Rate limiting based on max_fps
                let config = {
                    let streams_lock = streams.read().await;
                    if let Some(stream_state) = streams_lock.get(&display_id) {
                        let state = stream_state.read().await;
                        state.config.clone()
                    } else {
                        break;
                    }
                };

                let min_interval = Duration::from_millis(1000 / config.max_fps as u64);
                let elapsed = last_process_time.elapsed();
                if elapsed < min_interval {
                    sleep(min_interval - elapsed).await;
                }

                // Process screenshot into JPEG frame
                if let Ok(frame) = Self::process_screenshot(&screenshot, &config).await {
                    last_process_time = Instant::now();
                    
                    // Check if frame content changed (simple hash comparison) or force send
                    let frame_hash = Self::calculate_frame_hash(&frame.jpeg_data);
                    let should_send = {
                        let streams_lock = streams.read().await;
                        if let Some(stream_state) = streams_lock.get(&display_id) {
                            let mut state = stream_state.write().await;
                            let changed = state.last_screenshot_hash.map_or(true, |hash| hash != frame_hash);
                            let elapsed_ms = state.last_force_send.elapsed().as_millis();
                            let force_send = elapsed_ms > 200; // Force send every 200ms for higher FPS

                            if changed || force_send {
                                state.last_screenshot_hash = Some(frame_hash);
                                state.last_frame = Some(frame.clone());
                                if force_send {
                                    state.last_force_send = Instant::now();
                                }
                            }
                            changed || force_send
                        } else {
                            false
                        }
                    };

                    if should_send {
                        // Send to all subscribers
                        let streams_lock = streams.read().await;
                        if let Some(stream_state) = streams_lock.get(&display_id) {
                            let state = stream_state.read().await;
                            for tx in state.subscribers.iter() {
                                if let Err(_) = tx.send(frame.clone()) {
                                    log::warn!("Failed to send frame to subscriber for display_id: {}", display_id);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Mark stream as stopped
        {
            let streams_lock = streams.read().await;
            if let Some(stream_state) = streams_lock.get(&display_id) {
                let mut state = stream_state.write().await;
                state.is_running = false;
            }
        }

        Ok(())
    }

    async fn process_screenshot(screenshot: &Screenshot, config: &StreamConfig) -> Result<StreamFrame> {
        let total_start = Instant::now();
        let bytes = screenshot.bytes.read().await;

        // Convert BGRA to RGBA using unsafe with optimized batch processing for maximum performance
        let mut rgba_bytes = bytes.as_ref().clone();
        unsafe {
            let ptr = rgba_bytes.as_mut_ptr() as *mut u32;
            let len = rgba_bytes.len() / 4;

            // Process in larger chunks of 64 for better cache efficiency and loop unrolling
            let chunk_size = 64;
            let full_chunks = len / chunk_size;
            let remainder = len % chunk_size;

            // Process full chunks with manual loop unrolling
            for chunk_idx in 0..full_chunks {
                let base_ptr = ptr.add(chunk_idx * chunk_size);

                // Unroll the inner loop for better performance
                for i in (0..chunk_size).step_by(4) {
                    // Process 4 pixels at once
                    let p0 = base_ptr.add(i).read();
                    let p1 = base_ptr.add(i + 1).read();
                    let p2 = base_ptr.add(i + 2).read();
                    let p3 = base_ptr.add(i + 3).read();

                    // BGRA (0xAABBGGRR) -> RGBA (0xAAGGBBRR)
                    let s0 = (p0 & 0xFF00FF00) | ((p0 & 0x00FF0000) >> 16) | ((p0 & 0x000000FF) << 16);
                    let s1 = (p1 & 0xFF00FF00) | ((p1 & 0x00FF0000) >> 16) | ((p1 & 0x000000FF) << 16);
                    let s2 = (p2 & 0xFF00FF00) | ((p2 & 0x00FF0000) >> 16) | ((p2 & 0x000000FF) << 16);
                    let s3 = (p3 & 0xFF00FF00) | ((p3 & 0x00FF0000) >> 16) | ((p3 & 0x000000FF) << 16);

                    base_ptr.add(i).write(s0);
                    base_ptr.add(i + 1).write(s1);
                    base_ptr.add(i + 2).write(s2);
                    base_ptr.add(i + 3).write(s3);
                }
            }

            // Process remaining pixels
            let remainder_start = full_chunks * chunk_size;
            for i in 0..remainder {
                let idx = remainder_start + i;
                let pixel = ptr.add(idx).read();
                let swapped = (pixel & 0xFF00FF00) | ((pixel & 0x00FF0000) >> 16) | ((pixel & 0x000000FF) << 16);
                ptr.add(idx).write(swapped);
            }
        }

        // Create image from raw bytes
        let img = RgbaImage::from_raw(
            screenshot.width,
            screenshot.height,
            rgba_bytes,
        ).ok_or_else(|| anyhow::anyhow!("Failed to create image from raw bytes"))?;

        // Resize if needed
        let final_img = if screenshot.width != config.target_width || screenshot.height != config.target_height {
            image::imageops::resize(
                &img,
                config.target_width,
                config.target_height,
                image::imageops::FilterType::Nearest, // Fastest filter for real-time streaming
            )
        } else {
            img
        };

        // Convert to JPEG
        let mut jpeg_buffer = Vec::new();
        let mut cursor = Cursor::new(&mut jpeg_buffer);

        let rgb_img = image::DynamicImage::ImageRgba8(final_img).to_rgb8();
        rgb_img.write_to(&mut cursor, ImageFormat::Jpeg)?;

        let total_duration = total_start.elapsed();
        log::debug!("Screenshot processed for display {} in {}ms, JPEG size: {} bytes",
                   config.display_id, total_duration.as_millis(), jpeg_buffer.len());

        Ok(StreamFrame {
            display_id: config.display_id,
            timestamp: Instant::now(),
            jpeg_data: jpeg_buffer,
            width: config.target_width,
            height: config.target_height,
        })
    }

    fn calculate_frame_hash(data: &[u8]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        // Sample every 100th byte for better sensitivity (was 1000)
        for (i, &byte) in data.iter().enumerate() {
            if i % 100 == 0 {
                byte.hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    pub async fn stop_stream(&self, display_id: u32) {
        let mut streams = self.streams.write().await;
        streams.remove(&display_id);
    }
}

// Global instance
static SCREEN_STREAM_MANAGER: tokio::sync::OnceCell<ScreenStreamManager> = tokio::sync::OnceCell::const_new();

impl ScreenStreamManager {
    pub async fn global() -> &'static Self {
        SCREEN_STREAM_MANAGER.get_or_init(|| async {
            ScreenStreamManager::new()
        }).await
    }
}

// WebSocket handler for screen streaming
pub async fn handle_websocket_connection(
    stream: tokio::net::TcpStream,
) -> Result<()> {
    log::info!("Accepting WebSocket connection...");

    let ws_stream = match accept_async(stream).await {
        Ok(ws) => {
            log::info!("WebSocket handshake completed successfully");
            ws
        }
        Err(e) => {
            log::error!("WebSocket handshake failed: {}", e);
            return Err(e.into());
        }
    };
    let (ws_sender, mut ws_receiver) = ws_stream.split();

    log::info!("WebSocket connection established, waiting for configuration...");

    // Wait for the first configuration message
    let config = loop {
        // Add timeout to prevent hanging
        let timeout_duration = tokio::time::Duration::from_secs(10);
        match tokio::time::timeout(timeout_duration, ws_receiver.next()).await {
            Ok(Some(msg)) => {
                match msg {
                Ok(Message::Text(text)) => {
                    log::info!("Received configuration message: {}", text);

                    if let Ok(config_json) = serde_json::from_str::<serde_json::Value>(&text) {
                        // Parse configuration from JSON
                        let display_id = config_json.get("display_id")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as u32;
                        let width = config_json.get("width")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(320) as u32;  // Reduced from 400 for better performance
                        let height = config_json.get("height")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(180) as u32;  // Reduced from 225 for better performance
                        let quality = config_json.get("quality")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(50) as u8;  // Reduced from 75 for faster compression

                        let config = StreamConfig {
                            display_id,
                            target_width: width,
                            target_height: height,
                            quality,
                            max_fps: 15,
                        };

                        log::info!("Parsed stream config: display_id={}, width={}, height={}, quality={}",
                                  display_id, width, height, quality);
                        break config;
                    } else {
                        log::warn!("Failed to parse configuration JSON: {}", text);
                    }
                }
                Ok(Message::Close(_)) => {
                    log::info!("WebSocket connection closed before configuration");
                    return Ok(());
                }
                Err(e) => {
                    log::warn!("WebSocket error while waiting for config: {}", e);
                    return Err(e.into());
                }
                _ => {}
                }
            }
            Ok(None) => {
                log::warn!("WebSocket connection closed while waiting for configuration");
                return Ok(());
            }
            Err(_) => {
                log::warn!("Timeout waiting for WebSocket configuration message");
                return Err(anyhow::anyhow!("Timeout waiting for configuration"));
            }
        }
    };

    // Start the stream with the received configuration
    log::info!("Starting stream with config: display_id={}, width={}, height={}",
               config.display_id, config.target_width, config.target_height);
    let stream_manager = ScreenStreamManager::global().await;
    let mut frame_rx = match stream_manager.start_stream(config).await {
        Ok(rx) => {
            log::info!("Screen stream started successfully");
            rx
        }
        Err(e) => {
            log::error!("Failed to start screen stream: {}", e);
            return Err(e);
        }
    };

    // Handle incoming WebSocket messages (for control)
    let ws_sender = Arc::new(tokio::sync::Mutex::new(ws_sender));
    let ws_sender_clone = ws_sender.clone();

    // Task to handle outgoing frames
    let frame_task = tokio::spawn(async move {
        while let Ok(frame) = frame_rx.recv().await {
            let mut sender = ws_sender_clone.lock().await;
            match sender.send(Message::Binary(frame.jpeg_data)).await {
                Ok(_) => {},
                Err(e) => {
                    log::warn!("Failed to send frame: {}", e);
                    break;
                }
            }
        }
        log::info!("Frame sending task completed");
    });

    // Task to handle incoming messages
    let control_task = tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    log::info!("Received control message: {}", text);
                    // Additional configuration updates could be handled here
                }
                Ok(Message::Close(_)) => {
                    log::info!("WebSocket connection closed");
                    break;
                }
                Err(e) => {
                    log::warn!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
        log::info!("Control message task completed");
    });

    // Wait for either task to complete
    tokio::select! {
        _ = frame_task => {},
        _ = control_task => {},
    }

    log::info!("WebSocket connection handler completed");
    Ok(())
}
