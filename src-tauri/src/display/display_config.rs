use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

/// 显示器配置 - 包含稳定的内部ID和物理属性
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DisplayConfig {
    /// 程序生成的稳定ID，不会因系统重启或硬件变化而改变
    pub internal_id: String,
    /// 用户可编辑的显示器名称
    pub name: String,
    /// 显示器物理属性
    pub width: u32,
    pub height: u32,
    pub scale_factor: f32,
    pub is_primary: bool,
    /// 可选的识别信息
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    /// 最后检测到的系统信息（用于匹配）
    pub last_system_id: Option<u32>,
    pub last_position: Option<DisplayPosition>,
    pub last_detected_at: Option<SystemTime>,
}

/// 显示器位置信息
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DisplayPosition {
    pub x: i32,
    pub y: i32,
}

impl DisplayConfig {
    /// 创建新的显示器配置
    pub fn new(name: String, width: u32, height: u32, scale_factor: f32, is_primary: bool) -> Self {
        Self {
            internal_id: Self::generate_internal_id(),
            name,
            width,
            height,
            scale_factor,
            is_primary,
            manufacturer: None,
            model: None,
            last_system_id: None,
            last_position: None,
            last_detected_at: None,
        }
    }

    /// 从系统显示器信息创建配置
    pub fn from_display_info(display_info: &display_info::DisplayInfo) -> Self {
        let name = if display_info.is_primary {
            "主显示器".to_string()
        } else {
            format!("显示器 {}", display_info.id)
        };

        Self {
            internal_id: Self::generate_internal_id(),
            name,
            width: display_info.width,
            height: display_info.height,
            scale_factor: display_info.scale_factor,
            is_primary: display_info.is_primary,
            manufacturer: None,
            model: None,
            last_system_id: Some(display_info.id),
            last_position: Some(DisplayPosition {
                x: display_info.x,
                y: display_info.y,
            }),
            last_detected_at: Some(SystemTime::now()),
        }
    }

    /// 生成唯一的内部ID
    fn generate_internal_id() -> String {
        format!("display_{}", Uuid::new_v4().simple())
    }

    /// 更新最后检测信息
    pub fn update_last_detected(&mut self, display_info: &display_info::DisplayInfo) {
        self.last_system_id = Some(display_info.id);
        self.last_position = Some(DisplayPosition {
            x: display_info.x,
            y: display_info.y,
        });
        self.last_detected_at = Some(SystemTime::now());
    }

    /// 检查是否与给定的显示器信息精确匹配
    pub fn exact_match(&self, display_info: &display_info::DisplayInfo) -> bool {
        self.width == display_info.width
            && self.height == display_info.height
            && (self.scale_factor - display_info.scale_factor).abs() < 0.01
            && self.is_primary == display_info.is_primary
    }

    /// 检查是否与给定的显示器信息部分匹配（仅尺寸）
    pub fn partial_match(&self, display_info: &display_info::DisplayInfo) -> bool {
        self.width == display_info.width && self.height == display_info.height
    }

    /// 计算与给定显示器信息的匹配分数（0-100）
    pub fn match_score(&self, display_info: &display_info::DisplayInfo) -> u8 {
        let mut score = 0u8;

        // 尺寸匹配 (40分)
        if self.width == display_info.width && self.height == display_info.height {
            score += 40;
        }

        // 缩放因子匹配 (20分)
        if (self.scale_factor - display_info.scale_factor).abs() < 0.01 {
            score += 20;
        }

        // 主显示器状态匹配 (20分)
        if self.is_primary == display_info.is_primary {
            score += 20;
        }

        // 位置匹配 (20分)
        if let Some(last_pos) = &self.last_position {
            if last_pos.x == display_info.x && last_pos.y == display_info.y {
                score += 20;
            }
        }

        score
    }
}

/// 显示器配置组 - 包含所有显示器配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DisplayConfigGroup {
    /// 配置文件版本
    pub version: u8,
    /// 显示器配置列表
    pub displays: Vec<DisplayConfig>,
    /// 配置创建时间
    pub created_at: SystemTime,
    /// 最后更新时间
    pub updated_at: SystemTime,
}

impl DisplayConfigGroup {
    /// 创建新的显示器配置组
    pub fn new() -> Self {
        let now = SystemTime::now();
        Self {
            version: 1,
            displays: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// 添加显示器配置
    pub fn add_display(&mut self, display: DisplayConfig) {
        self.displays.push(display);
        self.updated_at = SystemTime::now();
    }

    /// 根据内部ID查找显示器配置
    pub fn find_by_internal_id(&self, internal_id: &str) -> Option<&DisplayConfig> {
        self.displays.iter().find(|d| d.internal_id == internal_id)
    }

    /// 根据内部ID查找显示器配置（可变引用）
    pub fn find_by_internal_id_mut(&mut self, internal_id: &str) -> Option<&mut DisplayConfig> {
        self.displays
            .iter_mut()
            .find(|d| d.internal_id == internal_id)
    }

    /// 移除显示器配置
    pub fn remove_display(&mut self, internal_id: &str) -> bool {
        let initial_len = self.displays.len();
        self.displays.retain(|d| d.internal_id != internal_id);
        let removed = self.displays.len() < initial_len;
        if removed {
            self.updated_at = SystemTime::now();
        }
        removed
    }

    /// 更新显示器配置
    pub fn update_display(&mut self, display: DisplayConfig) -> bool {
        if let Some(existing) = self.find_by_internal_id_mut(&display.internal_id) {
            *existing = display;
            self.updated_at = SystemTime::now();
            true
        } else {
            false
        }
    }
}

impl Default for DisplayConfigGroup {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_config_creation() {
        let config = DisplayConfig::new("Test Display".to_string(), 1920, 1080, 1.0, true);

        assert_eq!(config.name, "Test Display");
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert_eq!(config.scale_factor, 1.0);
        assert!(config.is_primary);
        assert!(config.internal_id.starts_with("display_"));
    }

    #[test]
    fn test_match_score() {
        let config = DisplayConfig::new("Test Display".to_string(), 1920, 1080, 1.0, true);

        // 创建模拟的显示器信息
        let display_info = display_info::DisplayInfo {
            id: 1,
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            rotation: 0.0,
            scale_factor: 1.0,
            is_primary: true,
            frequency: 60.0,
            raw_handle: unsafe { std::mem::zeroed() },
        };

        // 应该得到高分（尺寸+缩放+主显示器 = 80分）
        let score = config.match_score(&display_info);
        assert_eq!(score, 80);
    }

    #[test]
    fn test_display_config_group() {
        let mut group = DisplayConfigGroup::new();
        let config = DisplayConfig::new("Test Display".to_string(), 1920, 1080, 1.0, true);
        let internal_id = config.internal_id.clone();

        group.add_display(config);
        assert_eq!(group.displays.len(), 1);

        let found = group.find_by_internal_id(&internal_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Display");

        let removed = group.remove_display(&internal_id);
        assert!(removed);
        assert_eq!(group.displays.len(), 0);
    }
}
