use std::{collections::HashMap, sync::Arc};

use core_graphics::display::{
    kCGNullWindowID, kCGWindowImageDefault, kCGWindowListOptionOnScreenOnly, CGDisplay,
};
use paris::{error, info, warn};
use serde::{Deserialize, Serialize};
use tauri::{async_runtime::RwLock, Window};
use tokio::sync::{watch, OnceCell};

use crate::{
    ambient_light::{SamplePointConfig, SamplePointMapper},
    screenshot::{LedSamplePoints, ScreenSamplePoints, Screenshot, ScreenshotPayload},
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
    encode_listeners: Arc<RwLock<HashMap<u32, Vec<Window>>>>,
}

impl ScreenshotManager {
    pub async fn global() -> &'static Self {
        static SCREENSHOT_MANAGER: OnceCell<ScreenshotManager> = OnceCell::const_new();

        SCREENSHOT_MANAGER
            .get_or_init(|| async {
                let channels = Arc::new(RwLock::new(HashMap::new()));
                let encode_listeners = Arc::new(RwLock::new(HashMap::new()));
                Self {
                    channels,
                    encode_listeners,
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
        tokio::spawn(async move {
            let screenshot = take_screenshot(display_id, scale_factor);

            if screenshot.is_err() {
                warn!("take_screenshot_loop: {}", screenshot.err().unwrap());
                return;
            }

            let screenshot = screenshot.unwrap();
            let (tx, rx) = watch::channel(screenshot);
            {
                let mut channels = channels.write().await;
                channels.insert(display_id, rx);
            }
            loop {
                Self::take_screenshot_loop(display_id, scale_factor, &tx).await;
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            }
        });

        Ok(())
    }

    pub async fn subscribe_encoded_screenshot_updated(
        &self,
        window: Window,
        display_id: u32,
    ) -> anyhow::Result<()> {
        let channels = self.channels.to_owned();
        let encode_listeners = self.encode_listeners.to_owned();
        // log::info!("subscribe_encoded_screenshot_updated. {}", display_id);

        {
            let encode_listeners = encode_listeners.read().await;
            let listening_windows = encode_listeners.get(&display_id);
            if listening_windows.is_some() && listening_windows.unwrap().contains(&window) {
                log::debug!("subscribe_encoded_screenshot_updated: already listening. display#{}, window#{}", display_id, window.label());
                return Ok(());
            }
        }
        {
            encode_listeners
                .write()
                .await
                .entry(display_id)
                .or_default()
                .push(window);
        }

        tokio::spawn(async move {
            info!("subscribe_encoded_screenshot_updated: start");
            let channels = channels.read().await;
            let rx = channels.get(&display_id);
            if rx.is_none() {
                error!(
                    "subscribe_encoded_screenshot_updated: can not find display_id {}",
                    display_id
                );
                return;
            }
            let mut rx = rx.unwrap().clone();
            loop {
                if let Err(err) = rx.changed().await {
                    error!(
                        "subscribe_encoded_screenshot_updated: can not wait rx {}",
                        err
                    );
                    break;
                }
                let encode_listeners = encode_listeners.read().await;
                let windows = encode_listeners.get(&display_id);
                if windows.is_none() || windows.unwrap().is_empty() {
                    info!("subscribe_encoded_screenshot_updated: no listener, stop");
                    break;
                }
                let screenshot = rx.borrow().clone();
                // let base64_image = Self::encode_screenshot_to_base64(&screenshot).await;
                let height = screenshot.height;
                let width = screenshot.width;

                // if base64_image.is_err() {
                //     error!(
                //         "subscribe_encoded_screenshot_updated: encode_screenshot_to_base64 error {}",
                //         base64_image.err().unwrap()
                //     );
                //     continue;
                // }

                // let base64_image = base64_image.unwrap();
                for window in windows.unwrap().into_iter() {
                    // let base64_image = base64_image.clone();
                    let payload = ScreenshotPayload {
                        display_id,
                        // base64_image,
                        height,
                        width,
                    };
                    if let Err(err) = window.emit("encoded-screenshot-updated", payload) {
                        error!("subscribe_encoded_screenshot_updated: emit error {}", err)
                    } else {
                        log::debug!(
                            "subscribe_encoded_screenshot_updated: emit success. display#{}",
                            display_id
                        )
                    }
                }
            }
        });
        Ok(())
    }

    async fn unsubscribe_encoded_screenshot_updated(&self, display_id: u32) -> anyhow::Result<()> {
        let channels = self.channels.to_owned();
        let mut channels = channels.write().await;
        channels.remove(&display_id);
        Ok(())
    }

    async fn take_screenshot_loop(
        display_id: u32,
        scale_factor: f32,
        tx: &watch::Sender<Screenshot>,
    ) {
        let screenshot = take_screenshot(display_id, scale_factor);
        if let Ok(screenshot) = screenshot {
            tx.send(screenshot).unwrap();
            // log::info!("take_screenshot_loop: send success. display#{}", display_id)
        } else {
            warn!("take_screenshot_loop: {}", screenshot.err().unwrap());
        }
    }

    pub async fn get_all_colors(
        &self,
        configs: &Vec<SamplePointConfig>,
        mappers: &Vec<SamplePointMapper>,
        screenshots: &Vec<Screenshot>,
    ) -> Vec<u8> {
        let total_leds = configs
            .iter()
            .fold(0, |acc, config| acc + config.points.len());

        let mut global_colors = vec![0u8; total_leds * 3];
        let mut all_colors = vec![];

        for (index, screenshot) in screenshots.iter().enumerate() {
            let config = &configs[index];
            let mut colors = screenshot.get_colors_by_sample_points(&config.points).await;

            all_colors.append(&mut colors);
        }

        let mut color_index = 0;
        mappers.iter().for_each(|group| {
            if group.end > all_colors.len() || group.start > all_colors.len() {
                warn!(
                    "get_all_colors: group out of range. start: {}, end: {}, all_colors.len(): {}",
                    group.start,
                    group.end,
                    all_colors.len()
                );
                return;
            }
            if group.end > group.start {
                for i in group.start..group.end {
                    let rgb = all_colors[color_index].get_rgb();
                    color_index += 1;

                    global_colors[i * 3] = rgb[0];
                    global_colors[i * 3 + 1] = rgb[1];
                    global_colors[i * 3 + 2] = rgb[2];
                }
            } else {
                for i in (group.end..group.start).rev() {
                    let rgb = all_colors[color_index].get_rgb();
                    color_index += 1;

                    global_colors[i * 3] = rgb[0];
                    global_colors[i * 3 + 1] = rgb[1];
                    global_colors[i * 3 + 2] = rgb[2];
                }
            }
        });
        global_colors
    }
}
