use std::{
    borrow::Borrow,
    collections::HashMap,
    ops::Sub,
    sync::Arc,
    time::{Duration, SystemTime},
};

use base64::Config;
use ddc_hi::Display;
use paris::{error, info, warn};
use tauri::async_runtime::Mutex;
use tokio::sync::{broadcast, OwnedMutexGuard};
use tracing::warn;

use crate::{display::Brightness, models, rpc};

use super::{display_config::DisplayConfig, DisplayBrightness};
use ddc_hi::Ddc;

pub struct Manager {
    displays: Arc<Mutex<HashMap<usize, Arc<Mutex<DisplayConfig>>>>>,
}

impl Manager {
    pub fn global() -> &'static Self {
        static DISPLAY_MANAGER: once_cell::sync::OnceCell<Manager> =
            once_cell::sync::OnceCell::new();

        DISPLAY_MANAGER.get_or_init(|| Self::create())
    }

    pub fn create() -> Self {
        let instance = Self {
            displays: Arc::new(Mutex::new(HashMap::new())),
        };
        instance
    }

    pub async fn subscribe_display_brightness(&self) {
        let rpc = rpc::Manager::global().await;

        let mut rx = rpc.client().subscribe_change_display_brightness_rx();

        loop {
            if let Ok(display_brightness) = rx.recv().await {
                if let Err(err) = self.set_display_brightness(display_brightness).await {
                    error!("set_display_brightness failed. {:?}", err);
                }
            }
        }
    }

    fn read_display_config_by_ddc(index: usize) -> anyhow::Result<DisplayConfig> {
        let mut displays = Display::enumerate();
        match displays.get_mut(index) {
            Some(display) => {
                let mut config = DisplayConfig::default(index);
                match display.handle.get_vcp_feature(0x10) {
                    Ok(value) => {
                        config.max_brightness = value.maximum();
                        config.min_brightness = 0;
                        config.brightness = value.value();
                    }
                    Err(_) => {}
                };
                match display.handle.get_vcp_feature(0x12) {
                    Ok(value) => {
                        config.max_contrast = value.maximum();
                        config.min_contrast = 0;
                        config.contrast = value.value();
                    }
                    Err(_) => {}
                };
                match display.handle.get_vcp_feature(0xdc) {
                    Ok(value) => {
                        config.max_mode = value.maximum();
                        config.min_mode = 0;
                        config.mode = value.value();
                    }
                    Err(_) => {}
                };

                Ok(config)
            }
            None => anyhow::bail!("display#{} is missed.", index),
        }
    }

    async fn get_display(&self, index: usize) -> anyhow::Result<OwnedMutexGuard<DisplayConfig>> {
        let mut displays = self.displays.lock().await;
        match displays.get_mut(&index) {
            Some(config) => {
                let mut config = config.to_owned().lock_owned().await;
                if config.last_modified_at > SystemTime::now().sub(Duration::from_secs(10)) {
                    info!("cached");
                    return Ok(config);
                }
                return match Self::read_display_config_by_ddc(index) {
                    Ok(config) => {
                        let id = config.id;
                        let value = Arc::new(Mutex::new(config));
                        let valueGuard = value.clone().lock_owned().await;
                        displays.insert(id, value);
                        info!("read form ddc");
                        Ok(valueGuard)
                    }
                    Err(err) => {
                        warn!(
                            "can not read config from display by ddc, use CACHED value. {:?}",
                            err
                        );
                        config.last_modified_at = SystemTime::now();
                        Ok(config)
                    }
                };
            }
            None => {
                let config = Self::read_display_config_by_ddc(index).map_err(|err| {
                    anyhow::anyhow!(
                        "can not read config from display by ddc,use DEFAULT value. {:?}",
                        err
                    )
                })?;
                let id = config.id;
                let value = Arc::new(Mutex::new(config));
                let valueGuard = value.clone().lock_owned().await;
                displays.insert(id, value);
                Ok(valueGuard)
            }
        }
    }

    pub async fn set_display_brightness(
        &self,
        display_brightness: DisplayBrightness,
    ) -> anyhow::Result<()> {
        match Display::enumerate().get_mut(display_brightness.display_index) {
            Some(display) => {
                match self.get_display(display_brightness.display_index).await {
                    Ok(mut config) => {
                        let curr = config.brightness;
                        info!("curr_brightness: {:?}", curr);
                        let mut target = match display_brightness.brightness {
                            Brightness::Relative(v) => curr.wrapping_add_signed(v),
                            Brightness::Absolute(v) => v,
                        };
                        if target.gt(&config.max_brightness) {
                            target = config.max_brightness;
                        } else if target.lt(&config.min_brightness) {
                            target = config.min_brightness;
                        }
                        config.brightness = target;
                        display
                            .handle
                            .set_vcp_feature(0x10, target as u16)
                            .map_err(|err| anyhow::anyhow!("can not set brightness. {:?}", err))?;

                        let rpc = rpc::Manager::global().await;

                        rpc.publish_desktop_cmd(
                            format!("display{}/brightness", display_brightness.display_index)
                                .as_str(),
                            target.to_be_bytes().to_vec(),
                        )
                        .await;
                    }
                    Err(err) => {
                        info!(
                            "can not get display#{} brightness. {:?}",
                            display_brightness.display_index, err
                        );
                        if let Brightness::Absolute(v) = display_brightness.brightness {
                            display.handle.set_vcp_feature(0x10, v).map_err(|err| {
                                anyhow::anyhow!("can not set brightness. {:?}", err)
                            })?;
                        };
                    }
                };
            }
            None => {
                warn!("display#{} is not found.", display_brightness.display_index);
            }
        }
        Ok(())
    }
}
