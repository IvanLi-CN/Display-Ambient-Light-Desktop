use std::env::current_dir;

use display_info::DisplayInfo;
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
        let displays = DisplayInfo::all()?;

        // config path
        let path = config_dir()
            .unwrap_or(current_dir().unwrap())
            .join(CONFIG_FILE_NAME);

        let exists = tokio::fs::try_exists(path.clone())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check config file exists: {}", e))?;

        if exists {
            let config = tokio::fs::read_to_string(path).await?;

            let mut config: LedStripConfigGroup = toml::from_str(&config)
                .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;

            for strip in config.strips.iter_mut() {
                strip.display_id = displays[strip.index / 4].id;
            }

            // log::info!("config loaded: {:?}", config);

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

        tokio::fs::write(&path, config_text).await.map_err(|e| {
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
                    pos: (j + i * 4) * 30,
                })
            }
        }
        Ok(Self { strips, mappers })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplePointMapper {
    pub start: usize,
    pub end: usize,
    pub pos: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplePointConfig {
    pub display_id: u32,
    pub points: Vec<LedSamplePoints>,
}
