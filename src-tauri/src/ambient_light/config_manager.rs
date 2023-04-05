use std::{borrow::BorrowMut, sync::Arc};

use tauri::async_runtime::RwLock;
use tokio::sync::OnceCell;

use crate::ambient_light::{config, LedStripConfigGroup};

use super::{Border, SamplePointMapper};

pub struct ConfigManager {
    config: Arc<RwLock<LedStripConfigGroup>>,
    config_update_receiver: tokio::sync::watch::Receiver<LedStripConfigGroup>,
    config_update_sender: tokio::sync::watch::Sender<LedStripConfigGroup>,
}

impl ConfigManager {
    pub async fn global() -> &'static Self {
        static CONFIG_MANAGER_GLOBAL: OnceCell<ConfigManager> = OnceCell::const_new();
        CONFIG_MANAGER_GLOBAL
            .get_or_init(|| async {
                let configs = LedStripConfigGroup::read_config().await.unwrap();
                let (config_update_sender, config_update_receiver) =
                    tokio::sync::watch::channel(configs.clone());

                    config_update_sender.send(configs.clone()).unwrap();
                ConfigManager {
                    config: Arc::new(RwLock::new(configs)),
                    config_update_receiver,
                    config_update_sender,
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

        // log::info!("config updated: {:?}", configs);

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

    pub async fn move_strip_part(
        &self,
        display_id: u32,
        border: Border,
        target_start: usize,
    ) -> anyhow::Result<()> {
        let mut config = self.config.write().await;

        for (index, strip) in config.clone().strips.iter().enumerate() {
            if strip.display_id == display_id && strip.border == border {
                let mut mapper = config.mappers[index].borrow_mut();

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
                let mut mapper = config.mappers[index].borrow_mut();

                let start = mapper.start;
                mapper.start = mapper.end;
                mapper.end = start;
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
        self.config_update_receiver.clone()
    }
}
