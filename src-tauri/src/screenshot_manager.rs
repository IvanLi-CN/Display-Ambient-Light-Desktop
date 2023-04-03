use std::{collections::HashMap, sync::Arc};

use core_graphics::display::{
    kCGNullWindowID, kCGWindowImageDefault, kCGWindowListOptionOnScreenOnly, CGDisplay,
};
use paris::warn;
use tauri::async_runtime::RwLock;
use tokio::sync::{broadcast, watch, OnceCell};
use tokio::time::{self, Duration};

use crate::{
    ambient_light::{SamplePointConfig, SamplePointMapper},
    led_color::LedColor,
    screenshot::{ScreenSamplePoints, Screenshot},
};

pub fn take_screenshot(display_id: u32, scale_factor: f32) -> anyhow::Result<Screenshot> {
    log::debug!("take_screenshot");
    // let start_at = std::time::Instant::now();

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
    let bytes_per_row = cg_image.bytes_per_row();

    let height = cg_image.height();
    let width = cg_image.width();

    let mut bytes = vec![0u8; buffer.len() as usize];
    bytes.copy_from_slice(&buffer);

    Ok(Screenshot::new(
        display_id,
        height as u32,
        width as u32,
        bytes_per_row,
        bytes,
        scale_factor,
        ScreenSamplePoints {
            top: vec![],
            bottom: vec![],
            left: vec![],
            right: vec![],
        },
    ))
}

pub struct ScreenshotManager {
    pub channels: Arc<RwLock<HashMap<u32, watch::Receiver<Screenshot>>>>,
    merged_screenshot_rx: Arc<RwLock<broadcast::Receiver<Screenshot>>>,
    merged_screenshot_tx: Arc<RwLock<broadcast::Sender<Screenshot>>>,
}

impl ScreenshotManager {
    pub async fn global() -> &'static Self {
        static SCREENSHOT_MANAGER: OnceCell<ScreenshotManager> = OnceCell::const_new();

        SCREENSHOT_MANAGER
            .get_or_init(|| async {
                let channels = Arc::new(RwLock::new(HashMap::new()));
                let (merged_screenshot_tx, merged_screenshot_rx) = broadcast::channel(2);
                Self {
                    channels,
                    merged_screenshot_rx: Arc::new(RwLock::new(merged_screenshot_rx)),
                    merged_screenshot_tx: Arc::new(RwLock::new(merged_screenshot_tx)),
                }
            })
            .await
    }

    pub fn start(&self) -> anyhow::Result<()> {
        let displays = display_info::DisplayInfo::all()?;
        for display in displays {
            self.start_one(display.id, display.scale_factor)?;
        }
        Ok(())
    }

    fn start_one(&self, display_id: u32, scale_factor: f32) -> anyhow::Result<()> {
        let channels = self.channels.to_owned();
        let merged_screenshot_tx = self.merged_screenshot_tx.clone();
        tokio::spawn(async move {
            let screenshot = take_screenshot(display_id, scale_factor);

            if screenshot.is_err() {
                warn!("take_screenshot_loop: {}", screenshot.err().unwrap());
                return;
            }
            let mut interval = time::interval(Duration::from_millis(33));

            let screenshot = screenshot.unwrap();
            let (screenshot_tx, screenshot_rx) = watch::channel(screenshot);
            {
                let channels = channels.clone();
                let mut channels = channels.write().await;
                channels.insert(display_id, screenshot_rx.clone());
            }

            let merged_screenshot_tx = merged_screenshot_tx.read().await.clone();

            loop {
                // interval.tick().await;
                Self::take_screenshot_loop(
                    display_id,
                    scale_factor,
                    &screenshot_tx,
                    &merged_screenshot_tx,
                )
                .await;
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        });

        Ok(())
    }

    async fn take_screenshot_loop(
        display_id: u32,
        scale_factor: f32,
        screenshot_tx: &watch::Sender<Screenshot>,
        merged_screenshot_tx: &broadcast::Sender<Screenshot>,
    ) {
        let screenshot = take_screenshot(display_id, scale_factor);
        if let Ok(screenshot) = screenshot {
            screenshot_tx.send(screenshot.clone()).unwrap();
            merged_screenshot_tx.send(screenshot).unwrap();
            log::debug!("take_screenshot_loop: send success. display#{}", display_id)
        } else {
            warn!("take_screenshot_loop: {}", screenshot.err().unwrap());
        }
    }

    pub async fn get_all_colors(
        &self,
        configs: &Vec<SamplePointConfig>,
        screenshots: &Vec<&Screenshot>,
    ) -> Vec<LedColor> {
        let mut all_colors = vec![];

        for (index, screenshot) in screenshots.iter().enumerate() {
            let config = &configs[index];
            let mut colors = screenshot.get_colors_by_sample_points(&config.points).await;

            all_colors.append(&mut colors);
        }

        all_colors
    }

    pub async fn get_sorted_colors(
        colors: &Vec<LedColor>,
        mappers: &Vec<SamplePointMapper>,
    ) -> Vec<u8> {
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

            if color_index + group.start.abs_diff(group.end) > colors.len() {
                warn!(
                    "get_sorted_colors: color_index out of range. color_index: {}, strip len: {}, colors.len(): {}",
                    color_index,
                    group.start.abs_diff(group.end),
                    colors.len()
                );
                return;
            }

            if group.end > group.start {
                for i in group.start..group.end {
                    let rgb = colors[color_index].get_rgb();
                    color_index += 1;

                    global_colors[i * 3] = rgb[0];
                    global_colors[i * 3 + 1] = rgb[1];
                    global_colors[i * 3 + 2] = rgb[2];
                }
            } else {
                for i in (group.end..group.start).rev() {
                    let rgb = colors[color_index].get_rgb();
                    color_index += 1;

                    global_colors[i * 3] = rgb[0];
                    global_colors[i * 3 + 1] = rgb[1];
                    global_colors[i * 3 + 2] = rgb[2];
                }
            }
        });
        global_colors
    }

    pub async fn clone_merged_screenshot_rx(&self) -> broadcast::Receiver<Screenshot> {
        self.merged_screenshot_tx.read().await.subscribe()
    }
}
