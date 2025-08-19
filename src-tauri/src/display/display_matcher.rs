use anyhow::Result;
use std::collections::HashMap;

use super::DisplayConfigGroup;

/// 显示器匹配结果
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// 配置中的显示器内部ID
    pub config_internal_id: String,
    /// 系统检测到的显示器信息
    pub system_display: display_info::DisplayInfo,
    /// 匹配分数 (0-100)
    pub match_score: u8,
    /// 匹配类型
    pub match_type: MatchType,
}

/// 匹配类型
#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    /// 精确匹配：所有关键属性都匹配
    Exact,
    /// 部分匹配：尺寸匹配，其他属性可能不同
    Partial,
    /// 位置匹配：基于相对位置关系匹配
    Position,
    /// 新显示器：无法匹配到现有配置
    New,
}

/// 显示器匹配器
pub struct DisplayMatcher {
    /// 显示器配置组
    config_group: DisplayConfigGroup,
}

impl DisplayMatcher {
    /// 创建新的显示器匹配器
    pub fn new(config_group: DisplayConfigGroup) -> Self {
        Self { config_group }
    }

    /// 匹配系统检测到的显示器与配置中的显示器
    pub fn match_displays(
        &self,
        system_displays: &[display_info::DisplayInfo],
    ) -> Result<Vec<MatchResult>> {
        let mut results = Vec::new();
        let mut used_configs = std::collections::HashSet::new();
        let mut used_systems = std::collections::HashSet::new();

        log::info!(
            "🔍 开始匹配 {} 个系统显示器与 {} 个配置显示器",
            system_displays.len(),
            self.config_group.displays.len()
        );

        // 第一轮：精确匹配
        for (sys_idx, system_display) in system_displays.iter().enumerate() {
            for config_display in &self.config_group.displays {
                if used_configs.contains(&config_display.internal_id) {
                    continue;
                }

                if config_display.exact_match(system_display) {
                    let match_result = MatchResult {
                        config_internal_id: config_display.internal_id.clone(),
                        system_display: *system_display,
                        match_score: config_display.match_score(system_display),
                        match_type: MatchType::Exact,
                    };

                    log::info!(
                        "✅ 精确匹配: 配置 '{}' <-> 系统显示器 {} (分数: {})",
                        config_display.name,
                        system_display.id,
                        match_result.match_score
                    );

                    results.push(match_result);
                    used_configs.insert(config_display.internal_id.clone());
                    used_systems.insert(sys_idx);
                    break;
                }
            }
        }

        // 第二轮：部分匹配（尺寸匹配）
        for (sys_idx, system_display) in system_displays.iter().enumerate() {
            if used_systems.contains(&sys_idx) {
                continue;
            }

            let mut best_match: Option<(String, u8)> = None;

            for config_display in &self.config_group.displays {
                if used_configs.contains(&config_display.internal_id) {
                    continue;
                }

                if config_display.partial_match(system_display) {
                    let score = config_display.match_score(system_display);
                    if best_match.is_none() || score > best_match.as_ref().unwrap().1 {
                        best_match = Some((config_display.internal_id.clone(), score));
                    }
                }
            }

            if let Some((config_id, score)) = best_match {
                let config_display = self.config_group.find_by_internal_id(&config_id).unwrap();
                let match_result = MatchResult {
                    config_internal_id: config_id.clone(),
                    system_display: *system_display,
                    match_score: score,
                    match_type: MatchType::Partial,
                };

                log::info!(
                    "🔶 部分匹配: 配置 '{}' <-> 系统显示器 {} (分数: {})",
                    config_display.name,
                    system_display.id,
                    match_result.match_score
                );

                results.push(match_result);
                used_configs.insert(config_id);
                used_systems.insert(sys_idx);
            }
        }

        // 第三轮：位置匹配（基于相对位置关系）
        if system_displays.len() > 1 && self.config_group.displays.len() > 1 {
            self.position_based_matching(
                system_displays,
                &mut results,
                &mut used_configs,
                &mut used_systems,
            );
        }

        // 第四轮：处理新显示器
        for (sys_idx, system_display) in system_displays.iter().enumerate() {
            if used_systems.contains(&sys_idx) {
                continue;
            }

            let match_result = MatchResult {
                config_internal_id: String::new(), // 新显示器没有配置ID
                system_display: *system_display,
                match_score: 0,
                match_type: MatchType::New,
            };

            log::info!(
                "🆕 新显示器: 系统显示器 {} 需要创建新配置",
                system_display.id
            );
            results.push(match_result);
        }

        log::info!("🎯 匹配完成: {} 个匹配结果", results.len());
        Ok(results)
    }

    /// 基于位置关系的匹配
    fn position_based_matching(
        &self,
        system_displays: &[display_info::DisplayInfo],
        results: &mut Vec<MatchResult>,
        used_configs: &mut std::collections::HashSet<String>,
        used_systems: &mut std::collections::HashSet<usize>,
    ) {
        // 计算系统显示器的相对位置关系
        let system_relations = self.calculate_position_relations(system_displays);

        // 计算配置显示器的相对位置关系
        let config_relations = self.calculate_config_position_relations();

        // 尝试匹配相似的位置关系
        for (sys_idx, system_display) in system_displays.iter().enumerate() {
            if used_systems.contains(&sys_idx) {
                continue;
            }

            let mut best_match: Option<(String, u8)> = None;

            for config_display in &self.config_group.displays {
                if used_configs.contains(&config_display.internal_id) {
                    continue;
                }

                // 计算位置关系相似度
                let similarity = self.calculate_position_similarity(
                    &system_relations,
                    &config_relations,
                    sys_idx,
                    &config_display.internal_id,
                );

                if similarity > 50 {
                    // 至少50%的相似度
                    let base_score = config_display.match_score(system_display);
                    let position_bonus = (similarity as f32 * 0.3) as u8; // 位置匹配最多加30分
                    let total_score = (base_score + position_bonus).min(100);

                    if best_match.is_none() || total_score > best_match.as_ref().unwrap().1 {
                        best_match = Some((config_display.internal_id.clone(), total_score));
                    }
                }
            }

            if let Some((config_id, score)) = best_match {
                let config_display = self.config_group.find_by_internal_id(&config_id).unwrap();
                let match_result = MatchResult {
                    config_internal_id: config_id.clone(),
                    system_display: *system_display,
                    match_score: score,
                    match_type: MatchType::Position,
                };

                log::info!(
                    "📍 位置匹配: 配置 '{}' <-> 系统显示器 {} (分数: {})",
                    config_display.name,
                    system_display.id,
                    match_result.match_score
                );

                results.push(match_result);
                used_configs.insert(config_id);
                used_systems.insert(sys_idx);
            }
        }
    }

    /// 计算系统显示器的位置关系
    pub fn calculate_position_relations(
        &self,
        displays: &[display_info::DisplayInfo],
    ) -> HashMap<usize, Vec<String>> {
        let mut relations = HashMap::new();

        for (i, display) in displays.iter().enumerate() {
            let mut display_relations = Vec::new();

            for (j, other) in displays.iter().enumerate() {
                if i == j {
                    continue;
                }

                // 计算相对位置关系
                let relation = if other.x > (display.x + display.width as i32) {
                    "right_of"
                } else if (other.x + other.width as i32) < display.x {
                    "left_of"
                } else if other.y > (display.y + display.height as i32) {
                    "below"
                } else if (other.y + other.height as i32) < display.y {
                    "above"
                } else {
                    "overlapping"
                };

                display_relations.push(format!("{}:{}", j, relation));
            }

            relations.insert(i, display_relations);
        }

        relations
    }

    /// 计算配置显示器的位置关系
    fn calculate_config_position_relations(&self) -> HashMap<String, Vec<String>> {
        let mut relations = HashMap::new();

        for display in &self.config_group.displays {
            let mut display_relations = Vec::new();

            if let Some(pos) = &display.last_position {
                for other in &self.config_group.displays {
                    if display.internal_id == other.internal_id {
                        continue;
                    }

                    if let Some(other_pos) = &other.last_position {
                        // 计算相对位置关系
                        let relation = if other_pos.x > (pos.x + display.width as i32) {
                            "right_of"
                        } else if (other_pos.x + other.width as i32) < pos.x {
                            "left_of"
                        } else if other_pos.y > (pos.y + display.height as i32) {
                            "below"
                        } else if (other_pos.y + other.height as i32) < pos.y {
                            "above"
                        } else {
                            "overlapping"
                        };

                        display_relations.push(format!("{}:{}", other.internal_id, relation));
                    }
                }
            }

            relations.insert(display.internal_id.clone(), display_relations);
        }

        relations
    }

    /// 计算位置关系相似度
    fn calculate_position_similarity(
        &self,
        system_relations: &HashMap<usize, Vec<String>>,
        config_relations: &HashMap<String, Vec<String>>,
        system_idx: usize,
        config_id: &str,
    ) -> u8 {
        let empty_vec = Vec::new();
        let system_rels = system_relations.get(&system_idx).unwrap_or(&empty_vec);
        let config_rels = config_relations.get(config_id).unwrap_or(&empty_vec);

        if system_rels.is_empty() && config_rels.is_empty() {
            return 100; // 都是单显示器
        }

        if system_rels.is_empty() || config_rels.is_empty() {
            return 0; // 一个是单显示器，一个不是
        }

        // 计算关系匹配度
        let mut matches = 0;
        let total = system_rels.len().max(config_rels.len());

        for sys_rel in system_rels {
            let sys_relation = sys_rel.split(':').nth(1).unwrap_or("");
            for config_rel in config_rels {
                let config_relation = config_rel.split(':').nth(1).unwrap_or("");
                if sys_relation == config_relation {
                    matches += 1;
                    break;
                }
            }
        }

        if total == 0 {
            100
        } else {
            ((matches as f32 / total as f32) * 100.0) as u8
        }
    }

    /// 更新配置组
    pub fn update_config_group(&mut self, config_group: DisplayConfigGroup) {
        self.config_group = config_group;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::DisplayConfig;

    #[test]
    fn test_exact_matching() {
        let mut config_group = DisplayConfigGroup::new();
        let config = DisplayConfig::new("Test Display".to_string(), 1920, 1080, 1.0, true);
        config_group.add_display(config);

        let matcher = DisplayMatcher::new(config_group);

        let system_display = display_info::DisplayInfo {
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

        let results = matcher.match_displays(&[system_display]).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].match_type, MatchType::Exact);
        assert!(results[0].match_score >= 80);
    }

    #[test]
    fn test_new_display_detection() {
        let config_group = DisplayConfigGroup::new(); // 空配置
        let matcher = DisplayMatcher::new(config_group);

        let system_display = display_info::DisplayInfo {
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

        let results = matcher.match_displays(&[system_display]).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].match_type, MatchType::New);
        assert_eq!(results[0].match_score, 0);
    }
}
