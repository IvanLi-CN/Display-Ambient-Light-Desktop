use std::env::current_dir;

use serde::{Deserialize, Serialize};

use crate::screenshot::LedSamplePoints;

const CONFIG_FILE_NAME: &str = "cc.ivanli.ambient_light/led_strip_config.toml";

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
pub enum Border {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
pub enum LedType {
    WS2812B,
    SK6812,
}

impl Default for LedType {
    fn default() -> Self {
        LedType::WS2812B
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct LedStripConfig {
    pub index: usize,
    pub border: Border,
    pub display_id: u32,
    pub start_pos: usize,
    pub len: usize,
    #[serde(default)]
    pub led_type: LedType,
}

impl LedStripConfig {
    pub fn default_for_display(display_id: u32, index: usize) -> Self {
        Self {
            index,
            display_id,
            border: Border::Top,
            start_pos: 0,
            len: 0, // Default to 0 length
            led_type: LedType::WS2812B,
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct ColorCalibration {
    r: f32,
    g: f32,
    b: f32,
    #[serde(default = "default_w_value")]
    w: f32,
}

fn default_w_value() -> f32 {
    1.0
}

impl ColorCalibration {
    pub fn new() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            w: 1.0,
        }
    }

    pub fn to_bytes(&self) -> [u8; 3] {
        [
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
        ]
    }

    pub fn to_bytes_rgbw(&self) -> [u8; 4] {
        [
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.w * 255.0) as u8,
        ]
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LedStripConfigGroup {
    pub strips: Vec<LedStripConfig>,
    pub mappers: Vec<SamplePointMapper>,
    pub color_calibration: ColorCalibration,
}

impl LedStripConfigGroup {
    pub async fn read_config() -> anyhow::Result<Self> {
        log::info!("📖 Reading LED strip configuration...");

        // config path
        let path = dirs::config_dir()
            .unwrap_or(current_dir().unwrap())
            .join(CONFIG_FILE_NAME);

        log::info!("📁 Config file path: {:?}", path);

        let exists = tokio::fs::try_exists(path.clone())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check config file exists: {}", e))?;

        if exists {
            log::info!("📄 Config file exists, reading...");
            let config = tokio::fs::read_to_string(path).await?;

            let config: LedStripConfigGroup = toml::from_str(&config)
                .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;

            // Defer display detection to avoid blocking during initialization
            log::info!("⏳ Deferring display ID assignment to avoid blocking...");

            log::info!(
                "✅ Successfully loaded config with {} strips",
                config.strips.len()
            );

            Ok(config)
        } else {
            log::info!("📄 Config file not exist, fallback to default config");
            Ok(Self::get_default_config().await?)
        }
    }

    pub async fn write_config(configs: &Self) -> anyhow::Result<()> {
        let path = dirs::config_dir()
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
        log::info!("🔧 Creating default LED strip configuration...");

        // Create a minimal default configuration without display detection
        // Display IDs will be assigned later when needed
        let mut strips = Vec::new();
        let mut mappers = Vec::new();

        // Create default configuration for 2 displays (common setup)
        for i in 0..2 {
            for j in 0..4 {
                let item = LedStripConfig {
                    index: j + i * 4,
                    display_id: 0, // Will be assigned later
                    border: match j {
                        0 => Border::Top,
                        1 => Border::Bottom,
                        2 => Border::Left,
                        3 => Border::Right,
                        _ => unreachable!(),
                    },
                    start_pos: j + i * 4 * 30,
                    len: 30,
                    led_type: LedType::WS2812B,
                };
                strips.push(item);
                mappers.push(SamplePointMapper {
                    start: (j + i * 4) * 30,
                    end: (j + i * 4 + 1) * 30,
                    pos: (j + i * 4) * 30,
                })
            }
        }
        Ok(Self {
            strips,
            mappers,
            color_calibration: ColorCalibration::new(),
        })
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
