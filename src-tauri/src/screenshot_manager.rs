use std::time::Duration;
use std::{collections::HashMap, sync::Arc};

use core_graphics::display::{
    kCGNullWindowID, kCGWindowImageDefault, kCGWindowListOptionOnScreenOnly, CGDisplay,
};
use core_graphics::geometry::{CGPoint, CGRect, CGSize};
use paris::warn;
// Unused imports removed - these are for future screen capture functionality
use tauri::async_runtime::RwLock;
use tokio::sync::{broadcast, watch, OnceCell};
use tokio::task::yield_now;
use tokio::time::sleep;

use crate::screenshot::LedSamplePoints;
use crate::{ambient_light::SamplePointMapper, led_color::LedColor, screenshot::Screenshot};

pub fn get_display_colors(
    display_id: u32,
    sample_points: &Vec<Vec<LedSamplePoints>>,
    bound_scale_factor: f32,
) -> anyhow::Result<Vec<LedColor>> {
    let cg_display = CGDisplay::new(display_id);

    let mut colors = vec![];
    for points in sample_points {
        if points.len() == 0 {
            continue;
        }
        let start_x = points[0][0].0;
        let start_y = points[0][0].1;
        let end_x = points.last().unwrap().last().unwrap().0;
        let end_y = points.last().unwrap().last().unwrap().1;

        let (start_x, end_x) = (usize::min(start_x, end_x), usize::max(start_x, end_x));
        let (start_y, end_y) = (usize::min(start_y, end_y), usize::max(start_y, end_y));

        let origin = CGPoint {
            x: start_x as f64 * bound_scale_factor as f64 + cg_display.bounds().origin.x,
            y: start_y as f64 * bound_scale_factor as f64 + cg_display.bounds().origin.y,
        };
        let size = CGSize {
            width: (end_x - start_x + 1) as f64,
            height: (end_y - start_y + 1) as f64,
        };

        // log::info!(
        //     "origin: {:?}, size: {:?}, start_x: {}, start_y: {}, bounds: {:?}",
        //     origin,
        //     size,
        //     start_x,
        //     start_y,
        //     cg_display.bounds().size
        // );

        let cg_image = CGDisplay::screenshot(
            CGRect::new(&origin, &size),
            kCGWindowListOptionOnScreenOnly,
            kCGNullWindowID,
            kCGWindowImageDefault,
        )
        .ok_or_else(|| anyhow::anyhow!("Display#{}: take screenshot failed", display_id))?;

        let bitmap = cg_image.data();

        let points = points
            .iter()
            .map(|points| {
                points
                    .iter()
                    .map(|(x, y)| (*x - start_x, *y - start_y))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let mut part_colors =
            Screenshot::get_one_edge_colors_by_cg_image(&points, bitmap, cg_image.bytes_per_row());
        colors.append(&mut part_colors);
    }

    Ok(colors)
}

pub struct ScreenshotManager {
    pub channels: Arc<RwLock<HashMap<u32, Arc<RwLock<watch::Sender<Screenshot>>>>>>,
    merged_screenshot_tx: Arc<RwLock<broadcast::Sender<Screenshot>>>,
}

impl ScreenshotManager {
    pub async fn global() -> &'static Self {
        static SCREENSHOT_MANAGER: OnceCell<ScreenshotManager> = OnceCell::const_new();

        SCREENSHOT_MANAGER
            .get_or_init(|| async {
                let channels = Arc::new(RwLock::new(HashMap::new()));
                let (merged_screenshot_tx, _) = broadcast::channel::<Screenshot>(2);
                Self {
                    channels,
                    merged_screenshot_tx: Arc::new(RwLock::new(merged_screenshot_tx)),
                }
            })
            .await
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let displays = display_info::DisplayInfo::all()?;

        log::info!(
            "ScreenshotManager starting with {} displays:",
            displays.len()
        );
        for display in &displays {
            log::info!(
                "  Display ID: {}, Scale: {}",
                display.id,
                display.scale_factor
            );
        }

        let futures = displays.iter().map(|display| async {
            self.start_one(display.id, display.scale_factor)
                .await
                .unwrap_or_else(|err| {
                    warn!("start_one failed: display_id: {}, err: {}", display.id, err);
                });
        });

        futures::future::join_all(futures).await;
        log::info!("ScreenshotManager started successfully");
        Ok(())
    }

    async fn start_one(&self, display_id: u32, scale_factor: f32) -> anyhow::Result<()> {
        log::info!("Starting screenshot capture for display_id: {}", display_id);

        let merged_screenshot_tx = self.merged_screenshot_tx.clone();

        let (tx, _) = watch::channel(Screenshot::new(
            display_id,
            0,
            0,
            0,
            Arc::new(vec![]),
            scale_factor,
            scale_factor,
        ));
        let tx = Arc::new(RwLock::new(tx));

        let mut channels = self.channels.write().await;
        channels.insert(display_id, tx.clone());

        drop(channels);

        // Implement screen capture using screen-capture-kit
        loop {
            // Check if ambient light is enabled before capturing screenshots
            let ambient_light_enabled = {
                let state_manager =
                    crate::ambient_light_state::AmbientLightStateManager::global().await;
                state_manager.is_enabled().await
            };

            if ambient_light_enabled {
                match Self::capture_display_screenshot(display_id, scale_factor).await {
                    Ok(screenshot) => {
                        let tx_for_send = tx.read().await;
                        let merged_screenshot_tx = merged_screenshot_tx.write().await;

                        if let Err(_err) = merged_screenshot_tx.send(screenshot.clone()) {
                            // log::warn!("merged_screenshot_tx.send failed: {}", err);
                        }
                        if let Err(err) = tx_for_send.send(screenshot.clone()) {
                            log::warn!("display {} screenshot_tx.send failed: {}", display_id, err);
                        }
                    }
                    Err(err) => {
                        warn!(
                            "Failed to capture screenshot for display {}: {}",
                            display_id, err
                        );
                        // Create a fallback empty screenshot to maintain the interface
                        let screenshot = Screenshot::new(
                            display_id,
                            1080,
                            1920,
                            1920 * 4, // Assuming RGBA format
                            Arc::new(vec![0u8; 1920 * 1080 * 4]),
                            scale_factor,
                            scale_factor,
                        );

                        let tx_for_send = tx.read().await;
                        let merged_screenshot_tx = merged_screenshot_tx.write().await;

                        if let Err(_err) = merged_screenshot_tx.send(screenshot.clone()) {
                            // log::warn!("merged_screenshot_tx.send failed: {}", err);
                        }
                        if let Err(err) = tx_for_send.send(screenshot.clone()) {
                            log::warn!("display {} screenshot_tx.send failed: {}", display_id, err);
                        }
                    }
                }
            } else {
                // If ambient light is disabled, sleep longer to reduce CPU usage
                sleep(Duration::from_millis(1000)).await;
            }

            // Sleep for a frame duration (5 FPS for much better CPU performance when enabled)
            if ambient_light_enabled {
                sleep(Duration::from_millis(200)).await;
            }
            yield_now().await;
        }
    }

    async fn capture_display_screenshot(
        display_id: u32,
        scale_factor: f32,
    ) -> anyhow::Result<Screenshot> {
        // For now, use the existing CGDisplay approach as a fallback
        // TODO: Implement proper screen-capture-kit integration

        let cg_display = CGDisplay::new(display_id);
        let bounds = cg_display.bounds();

        let cg_image = CGDisplay::screenshot(
            bounds,
            kCGWindowListOptionOnScreenOnly,
            kCGNullWindowID,
            kCGWindowImageDefault,
        )
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Display#{}: take screenshot failed - possibly no screen recording permission",
                display_id
            )
        })?;

        let bitmap = cg_image.data();
        let width = cg_image.width() as u32;
        let height = cg_image.height() as u32;
        let bytes_per_row = cg_image.bytes_per_row();

        // Convert CFData to Vec<u8>
        let data_ptr = bitmap.bytes().as_ptr();
        let data_len = bitmap.len() as usize;
        let screenshot_data = unsafe { std::slice::from_raw_parts(data_ptr, data_len).to_vec() };

        Ok(Screenshot::new(
            display_id,
            height,
            width,
            bytes_per_row,
            Arc::new(screenshot_data),
            scale_factor,
            scale_factor,
        ))
    }

    pub fn get_sorted_colors(colors: &Vec<u8>, mappers: &Vec<SamplePointMapper>) -> Vec<u8> {
        let total_leds = mappers
            .iter()
            .map(|mapper| usize::max(mapper.start, mapper.end))
            .max()
            .unwrap_or(0) as usize;
        let mut global_colors = vec![0u8; total_leds * 3];

        let mut color_index = 0;
        mappers.iter().for_each(|group| {
            if group.end > global_colors.len() || group.start > global_colors.len() {
                warn!(
                    "get_sorted_colors: group out of range. start: {}, end: {}, global_colors.len(): {}",
                    group.start,
                    group.end,
                    global_colors.len()
                );
                return;
            }

            if color_index + group.start.abs_diff(group.end) * 3 > colors.len(){
                warn!(
                    "get_sorted_colors: color_index out of range. color_index: {}, strip len: {}, colors.len(): {}",
                    color_index / 3,
                    group.start.abs_diff(group.end),
                    colors.len() / 3
                );
                return;
            }

            if group.end > group.start {
                for i in group.start..group.end {
                    global_colors[i * 3] = colors[color_index +0];
                    global_colors[i * 3 + 1] = colors[color_index +1];
                    global_colors[i * 3 + 2] = colors[color_index +2];
                    color_index += 3;
                }
            } else {
                for i in (group.end..group.start).rev() {
                    global_colors[i * 3] = colors[color_index +0];
                    global_colors[i * 3 + 1] = colors[color_index +1];
                    global_colors[i * 3 + 2] = colors[color_index +2];
                    color_index += 3;
                }
            }
        });
        global_colors
    }

    pub async fn clone_merged_screenshot_rx(&self) -> broadcast::Receiver<Screenshot> {
        self.merged_screenshot_tx.read().await.subscribe()
    }

    pub async fn subscribe_by_display_id(
        &self,
        display_id: u32,
    ) -> anyhow::Result<watch::Receiver<Screenshot>> {
        let channels = self.channels.read().await;
        if let Some(tx) = channels.get(&display_id) {
            Ok(tx.read().await.subscribe())
        } else {
            Err(anyhow::anyhow!("display_id: {} not found", display_id))
        }
    }
}
