use std::{sync::Arc, borrow::Borrow};

use tauri::async_runtime::RwLock;
use tokio::sync::OnceCell;

use crate::ambient_light::{config, LedStripConfig};

use super::Border;

pub struct ConfigManager {
    configs: Arc<RwLock<Vec<LedStripConfig>>>,
    config_update_receiver: tokio::sync::watch::Receiver<Vec<LedStripConfig>>,
    config_update_sender: tokio::sync::watch::Sender<Vec<LedStripConfig>>,
}

impl ConfigManager {
    pub async fn global() -> &'static Self {
        static CONFIG_MANAGER_GLOBAL: OnceCell<ConfigManager> = OnceCell::const_new();
        CONFIG_MANAGER_GLOBAL
            .get_or_init(|| async {
                let configs = LedStripConfig::read_config().await.unwrap();
                let (config_update_sender, config_update_receiver) =
                    tokio::sync::watch::channel(configs.clone());
                ConfigManager {
                    configs: Arc::new(RwLock::new(configs)),
                    config_update_receiver,
                    config_update_sender,
                }
            })
            .await
    }

    pub async fn reload(&self) -> anyhow::Result<()> {
        let mut configs = self.configs.write().await;
        *configs = LedStripConfig::read_config().await?;

        Ok(())
    }

    pub async fn update(&self, configs: &Vec<LedStripConfig>) -> anyhow::Result<()> {
        LedStripConfig::write_config(configs).await?;
        self.reload().await?;

        self.config_update_sender.send(configs.clone()).map_err(|e| {
            anyhow::anyhow!("Failed to send config update: {}", e)
        })?;

        log::info!("config updated: {:?}", configs);

        Ok(())
    }

    pub async fn configs(&self) -> Vec<LedStripConfig> {
        self.configs.read().await.clone()
    }

    pub async fn patch_led_strip_len(&self, display_id: u32, border: Border, delta_len: i8) -> anyhow::Result<()> {
        let mut configs = self.configs.write().await;

        for config in configs.iter_mut() {
            if config.display_id == display_id && config.border == border {
                let target = config.len as i64 + delta_len as i64;
                if target < 0 || target > 1000 {
                    return Err(anyhow::anyhow!("Overflow. range: 0-1000, current: {}", target));
                }
                config.len = target as usize;
            }
        }

        let cloned_config = configs.clone();

        drop(configs);

        self.update(&cloned_config).await?;

        self.config_update_sender
            .send(cloned_config)
            .map_err(|e| anyhow::anyhow!("Failed to send config update: {}", e))?;

        Ok(())
    }

    pub fn clone_config_update_receiver(
        &self,
    ) -> tokio::sync::watch::Receiver<Vec<LedStripConfig>> {
        self.config_update_receiver.clone()
    }
}
