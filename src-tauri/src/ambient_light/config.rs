use std::env::current_dir;

use serde::{Deserialize, Serialize};

use crate::screenshot::LedSamplePoints;

const CONFIG_FILE_NAME: &str = "cc.ivanli.ambient_light/led_strip_config.toml";

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum Border {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Default)]
pub enum LedType {
    #[default]
    WS2812B,
    SK6812,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct LedStripConfig {
    pub index: usize,
    pub border: Border,
    pub display_id: u32,
    pub len: usize,
    #[serde(default)]
    pub led_type: LedType,
    #[serde(default)]
    pub reversed: bool,
}

impl LedStripConfig {
    /// 计算该灯带的起始位置（基于所有灯带的序列号和长度）
    pub fn calculate_start_pos(&self, all_strips: &[LedStripConfig]) -> usize {
        let mut start_pos = 0;

        // 按序列号排序所有灯带
        let mut sorted_strips: Vec<_> = all_strips.iter().collect();
        sorted_strips.sort_by_key(|strip| strip.index);

        // 计算当前灯带之前的所有LED数量
        for strip in sorted_strips {
            if strip.index < self.index {
                start_pos += strip.len;
            } else {
                break;
            }
        }

        start_pos
    }

    pub fn default_for_display(display_id: u32, index: usize) -> Self {
        Self {
            index,
            display_id,
            border: Border::Top,
            len: 0, // Default to 0 length
            led_type: LedType::WS2812B,
            reversed: false,
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct ColorCalibration {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    #[serde(default = "default_w_value")]
    pub w: f32,
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

    pub fn to_bytes(self) -> [u8; 3] {
        [
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
        ]
    }

    pub fn to_bytes_rgbw(self) -> [u8; 4] {
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
    #[serde(skip)]
    pub mappers: Vec<SamplePointMapper>,
    pub color_calibration: ColorCalibration,
}

impl LedStripConfigGroup {
    /// 根据 strips 配置动态生成 mappers
    pub fn generate_mappers(&mut self) {
        // 按序列号排序灯带
        let mut sorted_strips = self.strips.clone();
        sorted_strips.sort_by_key(|strip| strip.index);

        self.mappers = sorted_strips
            .iter()
            .map(|strip| {
                let start_pos = strip.calculate_start_pos(&self.strips);
                let end_pos = start_pos + strip.len;

                if strip.reversed {
                    // 如果反向，交换 start 和 end
                    SamplePointMapper {
                        start: end_pos,
                        end: start_pos,
                        pos: start_pos,
                    }
                } else {
                    SamplePointMapper {
                        start: start_pos,
                        end: end_pos,
                        pos: start_pos,
                    }
                }
            })
            .collect();

        log::debug!("生成了 {} 个 mappers", self.mappers.len());
        for (i, mapper) in self.mappers.iter().enumerate() {
            log::debug!(
                "Mapper {}: start={}, end={}, pos={}",
                i,
                mapper.start,
                mapper.end,
                mapper.pos
            );
        }
    }
}

impl LedStripConfigGroup {
    pub async fn read_config() -> anyhow::Result<Self> {
        log::warn!("⚠️ LedStripConfigGroup::read_config() 已弃用，不再从文件读取配置");
        log::info!("🔄 返回默认配置，请使用 ConfigManagerV2 和 LedStripConfigGroupV2");

        // 直接返回默认配置，不再尝试读取旧配置文件
        Self::get_default_config().await
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
        log::info!("🔧 Creating minimal LED strip configuration...");

        // Create a minimal default configuration without any LED strips
        // Users will need to manually add LED strips through the frontend
        let strips = Vec::new();

        let mut config = Self {
            strips,
            mappers: Vec::new(), // 将被 generate_mappers 填充
            color_calibration: ColorCalibration::new(),
        };

        // 生成 mappers
        config.generate_mappers();

        Ok(config)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_led_strip_config_group_from_toml() {
        let toml_str = r#"
            [[strips]]
            index = 0
            border = "Top"
            display_id = 1
            len = 60
            led_type = "WS2812B"
            reversed = false

            [[strips]]
            index = 1
            border = "Bottom"
            display_id = 1
            len = 60
            led_type = "SK6812"
            reversed = true

            [color_calibration]
            r = 1.0
            g = 0.9
            b = 0.8
            w = 1.0
        "#;

        let mut config: LedStripConfigGroup = toml::from_str(toml_str).unwrap();

        // 生成 mappers
        config.generate_mappers();

        assert_eq!(config.strips.len(), 2);
        assert_eq!(config.mappers.len(), 2);

        assert_eq!(config.strips[0].index, 0);
        assert_eq!(config.strips[0].border, Border::Top);
        assert_eq!(config.strips[0].led_type, LedType::WS2812B);
        assert!(!config.strips[0].reversed);

        assert_eq!(config.strips[1].index, 1);
        assert_eq!(config.strips[1].border, Border::Bottom);
        assert_eq!(config.strips[1].led_type, LedType::SK6812);
        assert!(config.strips[1].reversed);

        // 验证动态生成的 mappers
        assert_eq!(config.mappers[0].start, 0);
        assert_eq!(config.mappers[0].end, 60);
        assert_eq!(config.mappers[0].pos, 0);

        // 第二个灯带是反向的，所以 start > end
        assert_eq!(config.mappers[1].start, 120); // end position
        assert_eq!(config.mappers[1].end, 60); // start position
        assert_eq!(config.mappers[1].pos, 60);

        assert_eq!(config.color_calibration.g, 0.9);
    }

    #[test]
    fn test_cross_display_serial_led_strips() {
        // 测试跨显示器串联LED灯带的配置
        let toml_str = r#"
            # 显示器1的灯带 - 序列号0和1
            [[strips]]
            index = 0
            border = "Top"
            display_id = 1
            len = 30
            led_type = "WS2812B"
            reversed = false

            [[strips]]
            index = 1
            border = "Bottom"
            display_id = 1
            len = 30
            led_type = "WS2812B"
            reversed = false

            # 显示器2的灯带 - 序列号2和3（继续串联）
            [[strips]]
            index = 2
            border = "Top"
            display_id = 2
            len = 25
            led_type = "WS2812B"
            reversed = false

            [[strips]]
            index = 3
            border = "Bottom"
            display_id = 2
            len = 25
            led_type = "WS2812B"
            reversed = true

            [color_calibration]
            r = 1.0
            g = 1.0
            b = 1.0
            w = 1.0
        "#;

        let mut config: LedStripConfigGroup = toml::from_str(toml_str).unwrap();

        // 生成 mappers
        config.generate_mappers();

        // 验证配置正确解析
        assert_eq!(config.strips.len(), 4);
        assert_eq!(config.mappers.len(), 4);

        // 验证显示器1的灯带
        let display1_strips: Vec<_> = config.strips.iter().filter(|s| s.display_id == 1).collect();
        assert_eq!(display1_strips.len(), 2);

        // 验证动态计算的 start_pos
        assert_eq!(display1_strips[0].calculate_start_pos(&config.strips), 0);
        assert_eq!(display1_strips[1].calculate_start_pos(&config.strips), 30);

        // 验证显示器2的灯带（串联在显示器1之后）
        let display2_strips: Vec<_> = config.strips.iter().filter(|s| s.display_id == 2).collect();
        assert_eq!(display2_strips.len(), 2);

        // 验证动态计算的 start_pos
        assert_eq!(display2_strips[0].calculate_start_pos(&config.strips), 60);
        assert_eq!(display2_strips[1].calculate_start_pos(&config.strips), 85);

        // 验证总LED数量
        let total_leds: usize = config.strips.iter().map(|s| s.len).sum();
        assert_eq!(total_leds, 110);

        // 验证动态生成的 mappers
        assert_eq!(config.mappers[0].start, 0); // index 0
        assert_eq!(config.mappers[0].end, 30);
        assert_eq!(config.mappers[1].start, 30); // index 1
        assert_eq!(config.mappers[1].end, 60);
        assert_eq!(config.mappers[2].start, 60); // index 2
        assert_eq!(config.mappers[2].end, 85);
        // index 3 是反向的
        assert_eq!(config.mappers[3].start, 110); // end position
        assert_eq!(config.mappers[3].end, 85); // start position
    }

    #[test]
    fn test_calculate_start_pos_method() {
        // 测试 calculate_start_pos 方法的正确性
        let strips = vec![
            LedStripConfig {
                index: 0,
                border: Border::Bottom,
                display_id: 2,
                len: 38,
                led_type: LedType::SK6812,
                reversed: false,
            },
            LedStripConfig {
                index: 1,
                border: Border::Right,
                display_id: 2,
                len: 22,
                led_type: LedType::WS2812B,
                reversed: false,
            },
            LedStripConfig {
                index: 2,
                border: Border::Top,
                display_id: 2,
                len: 38,
                led_type: LedType::WS2812B,
                reversed: false,
            },
            LedStripConfig {
                index: 3,
                border: Border::Top,
                display_id: 1,
                len: 38,
                led_type: LedType::SK6812,
                reversed: false,
            },
        ];

        // 验证每个灯带的 start_pos 计算
        assert_eq!(strips[0].calculate_start_pos(&strips), 0); // index 0: 0
        assert_eq!(strips[1].calculate_start_pos(&strips), 38); // index 1: 38
        assert_eq!(strips[2].calculate_start_pos(&strips), 60); // index 2: 60
        assert_eq!(strips[3].calculate_start_pos(&strips), 98); // index 3: 98

        // 验证总LED数量
        let total_leds: usize = strips.iter().map(|s| s.len).sum();
        assert_eq!(total_leds, 136);
    }

    #[tokio::test]
    async fn test_get_default_config() {
        let default_config = LedStripConfigGroup::get_default_config().await.unwrap();

        assert_eq!(default_config.strips.len(), 8); // 2 displays * 4 borders
        assert_eq!(default_config.mappers.len(), 8);
        assert_eq!(default_config.color_calibration.r, 1.0);
        assert_eq!(default_config.color_calibration.g, 1.0);
        assert_eq!(default_config.color_calibration.b, 1.0);
        assert_eq!(default_config.color_calibration.w, 1.0);
    }

    #[tokio::test]
    async fn test_config_serialization_deserialization() {
        let original_config = LedStripConfigGroup::get_default_config().await.unwrap();
        let toml_string = toml::to_string(&original_config).unwrap();
        let mut deserialized_config: LedStripConfigGroup = toml::from_str(&toml_string).unwrap();

        // 生成 mappers（因为 mappers 被标记为 skip，不会被序列化）
        deserialized_config.generate_mappers();

        assert_eq!(
            original_config.strips.len(),
            deserialized_config.strips.len()
        );
        assert_eq!(
            original_config.mappers.len(),
            deserialized_config.mappers.len()
        );
        for (i, strip) in original_config.strips.iter().enumerate() {
            assert_eq!(strip.index, deserialized_config.strips[i].index);
            assert_eq!(strip.border, deserialized_config.strips[i].border);
            assert_eq!(strip.len, deserialized_config.strips[i].len);
            assert_eq!(strip.reversed, deserialized_config.strips[i].reversed);
        }
    }
}
