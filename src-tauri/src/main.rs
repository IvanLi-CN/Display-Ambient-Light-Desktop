// Prevents additional console window on WiOk(ndows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ambient_light;
mod display;
mod led_color;
mod led_test_effects;
mod rpc;
mod screen_stream;
mod screenshot;
mod screenshot_manager;
mod volume;

use ambient_light::{Border, ColorCalibration, LedStripConfig, LedStripConfigGroup, LedType};
use display::{DisplayManager, DisplayState};
use display_info::DisplayInfo;
use led_test_effects::{LedTestEffects, TestEffectConfig};
use paris::{error, info, warn};
use rpc::{BoardInfo, UdpRpc};
use screenshot::Screenshot;
use screenshot_manager::ScreenshotManager;

use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::sync::Arc;
use tauri::http::{Request, Response};
use tauri::{Emitter, Runtime};
use tokio::sync::RwLock;
use volume::VolumeManager;

// Global static variables for LED test effect management
static EFFECT_HANDLE: tokio::sync::OnceCell<Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>> =
    tokio::sync::OnceCell::const_new();
static CANCEL_TOKEN: tokio::sync::OnceCell<
    Arc<RwLock<Option<tokio_util::sync::CancellationToken>>>,
> = tokio::sync::OnceCell::const_new();
#[derive(Serialize, Deserialize)]
#[serde(remote = "DisplayInfo")]
struct DisplayInfoDef {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub rotation: f32,
    pub scale_factor: f32,
    pub is_primary: bool,
    pub frequency: f32,
    #[serde(skip, default = "_default_cg_display")]
    pub raw_handle: core_graphics::display::CGDisplay,
}

fn _default_cg_display() -> core_graphics::display::CGDisplay {
    // Default display for serde deserialization
    core_graphics::display::CGDisplay::main()
}

#[derive(Serialize)]
struct DisplayInfoWrapper<'a>(#[serde(with = "DisplayInfoDef")] &'a DisplayInfo);

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

#[derive(Serialize)]
struct AppVersion {
    version: String,
    is_dev: bool,
}

#[tauri::command]
fn get_app_version() -> AppVersion {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let is_dev = cfg!(debug_assertions) && std::env::var("HIDE_DEV_MARKER").is_err();

    AppVersion { version, is_dev }
}

#[tauri::command]
fn list_display_info() -> Result<String, String> {
    let displays = display_info::DisplayInfo::all().map_err(|e| {
        error!("can not list display info: {}", e);
        e.to_string()
    })?;
    let displays: Vec<DisplayInfoWrapper> = displays.iter().map(DisplayInfoWrapper).collect();
    let json_str = to_string(&displays).map_err(|e| {
        error!("can not list display info: {}", e);
        e.to_string()
    })?;
    Ok(json_str)
}

#[tauri::command]
async fn read_led_strip_configs() -> Result<LedStripConfigGroup, String> {
    let config = ambient_light::LedStripConfigGroup::read_config()
        .await
        .map_err(|e| {
            error!("can not read led strip configs: {}", e);
            e.to_string()
        })?;
    Ok(config)
}

#[tauri::command]
async fn write_led_strip_configs(
    configs: Vec<ambient_light::LedStripConfig>,
) -> Result<(), String> {
    let config_manager = ambient_light::ConfigManager::global().await;

    config_manager.set_items(configs).await.map_err(|e| {
        error!("can not write led strip configs: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn get_led_strips_sample_points(
    config: LedStripConfig,
) -> Result<Vec<screenshot::LedSamplePoints>, String> {
    let screenshot_manager = ScreenshotManager::global().await;
    let channels = screenshot_manager.channels.read().await;
    if let Some(rx) = channels.get(&config.display_id) {
        let rx = rx.read().await;
        let screenshot = rx.borrow().clone();
        let sample_points = screenshot.get_sample_points(&config);
        Ok(sample_points)
    } else {
        Err(format!("display not found: {}", config.display_id))
    }
}

#[tauri::command]
async fn get_one_edge_colors(
    display_id: u32,
    sample_points: Vec<screenshot::LedSamplePoints>,
) -> Result<Vec<led_color::LedColor>, String> {
    let screenshot_manager = ScreenshotManager::global().await;
    let channels = screenshot_manager.channels.read().await;
    if let Some(rx) = channels.get(&display_id) {
        let rx = rx.read().await;
        let screenshot = rx.borrow().clone();
        let bytes = screenshot.bytes.read().await.to_owned();
        let colors =
            Screenshot::get_one_edge_colors(&sample_points, &bytes, screenshot.bytes_per_row);
        Ok(colors)
    } else {
        Err(format!("display not found: {display_id}"))
    }
}

#[tauri::command]
async fn patch_led_strip_len(display_id: u32, border: Border, delta_len: i8) -> Result<(), String> {
    info!(
        "patch_led_strip_len: {} {:?} {}",
        display_id, border, delta_len
    );
    let config_manager = ambient_light::ConfigManager::global().await;
    config_manager
        .patch_led_strip_len(display_id, border, delta_len)
        .await
        .map_err(|e| {
            error!("can not patch led strip len: {}", e);
            e.to_string()
        })?;

    info!("patch_led_strip_len: ok");
    Ok(())
}

#[tauri::command]
async fn patch_led_strip_type(
    display_id: u32,
    border: Border,
    led_type: LedType,
) -> Result<(), String> {
    let config_manager = ambient_light::ConfigManager::global().await;
    config_manager
        .patch_led_strip_type(display_id, border, led_type)
        .await
        .map_err(|e| {
            error!("can not patch led strip type: {}", e);
            e.to_string()
        })?;

    Ok(())
}

#[tauri::command]
async fn send_colors(offset: u16, buffer: Vec<u8>) -> Result<(), String> {
    ambient_light::LedColorsPublisher::send_colors(offset, buffer)
        .await
        .map_err(|e| {
            error!("can not send colors: {}", e);
            e.to_string()
        })
}

#[tauri::command]
async fn send_test_colors_to_board(
    board_address: String,
    offset: u16,
    buffer: Vec<u8>,
) -> Result<(), String> {
    use tokio::net::UdpSocket;

    let socket = UdpSocket::bind("0.0.0.0:0").await.map_err(|e| {
        error!("Failed to bind UDP socket: {}", e);
        e.to_string()
    })?;

    let mut packet = vec![0x02]; // Header
    packet.push((offset >> 8) as u8); // Byte offset high
    packet.push((offset & 0xff) as u8); // Byte offset low
    packet.extend_from_slice(&buffer); // Color data

    socket.send_to(&packet, &board_address).await.map_err(|e| {
        error!(
            "Failed to send test colors to board {}: {}",
            board_address, e
        );
        e.to_string()
    })?;

    info!(
        "Sent test colors to board {} with offset {} and {} bytes",
        board_address,
        offset,
        buffer.len()
    );
    Ok(())
}

#[tauri::command]
async fn enable_test_mode() -> Result<(), String> {
    let publisher = ambient_light::LedColorsPublisher::global().await;
    publisher.enable_test_mode().await;
    Ok(())
}

#[tauri::command]
async fn disable_test_mode() -> Result<(), String> {
    info!("🔄 disable_test_mode command called from frontend");
    let publisher = ambient_light::LedColorsPublisher::global().await;
    publisher.disable_test_mode().await;
    info!("✅ disable_test_mode command completed");
    Ok(())
}

#[tauri::command]
async fn is_test_mode_active() -> Result<bool, String> {
    let publisher = ambient_light::LedColorsPublisher::global().await;
    Ok(publisher.is_test_mode_active().await)
}

#[tauri::command]
async fn start_led_test_effect(
    board_address: String,
    effect_config: TestEffectConfig,
    update_interval_ms: u64,
) -> Result<(), String> {
    use tokio::time::{interval, Duration};

    // Enable test mode first
    let publisher = ambient_light::LedColorsPublisher::global().await;
    publisher.enable_test_mode().await;

    let handle_storage = EFFECT_HANDLE
        .get_or_init(|| async { Arc::new(RwLock::new(None)) })
        .await;

    let cancel_storage = CANCEL_TOKEN
        .get_or_init(|| async { Arc::new(RwLock::new(None)) })
        .await;

    // Stop any existing effect
    {
        let mut cancel_guard = cancel_storage.write().await;
        if let Some(token) = cancel_guard.take() {
            token.cancel();
        }

        let mut handle_guard = handle_storage.write().await;
        if let Some(handle) = handle_guard.take() {
            let _ = handle.await; // Wait for graceful shutdown
        }
    }

    // Start new effect
    let effect_config = Arc::new(effect_config);
    let board_address = Arc::new(board_address);
    let start_time = std::time::Instant::now();

    // Create new cancellation token
    let cancel_token = tokio_util::sync::CancellationToken::new();
    let cancel_token_clone = cancel_token.clone();

    let handle = tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(update_interval_ms));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let elapsed_ms = start_time.elapsed().as_millis() as u64;
                    let colors = LedTestEffects::generate_colors(&effect_config, elapsed_ms);

                    // Send to board
                    if let Err(e) = send_test_colors_to_board_internal(&board_address, 0, colors).await {
                        error!("Failed to send test effect colors: {}", e);
                        break;
                    }
                }
                _ = cancel_token_clone.cancelled() => {
                    info!("LED test effect cancelled gracefully");
                    break;
                }
            }
        }
        info!("LED test effect task ended");
    });

    // Store the handle and cancel token
    {
        let mut handle_guard = handle_storage.write().await;
        *handle_guard = Some(handle);

        let mut cancel_guard = cancel_storage.write().await;
        *cancel_guard = Some(cancel_token);
    }

    Ok(())
}

#[tauri::command]
async fn stop_led_test_effect(
    board_address: String,
    led_count: u32,
    led_type: led_test_effects::LedType,
) -> Result<(), String> {
    // Stop the effect task first

    info!("🛑 Stopping LED test effect - board: {}", board_address);

    // Cancel the task gracefully first
    if let Some(cancel_storage) = CANCEL_TOKEN.get() {
        let mut cancel_guard = cancel_storage.write().await;
        if let Some(token) = cancel_guard.take() {
            info!("🔄 Cancelling test effect task gracefully");
            token.cancel();
        }
    }

    // Wait for the task to finish
    if let Some(handle_storage) = EFFECT_HANDLE.get() {
        let mut handle_guard = handle_storage.write().await;
        if let Some(handle) = handle_guard.take() {
            info!("⏳ Waiting for test effect task to finish");
            match handle.await {
                Ok(_) => info!("✅ Test effect task finished successfully"),
                Err(e) => warn!("⚠️ Test effect task finished with error: {}", e),
            }
        }
    }

    // Turn off all LEDs
    let bytes_per_led = match led_type {
        led_test_effects::LedType::WS2812B => 3,
        led_test_effects::LedType::SK6812 => 4,
    };
    let buffer = vec![0u8; (led_count * bytes_per_led) as usize];

    send_test_colors_to_board_internal(&board_address, 0, buffer)
        .await
        .map_err(|e| e.to_string())?;

    info!("💡 Sent LED off command");

    // Disable test mode to resume normal publishing
    let publisher = ambient_light::LedColorsPublisher::global().await;
    publisher.disable_test_mode().await;

    info!("🔄 Test mode disabled, normal publishing resumed");
    info!("✅ LED test effect stopped completely");

    Ok(())
}

// Internal helper function
async fn send_test_colors_to_board_internal(
    board_address: &str,
    offset: u16,
    buffer: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::net::UdpSocket;

    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    let mut packet = vec![0x02]; // Header
    packet.push((offset >> 8) as u8); // Byte offset high
    packet.push((offset & 0xff) as u8); // Byte offset low
    packet.extend_from_slice(&buffer); // Color data

    socket.send_to(&packet, board_address).await?;
    Ok(())
}

#[tauri::command]
async fn move_strip_part(
    display_id: u32,
    border: Border,
    target_start: usize,
) -> Result<(), String> {
    let config_manager = ambient_light::ConfigManager::global().await;
    config_manager
        .move_strip_part(display_id, border, target_start)
        .await
        .map_err(|e| {
            error!("can not move strip part: {}", e);
            e.to_string()
        })
}

#[tauri::command]
async fn reverse_led_strip_part(display_id: u32, border: Border) -> Result<(), String> {
    let config_manager = ambient_light::ConfigManager::global().await;
    config_manager
        .reverse_led_strip_part(display_id, border)
        .await
        .map_err(|e| {
            error!("can not reverse led strip part: {}", e);
            e.to_string()
        })
}

#[tauri::command]
async fn set_color_calibration(calibration: ColorCalibration) -> Result<(), String> {
    let config_manager = ambient_light::ConfigManager::global().await;
    config_manager
        .set_color_calibration(calibration)
        .await
        .map_err(|e| {
            error!("can not set color calibration: {}", e);
            e.to_string()
        })
}

#[tauri::command]
async fn read_config() -> ambient_light::LedStripConfigGroup {
    let config_manager = ambient_light::ConfigManager::global().await;
    config_manager.configs().await
}

#[tauri::command]
async fn get_boards() -> Result<Vec<BoardInfo>, String> {
    let udp_rpc = UdpRpc::global().await;

    if let Err(e) = udp_rpc {
        return Err(format!("can not ping: {e}"));
    }

    let udp_rpc = udp_rpc.as_ref().unwrap();

    let boards = udp_rpc.get_boards().await;
    let boards = boards.into_iter().collect::<Vec<_>>();
    Ok(boards)
}

#[tauri::command]
async fn get_displays() -> Vec<DisplayState> {
    let display_manager = DisplayManager::global().await;

    display_manager.get_displays().await
}

// Protocol handler for ambient-light://
fn handle_ambient_light_protocol<R: Runtime>(
    _ctx: tauri::UriSchemeContext<R>,
    request: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let url = request.uri();
    // info!("Handling ambient-light protocol request: {}", url);

    // Parse the URL to extract parameters
    let url_str = url.to_string();
    let re =
        regex::Regex::new(r"ambient-light://displays/(\d+)\?width=(\d+)&height=(\d+)").unwrap();

    if let Some(captures) = re.captures(&url_str) {
        let display_id: u32 = captures[1].parse().unwrap_or(0);
        let width: u32 = captures[2].parse().unwrap_or(400);
        let height: u32 = captures[3].parse().unwrap_or(300);

        // info!("Efficient screenshot request for display {}, {}x{}", display_id, width, height);

        // Optimized screenshot processing with much smaller intermediate size
        // info!("Screenshot request received: display_id={}, width={}, height={}", display_id, width, height);

        let screenshot_data = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let screenshot_manager = ScreenshotManager::global().await;
                let channels = screenshot_manager.channels.read().await;

                if let Some(rx) = channels.get(&display_id) {
                    let rx = rx.read().await;
                    let screenshot = rx.borrow().clone();
                    let bytes = screenshot.bytes.read().await.to_owned();

                    // Use much smaller intermediate resolution for performance
                    let intermediate_width = 800; // Much smaller than original 5120
                    let intermediate_height = 450; // Much smaller than original 2880

                    // Convert BGRA to RGBA format
                    let mut rgba_bytes = bytes.as_ref().clone();
                    for chunk in rgba_bytes.chunks_exact_mut(4) {
                        chunk.swap(0, 2); // Swap B and R channels
                    }

                    let image_result = image::RgbaImage::from_raw(
                        screenshot.width as u32,
                        screenshot.height as u32,
                        rgba_bytes,
                    );

                    if let Some(img) = image_result {
                        // Step 1: Fast downscale to intermediate size
                        let intermediate_image = image::imageops::resize(
                            &img,
                            intermediate_width,
                            intermediate_height,
                            image::imageops::FilterType::Nearest, // Fastest possible
                        );

                        // Step 2: Scale to final target size
                        let final_image =
                            if width == intermediate_width && height == intermediate_height {
                                intermediate_image
                            } else {
                                image::imageops::resize(
                                    &intermediate_image,
                                    width,
                                    height,
                                    image::imageops::FilterType::Triangle,
                                )
                            };

                        let raw_data = final_image.into_raw();
                        // info!("Efficient resize completed: {}x{}, {} bytes", width, height, raw_data.len());
                        Ok(raw_data)
                    } else {
                        error!("Failed to create image from raw bytes");
                        Err("Failed to create image from raw bytes".to_string())
                    }
                } else {
                    error!("Display {} not found", display_id);
                    Err(format!("Display {display_id} not found"))
                }
            })
        });

        match screenshot_data {
            Ok(data) => Response::builder()
                .header("Content-Type", "application/octet-stream")
                .header("Access-Control-Allow-Origin", "*")
                .header("X-Image-Width", width.to_string())
                .header("X-Image-Height", height.to_string())
                .body(data)
                .unwrap_or_else(|_| {
                    Response::builder()
                        .status(500)
                        .body("Failed to build response".as_bytes().to_vec())
                        .unwrap()
                }),
            Err(e) => {
                error!("Failed to get screenshot: {}", e);
                Response::builder()
                    .status(500)
                    .body(format!("Error: {e}").into_bytes())
                    .unwrap()
            }
        }
    } else {
        warn!("Invalid ambient-light URL format: {}", url_str);
        Response::builder()
            .status(400)
            .body("Invalid URL format".as_bytes().to_vec())
            .unwrap()
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    // Initialize display info (removed debug output)

    tokio::spawn(async move {
        let screenshot_manager = ScreenshotManager::global().await;
        screenshot_manager.start().await.unwrap_or_else(|e| {
            error!("can not start screenshot manager: {}", e);
        })
    });

    tokio::spawn(async move {
        let led_color_publisher = ambient_light::LedColorsPublisher::global().await;
        led_color_publisher.start().await;
    });

    // Start WebSocket server for screen streaming
    tokio::spawn(async move {
        if let Err(e) = start_websocket_server().await {
            error!("Failed to start WebSocket server: {}", e);
        }
    });

    let _volume = VolumeManager::global().await;

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_app_version,
            list_display_info,
            read_led_strip_configs,
            write_led_strip_configs,
            get_led_strips_sample_points,
            get_one_edge_colors,
            patch_led_strip_len,
            patch_led_strip_type,
            send_colors,
            send_test_colors_to_board,
            enable_test_mode,
            disable_test_mode,
            is_test_mode_active,
            start_led_test_effect,
            stop_led_test_effect,
            move_strip_part,
            reverse_led_strip_part,
            set_color_calibration,
            read_config,
            get_boards,
            get_displays
        ])
        .register_uri_scheme_protocol("ambient-light", handle_ambient_light_protocol)
        .setup(move |app| {
            let app_handle = app.handle().clone();
            tokio::spawn(async move {
                let config_manager = ambient_light::ConfigManager::global().await;
                let mut config_update_receiver = config_manager.clone_config_update_receiver();
                loop {
                    if let Err(err) = config_update_receiver.changed().await {
                        error!("config update receiver changed error: {}", err);
                        return;
                    }

                    log::info!("config changed. emit config_changed event.");

                    let config = config_update_receiver.borrow().clone();

                    app_handle.emit("config_changed", config).unwrap();
                }
            });

            let app_handle = app.handle().clone();
            tokio::spawn(async move {
                let publisher = ambient_light::LedColorsPublisher::global().await;
                let mut publisher_update_receiver = publisher.clone_sorted_colors_receiver().await;
                loop {
                    if let Err(err) = publisher_update_receiver.changed().await {
                        error!("publisher update receiver changed error: {}", err);
                        return;
                    }

                    let publisher = publisher_update_receiver.borrow().clone();

                    app_handle
                        .emit("led_sorted_colors_changed", publisher)
                        .unwrap();
                }
            });

            let app_handle = app.handle().clone();
            tokio::spawn(async move {
                let publisher = ambient_light::LedColorsPublisher::global().await;
                let mut publisher_update_receiver = publisher.clone_colors_receiver().await;
                loop {
                    if let Err(err) = publisher_update_receiver.changed().await {
                        error!("publisher update receiver changed error: {}", err);
                        return;
                    }

                    let publisher = publisher_update_receiver.borrow().clone();

                    app_handle.emit("led_colors_changed", publisher).unwrap();
                }
            });

            let app_handle = app.handle().clone();
            tokio::spawn(async move {
                match UdpRpc::global().await {
                    Ok(udp_rpc) => {
                        let mut receiver = udp_rpc.subscribe_boards_change();
                        loop {
                            if let Err(err) = receiver.changed().await {
                                error!("boards change receiver changed error: {}", err);
                                return;
                            }

                            let boards = receiver.borrow().clone();

                            let boards = boards.into_iter().collect::<Vec<_>>();

                            app_handle.emit("boards_changed", boards).unwrap();
                        }
                    }
                    Err(err) => {
                        error!("udp rpc error: {}", err);
                    }
                }
            });

            let app_handle = app.handle().clone();
            tokio::spawn(async move {
                let display_manager = DisplayManager::global().await;
                let mut rx = display_manager.subscribe_displays_changed();

                while rx.changed().await.is_ok() {
                    let displays = rx.borrow().clone();

                    log::info!("displays changed. emit displays_changed event.");

                    app_handle.emit("displays_changed", displays).unwrap();
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// WebSocket server for screen streaming
async fn start_websocket_server() -> anyhow::Result<()> {
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:8765").await?;
    info!("WebSocket server listening on ws://127.0.0.1:8765");

    while let Ok((stream, addr)) = listener.accept().await {
        info!("New WebSocket connection from: {}", addr);

        tokio::spawn(async move {
            info!("Starting WebSocket handler for connection from: {}", addr);
            match screen_stream::handle_websocket_connection(stream).await {
                Ok(_) => {
                    info!("WebSocket connection from {} completed successfully", addr);
                }
                Err(e) => {
                    warn!("WebSocket connection error from {}: {}", addr, e);
                }
            }
            info!("WebSocket handler task completed for: {}", addr);
        });
    }

    Ok(())
}
