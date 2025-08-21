use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::env::current_dir;
use std::time::SystemTime;

use crate::display::DisplayConfigGroup;

use super::{Border, ColorCalibration, LedType, SamplePointMapper};

const CONFIG_FILE_NAME_V2: &str = "cc.ivanli.ambient_light/config_v2.toml";

/// 新版本的LED灯带配置，使用稳定的显示器内部ID
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LedStripConfigV2 {
    pub index: usize,
    pub border: Border,
    /// 使用显示器的内部ID而不是系统ID
    pub display_internal_id: String,
    pub len: usize,
    #[serde(default)]
    pub led_type: LedType,
    #[serde(default)]
    pub reversed: bool,
}

impl LedStripConfigV2 {
    /// 计算该灯带的起始位置（基于所有灯带的序列号和长度）
    pub fn calculate_start_pos(&self, all_strips: &[LedStripConfigV2]) -> usize {
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

    pub fn default_for_display(display_internal_id: String, index: usize) -> Self {
        Self {
            index,
            display_internal_id,
            border: Border::Top,
            len: 0, // Default to 0 length
            led_type: LedType::WS2812B,
            reversed: false,
        }
    }
}

/// 新版本的LED灯带配置组
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LedStripConfigGroupV2 {
    /// 配置文件版本
    pub version: u8,
    /// 显示器配置
    pub display_config: DisplayConfigGroup,
    /// LED灯带配置
    pub strips: Vec<LedStripConfigV2>,
    /// 运行时生成的映射器（不序列化）
    #[serde(skip)]
    pub mappers: Vec<SamplePointMapper>,
    /// 颜色校准配置
    pub color_calibration: ColorCalibration,
    /// 配置创建时间
    pub created_at: SystemTime,
    /// 最后更新时间
    pub updated_at: SystemTime,
}

impl LedStripConfigGroupV2 {
    /// 创建新的配置组
    pub fn new() -> Self {
        let now = SystemTime::now();
        Self {
            version: 2,
            display_config: DisplayConfigGroup::new(),
            strips: Vec::new(),
            mappers: Vec::new(),
            color_calibration: ColorCalibration::new(),
            created_at: now,
            updated_at: now,
        }
    }

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

    /// 读取配置文件
    pub async fn read_config() -> anyhow::Result<Self> {
        let config_path = config_dir()
            .unwrap_or(current_dir().unwrap())
            .join(CONFIG_FILE_NAME_V2);

        if config_path.exists() {
            // 读取新版本配置
            let content = tokio::fs::read_to_string(&config_path).await?;
            let mut config: Self = toml::from_str(&content)?;
            config.generate_mappers();
            log::info!("✅ 成功加载新版本LED灯带配置 (v{})", config.version);
            Ok(config)
        } else {
            // 不再进行旧版迁移，直接创建并写入默认的 v2 配置
            log::info!("🆕 未找到 v2 配置，创建默认 v2 配置（不做迁移）");
            let config = Self::get_default_config().await?;
            // 立即写入以确保文件存在
            config.write_config().await?;
            Ok(config)
        }
    }

    /// 写入配置文件
    pub async fn write_config(&self) -> anyhow::Result<()> {
        let config_path = config_dir()
            .unwrap_or(current_dir().unwrap())
            .join(CONFIG_FILE_NAME_V2);

        // 确保目录存在
        if let Some(parent) = config_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let content = toml::to_string_pretty(self)?;
        tokio::fs::write(&config_path, content).await?;

        log::info!("✅ 配置已保存到: {:?}", config_path);
        Ok(())
    }

    /// 获取默认配置
    pub async fn get_default_config() -> anyhow::Result<Self> {
        log::info!("🔧 创建默认LED灯带配置...");

        let mut config = Self::new();

        // 尝试检测显示器
        match display_info::DisplayInfo::all() {
            Ok(displays) => {
                log::info!("🖥️ 检测到 {} 个显示器", displays.len());

                // 为每个检测到的显示器创建配置
                for display_info in &displays {
                    let display_config =
                        crate::display::DisplayConfig::from_display_info(display_info);
                    config.display_config.add_display(display_config);
                }

                // 不再自动创建默认灯带配置，让用户手动添加
                log::info!("🎯 显示器检测完成，等待用户手动配置LED灯带");
            }
            Err(e) => {
                log::warn!("⚠️ 无法检测显示器: {}，创建最小默认配置", e);

                // 创建默认显示器配置
                let default_display = crate::display::DisplayConfig::new(
                    "默认显示器".to_string(),
                    1920,
                    1080,
                    1.0,
                    true,
                );
                config.display_config.add_display(default_display);

                // 不再自动创建默认灯带配置，让用户手动添加
                log::info!("🎯 默认显示器配置已创建，等待用户手动配置LED灯带");
            }
        }

        config.generate_mappers();
        Ok(config)
    }
}

impl Default for LedStripConfigGroupV2 {
    fn default() -> Self {
        Self::new()
    }
}
