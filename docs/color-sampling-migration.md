# 颜色采样算法迁移文档

## 概述

本文档描述了从旧的颜色采样算法迁移到新的改进算法的过程，以解决之前屏幕氛围光颜色错误的问题。

## 问题背景

之前的颜色采样算法存在以下问题：

1. **颜色准确性问题**: 导致屏幕氛围光显示的颜色与实际屏幕边缘颜色不匹配
2. **多显示器采样错误**: 程序没有正确按显示器进行颜色采样，所有LED灯带都使用同一个显示器的截图数据

## 解决方案

### 新的采样函数

实现了 `sample_edge_colors_from_image` 函数，具有以下优势：

1. **更准确的颜色采样**: 使用严格的颜色判断标准
2. **更好的边界处理**: 智能处理角落颜色混合和渐变采样
3. **时间优化**: 不限制分辨率，性能优化
4. **严格测试**: 通过严格的颜色验证测试

### 函数签名

```rust
pub fn sample_edge_colors_from_image(
    image_data: &[u8],
    width: u32,
    height: u32,
    bytes_per_row: usize,
    led_configs: &[LedStripConfig],
) -> Vec<Vec<LedColor>>
```

## 迁移内容

### 1. Screenshot 结构体更新

**新增方法:**

```rust
pub async fn get_colors_by_led_configs(
    &self,
    led_configs: &[LedStripConfig],
) -> Vec<Vec<LedColor>>
```

**保留方法:**

```rust
pub async fn get_colors_by_sample_points(
    &self,
    points: &Vec<LedSamplePoints>,
) -> Vec<LedColor>
```

### 2. Publisher 更新

**文件:** `src-tauri/src/ambient_light/publisher.rs`

**更改内容:**

- 将 `screenshot.get_colors_by_sample_points(&sample_points)` 替换为 `screenshot.get_colors_by_led_configs(&strips)`
- **重要修复**: 添加显示器过滤逻辑，确保每个fetcher只处理属于当前显示器的LED灯带
- 添加二维数组展平逻辑以保持API兼容性
- 更新参数名称以反映不再使用旧的采样点

**更改前:**

```rust
let colors = screenshot.get_colors_by_sample_points(&sample_points).await;
```

**更改后:**

```rust
// 使用新的采样函数替换旧的采样逻辑
// 只处理属于当前显示器的LED灯带配置
let current_display_strips: Vec<LedStripConfig> = strips
    .iter()
    .filter(|strip| strip.display_id == display_id)
    .cloned()
    .collect();

let colors_by_strips = screenshot.get_colors_by_led_configs(&current_display_strips).await;

// 将二维颜色数组展平为一维数组，保持与旧API的兼容性
let colors: Vec<LedColor> = colors_by_strips.into_iter().flatten().collect();
```

### 3. Tauri 命令更新

**文件:** `src-tauri/src/main.rs`

**新增命令:**

```rust
#[tauri::command]
async fn get_colors_by_led_configs(
    display_id: u32,
    led_configs: Vec<ambient_light::LedStripConfig>,
) -> Result<Vec<Vec<led_color::LedColor>>, String>
```

**保留命令:**

```rust
#[tauri::command]
async fn get_one_edge_colors(
    display_id: u32,
    sample_points: Vec<screenshot::LedSamplePoints>,
) -> Result<Vec<led_color::LedColor>, String>
```

## 向后兼容性

- 保留了所有旧的API和方法
- 新的采样算法通过新的方法提供
- 前端可以逐步迁移到新的API

## 测试验证

### 严格颜色判断测试

- **中心LED**: 容差10，必须是纯色
- **边缘LED**: 智能处理角落混合和渐变采样
- **主导色验证**: 确保边缘LED的主色分量正确

### 测试结果

```bash
cargo test color_sampling_tests -- --nocapture
# ✅ test_edge_color_sampling_from_test_wallpaper ... ok
# ✅ test_single_border_sampling ... ok
# ✅ test_new_api_compatibility ... ok
# ✅ test_multi_display_color_sampling ... ok
```

**多显示器测试验证:**

- 正确按显示器ID过滤LED灯带配置
- 每个显示器独立进行颜色采样
- 验证数据结构和LED数量正确性

## 性能影响

- **内存使用**: 新算法复用现有采样逻辑，内存使用相似
- **CPU使用**: 采样精度提高，CPU使用略有增加但在可接受范围内
- **响应时间**: 保持实时性能，无明显延迟

## 部署建议

1. **测试环境验证**: 在测试环境中验证新算法的颜色准确性
2. **逐步迁移**: 可以通过配置开关在新旧算法间切换
3. **监控**: 部署后监控颜色准确性和性能指标

## 故障排除

### 如果颜色仍然不准确

1. 检查LED灯带配置是否正确
2. 验证显示器缩放设置
3. 检查测试图片是否正确加载

### 如果性能下降

1. 检查LED配置数量是否过多
2. 验证采样频率设置
3. 监控内存使用情况

## 文件变更清单

- ✅ `src-tauri/src/screenshot.rs` - 新增采样函数和测试
- ✅ `src-tauri/src/ambient_light/publisher.rs` - 更新颜色采样逻辑
- ✅ `src-tauri/src/main.rs` - 新增Tauri命令
- ✅ `docs/color-sampling-implementation.md` - 实现文档
- ✅ `docs/color-sampling-migration.md` - 迁移文档

## 总结

新的颜色采样算法通过严格的颜色判断和智能边界处理，显著提高了屏幕氛围光的颜色准确性。迁移过程保持了向后兼容性，确保现有功能不受影响。
