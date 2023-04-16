use std::{collections::HashMap, sync::Arc};

use core_graphics::display::{
    kCGNullWindowID, kCGWindowImageDefault, kCGWindowListOptionOnScreenOnly, CGDisplay,
};
use core_graphics::geometry::{CGPoint, CGRect, CGSize};
use paris::warn;
use tauri::async_runtime::RwLock;
use tokio::sync::{broadcast, watch, OnceCell};
use tokio::time::{self, Duration};

use crate::screenshot::LedSamplePoints;
use crate::{ambient_light::SamplePointMapper, led_color::LedColor, screenshot::Screenshot};

pub fn take_screenshot(display_id: u32, scale_factor: f32) -> anyhow::Result<Screenshot> {
    log::debug!("take_screenshot");

    let cg_display = CGDisplay::new(display_id);
    let cg_image = CGDisplay::screenshot(
        cg_display.bounds(),
        kCGWindowListOptionOnScreenOnly,
        kCGNullWindowID,
        kCGWindowImageDefault,
    )
    .ok_or_else(|| anyhow::anyhow!("Display#{}: take screenshot failed", display_id))?;

    let buffer = cg_image.data();
    let bytes_per_row = cg_image.bytes_per_row();

    let height = cg_image.height();
    let width = cg_image.width();

    let bytes = buffer.bytes().to_owned();

    let cg_display = CGDisplay::new(display_id);
    let bound_scale_factor = (cg_display.bounds().size.width / width as f64) as f32;

    Ok(Screenshot::new(
        display_id,
        height as u32,
        width as u32,
        bytes_per_row,
        bytes,
        scale_factor,
        bound_scale_factor,
    ))
}

pub fn get_display_colors(
    display_id: u32,
    sample_points: &Vec<Vec<LedSamplePoints>>,
    bound_scale_factor: f32,
) -> anyhow::Result<Vec<LedColor>> {
    log::debug!("take_screenshot");
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
    pub channels: Arc<RwLock<HashMap<u32, watch::Receiver<Screenshot>>>>,
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
            let mut interval = time::interval(Duration::from_millis(1000));

            let screenshot = screenshot.unwrap();
            let (screenshot_tx, screenshot_rx) = watch::channel(screenshot);
            {
                let channels = channels.clone();
                let mut channels = channels.write().await;
                channels.insert(display_id, screenshot_rx.clone());
            }

            let merged_screenshot_tx = merged_screenshot_tx.read().await.clone();

            loop {
                Self::take_screenshot_loop(
                    display_id,
                    scale_factor,
                    &screenshot_tx,
                    &merged_screenshot_tx,
                )
                .await;
                interval.tick().await;
                tokio::time::sleep(Duration::from_millis(1)).await;
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
            match merged_screenshot_tx.send(screenshot.clone()) {
                Ok(_) => {
                    log::info!(
                        "take_screenshot_loop: merged_screenshot_tx.send success. display#{}",
                        display_id
                    );
                }
                Err(_) => {
                }
            }
            screenshot_tx.send(screenshot).unwrap();
            // log::info!("take_screenshot_loop: send success. display#{}", display_id)
        } else {
            warn!("take_screenshot_loop: {}", screenshot.err().unwrap());
        }
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
}
