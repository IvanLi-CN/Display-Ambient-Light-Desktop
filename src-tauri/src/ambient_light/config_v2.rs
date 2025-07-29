use std::env::current_dir;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use dirs::config_dir;

use crate::display::DisplayConfigGroup;

use super::{Border, LedType, ColorCalibration, SamplePointMapper};

const CONFIG_FILE_NAME_V2: &str = "cc.ivanli.ambient_light/config_v2.toml";
const LEGACY_LED_CONFIG_FILE: &str = "cc.ivanli.ambient_light/led_strip_config.toml";

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
            // 尝试从旧版本配置迁移
            log::info!("🔄 未找到新版本配置，尝试从旧版本迁移...");
            Self::migrate_from_legacy().await
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

    /// 从旧版本配置迁移
    pub async fn migrate_from_legacy() -> anyhow::Result<Self> {
        use super::LedStripConfigGroup;

        let legacy_path = config_dir()
            .unwrap_or(current_dir().unwrap())
            .join(LEGACY_LED_CONFIG_FILE);

        if !legacy_path.exists() {
            log::info!("🔧 未找到旧配置文件，创建默认配置");
            return Self::get_default_config().await;
        }

        log::info!("📦 开始迁移旧版本配置...");

        // 读取旧版本配置
        let legacy_config = LedStripConfigGroup::read_config().await?;

        // 获取当前显示器信息
        let displays = display_info::DisplayInfo::all()
            .map_err(|e| anyhow::anyhow!("Failed to get displays: {}", e))?;

        // 创建新配置
        let mut new_config = Self::new();

        // 迁移显示器配置
        for display_info in &displays {
            let display_config = crate::display::DisplayConfig::from_display_info(display_info);
            new_config.display_config.add_display(display_config);
        }

        // 迁移LED灯带配置
        for old_strip in &legacy_config.strips {
            // 根据旧的display_id找到对应的显示器配置
            let display_internal_id = if old_strip.display_id == 0 {
                // 如果是0，根据index分配
                let display_index = old_strip.index / 4;
                if display_index < new_config.display_config.displays.len() {
                    new_config.display_config.displays[display_index].internal_id.clone()
                } else {
                    // 如果没有足够的显示器，创建一个默认的
                    let default_display = crate::display::DisplayConfig::new(
                        format!("显示器 {}", display_index + 1),
                        1920,
                        1080,
                        1.0,
                        false,
                    );
                    let internal_id = default_display.internal_id.clone();
                    new_config.display_config.add_display(default_display);
                    internal_id
                }
            } else {
                // 根据系统ID查找对应的显示器配置
                new_config
                    .display_config
                    .displays
                    .iter()
                    .find(|d| d.last_system_id == Some(old_strip.display_id))
                    .map(|d| d.internal_id.clone())
                    .unwrap_or_else(|| {
                        // 如果找不到，创建一个新的
                        let default_display = crate::display::DisplayConfig::new(
                            format!("显示器 {}", old_strip.display_id),
                            1920,
                            1080,
                            1.0,
                            false,
                        );
                        let internal_id = default_display.internal_id.clone();
                        new_config.display_config.add_display(default_display);
                        internal_id
                    })
            };

            let new_strip = LedStripConfigV2 {
                index: old_strip.index,
                border: old_strip.border,
                display_internal_id,
                len: old_strip.len,
                led_type: old_strip.led_type,
                reversed: old_strip.reversed,
            };

            new_config.strips.push(new_strip);
        }

        // 迁移颜色校准配置
        new_config.color_calibration = legacy_config.color_calibration;

        // 生成mappers
        new_config.generate_mappers();

        // 保存新配置
        new_config.write_config().await?;

        log::info!("✅ 配置迁移完成，已保存新版本配置");

        // 备份旧配置文件
        let backup_path = legacy_path.with_extension("toml.backup");
        if let Err(e) = tokio::fs::copy(&legacy_path, &backup_path).await {
            log::warn!("⚠️ 备份旧配置文件失败: {}", e);
        } else {
            log::info!("📦 旧配置文件已备份到: {:?}", backup_path);
        }

        Ok(new_config)
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
                    let display_config = crate::display::DisplayConfig::from_display_info(display_info);
                    config.display_config.add_display(display_config);
                }

                // 为每个显示器创建默认的4个灯带配置
                for (display_index, display) in config.display_config.displays.iter().enumerate() {
                    for border_index in 0..4 {
                        let strip = LedStripConfigV2 {
                            index: border_index + display_index * 4,
                            display_internal_id: display.internal_id.clone(),
                            border: match border_index {
                                0 => Border::Top,
                                1 => Border::Right,
                                2 => Border::Bottom,
                                3 => Border::Left,
                                _ => unreachable!(),
                            },
                            len: 30,
                            led_type: LedType::WS2812B,
                            reversed: false,
                        };
                        config.strips.push(strip);
                    }
                }
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
                let display_id = default_display.internal_id.clone();
                config.display_config.add_display(default_display);

                // 创建默认灯带配置
                for i in 0..4 {
                    let strip = LedStripConfigV2 {
                        index: i,
                        display_internal_id: display_id.clone(),
                        border: match i {
                            0 => Border::Top,
                            1 => Border::Right,
                            2 => Border::Bottom,
                            3 => Border::Left,
                            _ => unreachable!(),
                        },
                        len: 30,
                        led_type: LedType::WS2812B,
                        reversed: false,
                    };
                    config.strips.push(strip);
                }
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
