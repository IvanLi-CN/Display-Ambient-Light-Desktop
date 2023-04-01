use std::env::current_dir;

use paris::{error, info};
use serde::{Deserialize, Serialize};
use tauri::api::path::config_dir;

use crate::screenshot::{self, LedSamplePoints};

const CONFIG_FILE_NAME: &str = "cc.ivanli.ambient_light/led_strip_config.toml";

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LedStripConfigGroup {
    pub strips: Vec<LedStripConfig>,
    pub mappers: Vec<SamplePointMapper>,
}

impl LedStripConfigGroup {
    pub async fn read_config() -> anyhow::Result<Self> {
        // config path
        let path = config_dir()
            .unwrap_or(current_dir().unwrap())
            .join(CONFIG_FILE_NAME);

        let exists = tokio::fs::try_exists(path.clone())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check config file exists: {}", e))?;

        if exists {
            let config = tokio::fs::read_to_string(path).await?;

            let config: LedStripConfigGroup = toml::from_str(&config)
                .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;

            Ok(config)
        } else {
            info!("config file not exist, fallback to default config");
            Ok(Self::get_default_config().await?)
        }
    }

    pub async fn write_config(configs: &Self) -> anyhow::Result<()> {
        let path = config_dir()
            .unwrap_or(current_dir().unwrap())
            .join(CONFIG_FILE_NAME);

        tokio::fs::create_dir_all(path.parent().unwrap()).await?;

        let config_text = toml::to_string(&configs).map_err(|e| {
            anyhow::anyhow!("Failed to parse config file: {}. configs: {:?}", e, configs)
        })?;

        tokio::fs::write  (&path, config_text).await.map_err(|e| {
            anyhow::anyhow!("Failed to write config file: {}. path: {:?}", e, &path)
        })?;

        Ok(())
    }

    pub async fn get_default_config() -> anyhow::Result<Self> {
        let displays = display_info::DisplayInfo::all().map_err(|e| {
            error!("can not list display info: {}", e);
            anyhow::anyhow!("can not list display info: {}", e)
        })?;

        let mut strips = Vec::new();
        let mut mappers = Vec::new();
        for (i, display) in displays.iter().enumerate() {
            let mut configs = Vec::new();
            for j in 0..4 {
                let item = LedStripConfig {
                    index: j + i * 4,
                    display_id: display.id,
                    border: match j {
                        0 => Border::Top,
                        1 => Border::Bottom,
                        2 => Border::Left,
                        3 => Border::Right,
                        _ => unreachable!(),
                    },
                    start_pos: j + i * 4 * 30,
                    len: 30,
                };
                configs.push(item);
                strips.push(item);
                mappers.push(SamplePointMapper {
                    start: (j + i * 4) * 30,
                    end: (j + i * 4 + 1) * 30,
                })
            }
        }
        Ok(Self { strips, mappers })
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct LedStripConfigOfDisplays {
    pub id: u32,
    pub index_of_display: usize,
    pub led_strip_of_borders: LedStripConfigOfBorders,
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

impl LedStripConfigOfDisplays {
    pub fn default(id: u32, index_of_display: usize) -> Self {
        Self {
            id,
            index_of_display,
            led_strip_of_borders: LedStripConfigOfBorders::default(),
        }
    }

    pub async fn read_from_disk() -> anyhow::Result<Self> {
        let path = config_dir()
            .unwrap_or(current_dir().unwrap())
            .join("led_strip_config_of_displays.toml");

        let exists = tokio::fs::try_exists(path.clone())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check config file exists: {}", e))?;

        if exists {
            let config = tokio::fs::read_to_string(path).await?;

            let config: Self = toml::from_str(&config)
                .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;

            Ok(config)
        } else {
            info!("config file not exist, fallback to default config");
            Ok(Self::get_default_config().await?)
        }
    }

    pub async fn write_to_disk(&self) -> anyhow::Result<()> {
        let path = config_dir()
            .unwrap_or(current_dir().unwrap())
            .join("led_strip_config_of_displays.toml");

        let config = toml::to_string(self).map_err(|e| {
            anyhow::anyhow!("Failed to parse config file: {}. config: {:?}", e, self)
        })?;

        tokio::fs::write(&path, config).await.map_err(|e| {
            anyhow::anyhow!("Failed to write config file: {}. path: {:?}", e, &path)
        })?;

        Ok(())
    }

    pub async fn get_default_config() -> anyhow::Result<Self> {
        let displays = display_info::DisplayInfo::all().map_err(|e| {
            error!("can not list display info: {}", e);
            anyhow::anyhow!("can not list display info: {}", e)
        })?;

        let mut configs = Vec::new();
        for (i, display) in displays.iter().enumerate() {
            let config = Self {
                id: display.id,
                index_of_display: i,
                led_strip_of_borders: LedStripConfigOfBorders {
                    top: Some(LedStripConfig {
                        index: i * 4 * 30,
                        display_id: display.id,
                        border: Border::Top,
                        start_pos: i * 4 * 30,
                        len: 30,
                    }),
                    bottom: Some(LedStripConfig {
                        index: i * 4 * 30 + 30,
                        display_id: display.id,
                        border: Border::Bottom,
                        start_pos: i * 4 * 30 + 30,
                        len: 30,
                    }),
                    left: Some(LedStripConfig {
                        index: i * 4 * 30 + 60,
                        display_id: display.id,
                        border: Border::Left,
                        start_pos: i * 4 * 30 + 60,
                        len: 30,
                    }),
                    right: Some(LedStripConfig {
                        index: i * 4 * 30 + 90,
                        display_id: display.id,
                        border: Border::Right,
                        start_pos: i * 4 * 30 + 90,
                        len: 30,
                    }),
                },
            };
            configs.push(config);
        }

        Ok(configs[0])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplePointMapper {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplePointConfig {
    pub display_id: u32,
    pub points: Vec<LedSamplePoints>,
}
