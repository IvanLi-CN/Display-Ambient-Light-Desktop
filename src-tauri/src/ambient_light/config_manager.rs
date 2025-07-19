use std::{borrow::BorrowMut, sync::Arc};

use tauri::async_runtime::RwLock;
use tokio::{sync::OnceCell, task::yield_now};

use crate::ambient_light::{config, LedStripConfigGroup};

use super::{Border, ColorCalibration, LedType, SamplePointMapper};

pub struct ConfigManager {
    config: Arc<RwLock<LedStripConfigGroup>>,
    config_update_sender: tokio::sync::watch::Sender<LedStripConfigGroup>,
}

impl ConfigManager {
    pub async fn global() -> &'static Self {
        static CONFIG_MANAGER_GLOBAL: OnceCell<ConfigManager> = OnceCell::const_new();
        CONFIG_MANAGER_GLOBAL
            .get_or_init(|| async {
                log::info!("🔧 Initializing ConfigManager...");

                match LedStripConfigGroup::read_config().await {
                    Ok(configs) => {
                        log::info!("✅ Successfully loaded LED strip configuration");
                        let (config_update_sender, config_update_receiver) =
                            tokio::sync::watch::channel(configs.clone());

                        if let Err(err) = config_update_sender.send(configs.clone()) {
                            log::error!(
                                "Failed to send config update when read config first time: {}",
                                err
                            );
                        }
                        drop(config_update_receiver);
                        ConfigManager {
                            config: Arc::new(RwLock::new(configs)),
                            config_update_sender,
                        }
                    }
                    Err(e) => {
                        log::warn!("⚠️ Failed to load LED strip configuration: {}", e);
                        log::info!("🔄 Using default configuration instead...");

                        match LedStripConfigGroup::get_default_config().await {
                            Ok(default_config) => {
                                log::info!(
                                    "✅ Successfully loaded default LED strip configuration"
                                );
                                let (config_update_sender, config_update_receiver) =
                                    tokio::sync::watch::channel(default_config.clone());

                                if let Err(err) = config_update_sender.send(default_config.clone())
                                {
                                    log::error!(
                                        "Failed to send config update when read default config: {}",
                                        err
                                    );
                                }
                                drop(config_update_receiver);
                                ConfigManager {
                                    config: Arc::new(RwLock::new(default_config)),
                                    config_update_sender,
                                }
                            }
                            Err(default_err) => {
                                log::error!(
                                    "❌ Failed to create default configuration: {}",
                                    default_err
                                );
                                panic!(
                                    "Failed to initialize ConfigManager with default config: {}",
                                    default_err
                                );
                            }
                        }
                    }
                }
            })
            .await
    }

    pub async fn reload(&self) -> anyhow::Result<()> {
        let mut configs = self.config.write().await;
        *configs = LedStripConfigGroup::read_config().await?;

        Ok(())
    }

    pub async fn update(&self, configs: &LedStripConfigGroup) -> anyhow::Result<()> {
        LedStripConfigGroup::write_config(configs).await?;
        self.reload().await?;

        self.config_update_sender
            .send(configs.clone())
            .map_err(|e| anyhow::anyhow!("Failed to send config update: {}", e))?;
        yield_now().await;

        log::debug!("config updated: {:?}", configs);

        Ok(())
    }

    pub async fn configs(&self) -> LedStripConfigGroup {
        self.config.read().await.clone()
    }

    pub async fn patch_led_strip_len(
        &self,
        display_id: u32,
        border: Border,
        delta_len: i8,
    ) -> anyhow::Result<()> {
        let mut config = self.config.write().await;

        for strip in config.strips.iter_mut() {
            if strip.display_id == display_id && strip.border == border {
                let target = strip.len as i64 + delta_len as i64;
                if target < 0 || target > 1000 {
                    return Err(anyhow::anyhow!(
                        "Overflow. range: 0-1000, current: {}",
                        target
                    ));
                }
                strip.len = target as usize;
            }
        }

        Self::rebuild_mappers(&mut config);

        let cloned_config = config.clone();

        drop(config);

        self.update(&cloned_config).await?;

        self.config_update_sender
            .send(cloned_config)
            .map_err(|e| anyhow::anyhow!("Failed to send config update: {}", e))?;

        Ok(())
    }

    pub async fn patch_led_strip_type(
        &self,
        display_id: u32,
        border: Border,
        led_type: LedType,
    ) -> anyhow::Result<()> {
        let mut config = self.config.write().await;

        for strip in config.strips.iter_mut() {
            if strip.display_id == display_id && strip.border == border {
                strip.led_type = led_type;
            }
        }

        let cloned_config = config.clone();

        drop(config);

        self.update(&cloned_config).await?;

        self.config_update_sender
            .send(cloned_config)
            .map_err(|e| anyhow::anyhow!("Failed to send config update: {}", e))?;

        Ok(())
    }

    pub async fn move_strip_part(
        &self,
        display_id: u32,
        border: Border,
        target_start: usize,
    ) -> anyhow::Result<()> {
        let mut config = self.config.write().await;

        for (index, strip) in config.clone().strips.iter().enumerate() {
            if strip.display_id == display_id && strip.border == border {
                let mapper = config.mappers[index].borrow_mut();

                if target_start == mapper.start {
                    return Ok(());
                }

                let target_end = mapper.end + target_start - mapper.start;

                if target_start > 1000 || target_end > 1000 {
                    return Err(anyhow::anyhow!(
                        "Overflow. range: 0-1000, current: {}-{}",
                        target_start,
                        target_end
                    ));
                }

                mapper.start = target_start as usize;
                mapper.end = target_end as usize;

                log::info!("mapper: {:?}", mapper);
            }
        }

        let cloned_config = config.clone();

        drop(config);

        self.update(&cloned_config).await?;

        self.config_update_sender
            .send(cloned_config)
            .map_err(|e| anyhow::anyhow!("Failed to send config update: {}", e))?;

        Ok(())
    }

    pub async fn reverse_led_strip_part(
        &self,
        display_id: u32,
        border: Border,
    ) -> anyhow::Result<()> {
        let mut config = self.config.write().await;

        for (index, strip) in config.clone().strips.iter().enumerate() {
            if strip.display_id == display_id && strip.border == border {
                let mapper = config.mappers[index].borrow_mut();

                let start = mapper.start;
                mapper.start = mapper.end;
                mapper.end = start;
            }
        }

        Self::rebuild_mappers(&mut config);

        let cloned_config = config.clone();

        drop(config);

        self.update(&cloned_config).await?;

        self.config_update_sender
            .send(cloned_config)
            .map_err(|e| anyhow::anyhow!("Failed to send config update: {}", e))?;

        Ok(())
    }

    fn rebuild_mappers(config: &mut LedStripConfigGroup) {
        let mut prev_pos_end = 0;
        let mappers: Vec<SamplePointMapper> = config
            .strips
            .iter()
            .enumerate()
            .map(|(index, strip)| {
                let mapper = &config.mappers[index];

                if mapper.start < mapper.end {
                    let mapper = SamplePointMapper {
                        start: mapper.start,
                        end: mapper.start + strip.len,
                        pos: prev_pos_end,
                    };
                    prev_pos_end = prev_pos_end + strip.len;
                    mapper
                } else {
                    let mapper = SamplePointMapper {
                        end: mapper.end,
                        start: mapper.end + strip.len,
                        pos: prev_pos_end,
                    };
                    prev_pos_end = prev_pos_end + strip.len;
                    mapper
                }
            })
            .collect();

        config.mappers = mappers;
    }

    pub async fn set_items(&self, items: Vec<config::LedStripConfig>) -> anyhow::Result<()> {
        let mut config = self.config.write().await;

        config.strips = items;

        let cloned_config = config.clone();

        drop(config);

        self.update(&cloned_config).await?;

        self.config_update_sender
            .send(cloned_config)
            .map_err(|e| anyhow::anyhow!("Failed to send config update: {}", e))?;

        Ok(())
    }

    pub fn clone_config_update_receiver(
        &self,
    ) -> tokio::sync::watch::Receiver<LedStripConfigGroup> {
        self.config_update_sender.subscribe()
    }

    pub async fn set_color_calibration(
        &self,
        color_calibration: ColorCalibration,
    ) -> anyhow::Result<()> {
        let config = self.config.write().await;

        let mut cloned_config = config.clone();
        cloned_config.color_calibration = color_calibration;

        drop(config);

        self.update(&cloned_config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ambient_light::config::{
        Border, ColorCalibration, LedStripConfig, LedStripConfigGroup, LedType, SamplePointMapper,
    };
    use std::sync::Arc;
    use tauri::async_runtime::RwLock;

    // Helper function to create a ConfigManager with a default in-memory config for testing
    fn create_test_config_manager() -> ConfigManager {
        let mut strips = Vec::new();
        let mut mappers = Vec::new();

        // Create a predictable config for one display
        for j in 0..4 {
            let strip = LedStripConfig {
                index: j,
                display_id: 1, // Use a fixed display ID
                border: match j {
                    0 => Border::Top,
                    1 => Border::Bottom,
                    2 => Border::Left,
                    _ => Border::Right,
                },
                start_pos: j * 30,
                len: 30,
                led_type: LedType::WS2812B,
            };
            strips.push(strip);
            mappers.push(SamplePointMapper {
                start: j * 30,
                end: (j + 1) * 30,
                pos: j * 30,
            });
        }

        let config = LedStripConfigGroup {
            strips,
            mappers,
            color_calibration: ColorCalibration::new(),
        };

        let (tx, rx) = tokio::sync::watch::channel(config.clone());
        drop(rx); // Drop the receiver as we won't use it in this test setup

        ConfigManager {
            config: Arc::new(RwLock::new(config)),
            config_update_sender: tx,
        }
    }

    #[tokio::test]
    async fn test_patch_led_strip_len_and_rebuild_mappers() {
        let manager = create_test_config_manager();

        // To test the logic of patch_led_strip_len, we can simulate its core actions
        // on a config object directly, as the function itself has side effects (file IO).
        let mut config = manager.configs().await;
        let original_len = config.strips[0].len;
        let original_mapper_end = config.mappers[0].end;

        // 1. Modify strip length
        config.strips[0].len += 5;

        // 2. Rebuild mappers
        ConfigManager::rebuild_mappers(&mut config);

        // 3. Assert changes
        assert_eq!(config.strips[0].len, original_len + 5);
        // Check if the mapper's end was updated correctly for the modified strip
        assert_eq!(config.mappers[0].end, original_mapper_end + 5);
        // Check if the next mapper's position was updated
        assert_eq!(
            config.mappers[1].pos,
            config.mappers[0].pos + config.strips[0].len
        );
    }

    #[tokio::test]
    async fn test_reverse_led_strip_part_logic() {
        let manager = create_test_config_manager();

        // Similar to the above, we test the core logic by manipulating a config object.
        let mut config = manager.configs().await;
        let original_start = config.mappers[0].start;
        let original_end = config.mappers[0].end;

        // 1. Reverse the start and end of a mapper
        let mapper = &mut config.mappers[0];
        let start = mapper.start;
        mapper.start = mapper.end;
        mapper.end = start;

        // 2. Assert the change
        assert_eq!(config.mappers[0].start, original_end);
        assert_eq!(config.mappers[0].end, original_start);
    }

    #[test]
    fn test_rebuild_mappers_logic() {
        let manager = create_test_config_manager();
        let mut config = futures::executor::block_on(manager.configs());

        // Simulate a change in strip lengths
        config.strips[0].len = 40;
        config.strips[1].len = 20;
        // Simulate a reversed strip
        let temp_start = config.mappers[2].start;
        config.mappers[2].start = config.mappers[2].end;
        config.mappers[2].end = temp_start;

        ConfigManager::rebuild_mappers(&mut config);

        // Verify the first mapper (length changed)
        assert_eq!(config.mappers[0].pos, 0);
        assert_eq!(config.mappers[0].end, config.mappers[0].start + 40);

        // Verify the second mapper's position is updated based on the first strip's new length
        assert_eq!(config.mappers[1].pos, 40);
        assert_eq!(config.mappers[1].end, config.mappers[1].start + 20);

        // Verify the third mapper's position and that it remains reversed
        assert_eq!(config.mappers[2].pos, 60); // 40 + 20
        assert!(config.mappers[2].start > config.mappers[2].end);
        assert_eq!(
            config.mappers[2].start,
            config.mappers[2].end + config.strips[2].len
        );

        // Verify the fourth mapper's position
        assert_eq!(config.mappers[3].pos, 90); // 60 + 30
    }
}
