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
use tauri::{http::ResponseBuilder, regex, Manager};
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
        .register_uri_scheme_protocol("ambient-light", move |_app, request| {
            let response = ResponseBuilder::new().header("Access-Control-Allow-Origin", "*");

            let uri = request.uri();
            let uri = percent_encoding::percent_decode_str(uri)
                .decode_utf8()
                .unwrap()
                .to_string();

            let url = url_build_parse::parse_url(uri.as_str());

            if let Err(err) = url {
                error!("url parse error: {}", err);
                return response
                    .status(500)
                    .mimetype("text/plain")
                    .body("Parse uri failed.".as_bytes().to_vec());
            }

            let url = url.unwrap();

            let re = regex::Regex::new(r"^/displays/(\d+)$").unwrap();
            let path = url.path;
            let captures = re.captures(path.as_str());

            if let None = captures {
                error!("path not matched: {:?}", path);
                return response
                    .status(404)
                    .mimetype("text/plain")
                    .body("Path Not Found.".as_bytes().to_vec());
            }

            let captures = captures.unwrap();

            let display_id = captures[1].parse::<u32>().unwrap();

            let bytes = tokio::task::block_in_place(move || {
                tauri::async_runtime::block_on(async move {
                    let screenshot_manager = ScreenshotManager::global().await;
                    let rx: Result<tokio::sync::watch::Receiver<Screenshot>, anyhow::Error> =
                        screenshot_manager.subscribe_by_display_id(display_id).await;

                    if let Err(err) = rx {
                        anyhow::bail!("Display#{}: not found. {}", display_id, err);
                    }
                    let mut rx = rx.unwrap();

                    if rx.changed().await.is_err() {
                        anyhow::bail!("Display#{}: no more screenshot.", display_id);
                    }
                    let screenshot = rx.borrow().clone();
                    let bytes = screenshot.bytes.read().await;
                    if bytes.len() == 0 {
                        anyhow::bail!("Display#{}: no screenshot.", display_id);
                    }

                    log::debug!("Display#{}: screenshot size: {}", display_id, bytes.len());

                    let (scale_factor_x, scale_factor_y, width, height) = if url.query.is_some()
                        && url.query.as_ref().unwrap().contains_key("height")
                        && url.query.as_ref().unwrap().contains_key("width")
                    {
                        let width = url.query.as_ref().unwrap()["width"]
                            .parse::<u32>()
                            .map_err(|err| {
                                warn!("width parse error: {}", err);
                                err
                            })?;
                        let height = url.query.as_ref().unwrap()["height"]
                            .parse::<u32>()
                            .map_err(|err| {
                                warn!("height parse error: {}", err);
                                err
                            })?;
                        (
                            screenshot.width as f32 / width as f32,
                            screenshot.height as f32 / height as f32,
                            width,
                            height,
                        )
                    } else {
                        log::debug!("scale by scale_factor");
                        let scale_factor = screenshot.scale_factor;
                        (
                            scale_factor,
                            scale_factor,
                            (screenshot.width as f32 / scale_factor) as u32,
                            (screenshot.height as f32 / scale_factor) as u32,
                        )
                    };
                    log::debug!(
                        "scale by query. width: {}, height: {}, scale_factor: {}, len: {}",
                        width,
                        height,
                        screenshot.width as f32 / width as f32,
                        width * height * 4,
                    );

                    let bytes_per_row = screenshot.bytes_per_row as f32;

                    let mut rgba_buffer = vec![0u8; (width * height * 4) as usize];

                    for y in 0..height {
                        for x in 0..width {
                            let offset = ((y as f32) * scale_factor_y).floor() as usize
                                * bytes_per_row as usize
                                + ((x as f32) * scale_factor_x).floor() as usize * 4;
                            let b = bytes[offset];
                            let g = bytes[offset + 1];
                            let r = bytes[offset + 2];
                            let a = bytes[offset + 3];
                            let offset_2 = (y * width + x) as usize * 4;
                            rgba_buffer[offset_2] = r;
                            rgba_buffer[offset_2 + 1] = g;
                            rgba_buffer[offset_2 + 2] = b;
                            rgba_buffer[offset_2 + 3] = a;
                        }
                    }

                    Ok(rgba_buffer.clone())
                })
            });

            if let Ok(bytes) = bytes {
                return response
                    .mimetype("octet/stream")
                    .status(200)
                    .body(bytes.to_vec());
            }
            let err = bytes.unwrap_err();
            error!("request screenshot bin data failed: {}", err);
            return response
                .mimetype("text/plain")
                .status(500)
                .body(err.to_string().into_bytes());
        })
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

                    app_handle.emit_all("config_changed", config).unwrap();
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
                        .emit_all("led_sorted_colors_changed", publisher)
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
                        .emit_all("led_colors_changed", publisher)
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

                                app_handle.emit_all("boards_changed", boards).unwrap();
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

                    app_handle.emit_all("displays_changed", displays).unwrap();
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
