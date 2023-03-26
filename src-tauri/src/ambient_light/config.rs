use std::env::current_dir;

use paris::{error, info};
use serde::{Deserialize, Serialize};
use tauri::api::path::config_dir;

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum Border {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct LedStripConfigOfBorders {
    pub top: Option<LedStripConfig>,
    pub bottom: Option<LedStripConfig>,
    pub left: Option<LedStripConfig>,
    pub right: Option<LedStripConfig>,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct LedStripConfig {
    pub index: usize,
    pub border: Border,
    pub display_id: u32,
    pub start_pos: usize,
    pub len: usize,
}

impl LedStripConfig {
    pub async fn read_config() -> anyhow::Result<Vec<Self>> {
        // config path
        let path = config_dir()
            .unwrap_or(current_dir().unwrap())
            .join("led_strip_config.toml");

        let exists = tokio::fs::try_exists(path.clone())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check config file exists: {}", e))?;

        if exists {
            let config = tokio::fs::read_to_string(path).await?;

            let config: Vec<Self> = toml::from_str(&config)
                .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;

            Ok(config)
        } else {
            info!("config file not exist, fallback to default config");
            Ok(Self::get_default_config().await?)
        }
    }

    pub async fn write_config(configs: &Vec<Self>) -> anyhow::Result<()> {
        let path = config_dir()
            .unwrap_or(current_dir().unwrap())
            .join("led_strip_config.toml");

        let config = toml::to_string(configs)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;

        tokio::fs::write(path, config)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to write config file: {}", e))?;

        Ok(())
    }

    pub async fn get_default_config() -> anyhow::Result<Vec<Self>> {
        let displays = display_info::DisplayInfo::all().map_err(|e| {
            error!("can not list display info: {}", e);
            anyhow::anyhow!("can not list display info: {}", e)
        })?;

        let mut configs = Vec::new();
        for (i, display) in displays.iter().enumerate() {
            for j in 0..4 {
                let config = Self {
                    index: j + i * 4 * 30,
                    display_id: display.id,
                    border: match j {
                        0 => Border::Top,
                        1 => Border::Bottom,
                        2 => Border::Left,
                        3 => Border::Right,
                        _ => unreachable!(),
                    },
                    start_pos: 0,
                    len: 30,
                };
                configs.push(config);
            }
        }

        Ok(configs)
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct DisplayConfig {
    pub id: u32,
    pub index_of_display: usize,
    pub display_width: usize,
    pub display_height: usize,
    pub led_strip_of_borders: LedStripConfigOfBorders,
    pub scale_factor: f32,
}

impl LedStripConfigOfBorders {
    pub fn default() -> Self {
        Self {
            top: None,
            bottom: None,
            left: None,
            right: None,
        }
    }
}

impl DisplayConfig {
    pub fn default(
        id: u32,
        index_of_display: usize,
        display_width: usize,
        display_height: usize,
        scale_factor: f32,
    ) -> Self {
        Self {
            id,
            index_of_display,
            display_width,
            display_height,
            led_strip_of_borders: LedStripConfigOfBorders::default(),
            scale_factor,
        }
    }
}
