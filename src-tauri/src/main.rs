// Prevents additional console window on WiOk(ndows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod screenshot;
mod screenshot_manager;

use base64::Engine;
use core_graphics::display::{
    kCGNullWindowID, kCGWindowImageDefault, kCGWindowListOptionOnScreenOnly, CGDisplay,
};
use display_info::DisplayInfo;
use paris::{error, info};
use screenshot_manager::ScreenshotManager;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use tauri::{http::ResponseBuilder, regex};

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
fn take_screenshot(display_id: u32, scale_factor: f32) -> Result<String, String> {
    let exec = || {
        println!("take_screenshot");
        let start_at = std::time::Instant::now();

        let cg_display = CGDisplay::new(display_id);
        let cg_image = CGDisplay::screenshot(
            cg_display.bounds(),
            kCGWindowListOptionOnScreenOnly,
            kCGNullWindowID,
            kCGWindowImageDefault,
        )
        .ok_or_else(|| anyhow::anyhow!("Display#{}: take screenshot failed", display_id))?;
        // println!("take screenshot took {}ms", start_at.elapsed().as_millis());

        let buffer = cg_image.data();
        let bytes_per_row = cg_image.bytes_per_row() as f32;

        let height = cg_image.height();
        let width = cg_image.width();

        let image_height = (height as f32 / scale_factor) as u32;
        let image_width = (width as f32 / scale_factor) as u32;

        // println!(
        //     "raw image: {}x{}, output image: {}x{}",
        //     width, height, image_width, image_height
        // );
        // // from bitmap vec
        let mut image_buffer = vec![0u8; (image_width * image_height * 3) as usize];

        for y in 0..image_height {
            for x in 0..image_width {
                let offset =
                    (((y as f32) * bytes_per_row + (x as f32) * 4.0) * scale_factor) as usize;
                let b = buffer[offset];
                let g = buffer[offset + 1];
                let r = buffer[offset + 2];
                let offset = (y * image_width + x) as usize;
                image_buffer[offset * 3] = r;
                image_buffer[offset * 3 + 1] = g;
                image_buffer[offset * 3 + 2] = b;
            }
        }
        println!(
            "convert to image buffer took {}ms",
            start_at.elapsed().as_millis()
        );

        // to png image
        // let mut image_png = Vec::new();
        // let mut encoder = png::Encoder::new(&mut image_png, image_width, image_height);
        // encoder.set_color(png::ColorType::Rgb);
        // encoder.set_depth(png::BitDepth::Eight);

        // let mut writer = encoder
        //     .write_header()
        //     .map_err(|e| anyhow::anyhow!("png: {}", anyhow::anyhow!(e.to_string())))?;
        // writer
        //     .write_image_data(&image_buffer)
        //     .map_err(|e| anyhow::anyhow!("png: {}", anyhow::anyhow!(e.to_string())))?;
        // writer
        //     .finish()
        //     .map_err(|e| anyhow::anyhow!("png: {}", anyhow::anyhow!(e.to_string())))?;
        // println!("encode to png took {}ms", start_at.elapsed().as_millis());
        let image_webp =
            webp::Encoder::from_rgb(&image_buffer, image_width, image_height).encode(90f32);
        // // base64 image
        let mut image_base64 = String::new();
        image_base64.push_str("data:image/webp;base64,");
        let encoded = base64::engine::general_purpose::STANDARD_NO_PAD.encode(&*image_webp);
        image_base64.push_str(encoded.as_str());

        println!("took {}ms", start_at.elapsed().as_millis());
        println!("image_base64: {}", image_base64.len());

        Ok(image_base64)
    };

    exec().map_err(|e: anyhow::Error| {
        println!("error: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn subscribe_encoded_screenshot_updated(
    window: tauri::Window,
    display_id: u32,
) -> Result<(), String> {
    let screenshot_manager = ScreenshotManager::global().await;
    screenshot_manager
        .subscribe_encoded_screenshot_updated(window, display_id)
        .await
        .map_err(|err| {
            error!("subscribe_encoded_screenshot_updated: {}", err);
            err.to_string()
        })
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let screenshot_manager = ScreenshotManager::global().await;
    screenshot_manager.start().unwrap();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            take_screenshot,
            list_display_info,
            subscribe_encoded_screenshot_updated
        ])
        .register_uri_scheme_protocol("ambient-light", move |_app, request| {
            info!("request: {:?}", request.uri());
            // prepare our response
            let response = ResponseBuilder::new().header("Access-Control-Allow-Origin", "*");
            // get the file path
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
                    let channels = screenshot_manager.channels.read().await;
                    if let Some(rx) = channels.get(&display_id) {
                        let rx = rx.clone();
                        let screenshot = rx.borrow().clone();
                        let bytes = screenshot.bytes.read().await;

                        let (scale_factor, width, height) = if url.query.is_some()
                            && url.query.as_ref().unwrap().contains_key("height")
                            && url.query.as_ref().unwrap().contains_key("width")
                        {
                            let width =
                                url.query.as_ref().unwrap()["width"].parse::<u32>().unwrap();
                            let height = url.query.as_ref().unwrap()["height"]
                                .parse::<u32>()
                                .unwrap();
                            (screenshot.width as f32 / width as f32, width, height)
                        } else {
                            info!("scale by scale_factor");
                            let scale_factor = screenshot.scale_factor;
                            (
                                scale_factor,
                                (screenshot.width as f32 / scale_factor) as u32,
                                (screenshot.height as f32 / scale_factor) as u32,
                            )
                        };
                        info!(
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
                                let offset = ((y as f32) * scale_factor) as usize * bytes_per_row as usize
                                    + ((x as f32) * scale_factor) as usize * 4;
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
                    } else {
                        anyhow::bail!("Display#{}: not found", display_id);
                    }
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
