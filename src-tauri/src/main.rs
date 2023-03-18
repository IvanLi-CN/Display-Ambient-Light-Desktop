// Prevents additional console window on WiOk(ndows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod screenshot;
mod screenshot_manager;

use base64::Engine;
use core_graphics::display::{
    kCGNullWindowID, kCGWindowImageDefault, kCGWindowListOptionOnScreenOnly, CGDisplay,
};
use display_info::DisplayInfo;
use paris::error;
use screenshot_manager::ScreenshotManager;
use serde::{Deserialize, Serialize};
use serde_json::to_string;

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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
