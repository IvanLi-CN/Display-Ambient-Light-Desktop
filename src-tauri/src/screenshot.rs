use std::sync::Arc;

use serde::Serialize;
use tauri::async_runtime::RwLock;

#[derive(Debug, Clone)]
pub struct Screenshot {
    pub display_id: u32,
    pub height: u32,
    pub width: u32,
    pub bytes_per_row: usize,
    pub bytes: Arc<RwLock<Vec<u8>>>,
    pub scale_factor: f32,
}

impl Screenshot {
    pub fn new(
        display_id: u32,
        height: u32,
        width: u32,
        bytes_per_row: usize,
        bytes: Vec<u8>,
        scale_factor: f32,
    ) -> Self {
        Self {
            display_id,
            height,
            width,
            bytes_per_row,
            bytes: Arc::new(RwLock::new(bytes)),
            scale_factor,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ScreenshotPayload {
    pub display_id: u32,
    pub height: u32,
    pub width: u32,
    // pub base64_image: String,
}
