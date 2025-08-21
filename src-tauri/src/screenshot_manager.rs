use std::time::Duration;
use std::{collections::HashMap, sync::Arc};

use core_graphics::display::{
    kCGNullWindowID, kCGWindowImageDefault, kCGWindowListOptionOnScreenOnly, CGDisplay,
};
use paris::warn;
use tauri::async_runtime::RwLock;
use tokio::sync::{broadcast, watch, OnceCell};
use tokio::task::yield_now;
use tokio::time::sleep;

use crate::{ambient_light::SamplePointMapper, screenshot::Screenshot};

/// 检查屏幕录制权限
fn check_screen_recording_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        use core_graphics::display::CGDisplay;

        // 尝试获取主显示器的截图来测试权限
        let main_display = CGDisplay::main();
        let bounds = main_display.bounds();

        // 如果能成功创建截图，说明有权限
        let test_screenshot = CGDisplay::screenshot(
            bounds,
            kCGWindowListOptionOnScreenOnly,
            kCGNullWindowID,
            kCGWindowImageDefault,
        );

        test_screenshot.is_some()
    }
    #[cfg(not(target_os = "macos"))]
    {
        // 非macOS系统默认认为有权限
        true
    }
}

/// 请求屏幕录制权限（引导用户到系统设置）
fn request_screen_recording_permission() {
    #[cfg(target_os = "macos")]
    {
        log::warn!("🔒 屏幕录制权限缺失！");
        log::warn!("📋 请按照以下步骤授予权限：");
        log::warn!("1. 打开 系统偏好设置 > 安全性与隐私 > 隐私");
        log::warn!("2. 在左侧列表中选择 '屏幕录制'");
        log::warn!("3. 点击锁图标并输入密码");
        log::warn!("4. 勾选 'Ambient Light Control' 应用");
        log::warn!("5. 重启应用以使权限生效");

        // 尝试打开系统偏好设置到隐私页面
        if let Err(e) = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
            .spawn()
        {
            log::warn!("无法自动打开系统偏好设置: {}", e);
        }
    }
}

pub struct ScreenshotManager {
    #[allow(clippy::type_complexity)]
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
        log::info!("🔍 Attempting to detect displays...");

        let displays = match display_info::DisplayInfo::all() {
            Ok(displays) => {
                log::info!("✅ Successfully detected {} displays", displays.len());
                displays
            }
            Err(e) => {
                log::error!("❌ Failed to detect displays: {e}");
                return Err(e);
            }
        };

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
        log::info!("🎯 ScreenshotManager internal start completed successfully");
        Ok(())
    }

    async fn start_one(&self, display_id: u32, scale_factor: f32) -> anyhow::Result<()> {
        log::info!("Starting screenshot capture for display_id: {display_id}");

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

        // Start background task for screen capture
        tokio::spawn(async move {
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
                                log::warn!("display {display_id} screenshot_tx.send failed: {err}");
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
                                log::warn!("display {display_id} screenshot_tx.send failed: {err}");
                            }
                        }
                    }
                } else {
                    // If ambient light is disabled, sleep longer to reduce CPU usage
                    sleep(Duration::from_millis(1000)).await;
                }

                // Sleep for a frame duration when enabled
                // Adjusted from 200ms (5 FPS) -> 33ms (~30 FPS) for smoother preview
                if ambient_light_enabled {
                    sleep(Duration::from_millis(33)).await;
                }
                yield_now().await;
            }
        });

        Ok(())
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

    pub fn get_sorted_colors(colors: &[u8], _mappers: &[SamplePointMapper]) -> Vec<u8> {
        // 不再使用mappers，直接返回原始颜色数据
        // mappers配置已过时，现在直接基于strips配置处理数据
        colors.to_vec()
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
