// Prevents additional console window on WiOk(ndows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ambient_light;
mod display;
mod led_color;
mod rpc;
mod screenshot;
mod screenshot_manager;
mod volume;

use ambient_light::{Border, ColorCalibration, LedStripConfig, LedStripConfigGroup};
use display::{DisplayManager, DisplayState};
use display_info::DisplayInfo;
use paris::{error, info, warn};
use rpc::{BoardInfo, UdpRpc};
use screenshot::Screenshot;
use screenshot_manager::ScreenshotManager;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use tauri::{Manager, Emitter, Runtime};
use regex;
use tauri::http::{Request, Response};
use volume::VolumeManager;
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
}

#[derive(Serialize)]
struct DisplayInfoWrapper<'a>(#[serde(with = "DisplayInfoDef")] &'a DisplayInfo);

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn list_display_info() -> Result<String, String> {
    let displays = display_info::DisplayInfo::all().map_err(|e| {
        error!("can not list display info: {}", e);
        e.to_string()
    })?;
    let displays: Vec<DisplayInfoWrapper> =
        displays.iter().map(|v| DisplayInfoWrapper(v)).collect();
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
        return Err(format!("display not found: {}", config.display_id));
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
        Err(format!("display not found: {}", display_id))
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
async fn send_colors(offset: u16, buffer: Vec<u8>) -> Result<(), String> {
    ambient_light::LedColorsPublisher::send_colors(offset, buffer)
        .await
        .map_err(|e| {
            error!("can not send colors: {}", e);
            e.to_string()
        })
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
        return Err(format!("can not ping: {}", e));
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
    request: Request<Vec<u8>>
) -> Response<Vec<u8>> {
    let url = request.uri();
    // info!("Handling ambient-light protocol request: {}", url);

    // Parse the URL to extract parameters
    let url_str = url.to_string();
    let re = regex::Regex::new(r"ambient-light://displays/(\d+)\?width=(\d+)&height=(\d+)").unwrap();

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
                    let intermediate_width = 800;  // Much smaller than original 5120
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
                        let final_image = if width == intermediate_width && height == intermediate_height {
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
                    Err(format!("Display {} not found", display_id))
                }
            })
        });

        match screenshot_data {
            Ok(data) => {
                Response::builder()
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
                    })
            }
            Err(e) => {
                error!("Failed to get screenshot: {}", e);
                Response::builder()
                    .status(500)
                    .body(format!("Error: {}", e).into_bytes())
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

    let _volume = VolumeManager::global().await;

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            list_display_info,
            read_led_strip_configs,
            write_led_strip_configs,
            get_led_strips_sample_points,
            get_one_edge_colors,
            patch_led_strip_len,
            send_colors,
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

                    app_handle
                        .emit("led_colors_changed", publisher)
                        .unwrap();
                }
            });

            let app_handle = app.handle().clone();
            tokio::spawn(async move {
                loop {
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
                            return;
                        }
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
