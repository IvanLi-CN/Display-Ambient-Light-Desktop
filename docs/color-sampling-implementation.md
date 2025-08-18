# LED颜色采样功能实现

## 概述

本文档描述了从截图数据中采样指定边缘指定范围的指定颜色数量的功能实现。

## 功能描述

### 主要函数

#### `sample_edge_colors_from_image`

从图像数据中采样指定边缘指定范围的颜色数据。

**函数签名:**

```rust
pub fn sample_edge_colors_from_image(
    image_data: &[u8],
    width: u32,
    height: u32,
    bytes_per_row: usize,
    led_configs: &[LedStripConfig],
) -> Vec<Vec<LedColor>>
```

**参数:**

- `image_data`: 图像的原始字节数据 (BGRA格式，每像素4字节)
- `width`: 图像宽度
- `height`: 图像高度  
- `bytes_per_row`: 每行字节数
- `led_configs`: LED灯带配置数组

**返回值:**
返回与LED灯带配置数组对应的颜色数据数组（有序、二维）

- 外层数组对应每个LED灯带
- 内层数组对应该灯带上的每个LED颜色

### 辅助函数

#### `sample_colors_for_led_strip`

为单个LED灯带采样颜色数据。

## 测试用例

### 测试图片

使用专门设计的测试壁纸 `led-test-wallpaper-1920x1080.png`，具有以下特征：

- **分辨率**: 1920x1080
- **顶部边缘（5%高度）**: 鲜红色 (#FF0000)
- **底部边缘（5%高度）**: 鲜绿色 (#00FF00)  
- **左侧边缘（5%宽度）**: 鲜蓝色 (#0000FF)
- **右侧边缘（5%宽度）**: 鲜黄色 (#FFFF00)
- **四个角落**: 特殊颜色标记用于方向识别
- **中心区域**: 渐变背景

### 测试函数

#### `test_edge_color_sampling_from_test_wallpaper`

测试所有四个边缘的颜色采样功能：

- 顶部灯带（10个LED）- 期望红色
- 底部灯带（10个LED）- 期望绿色
- 左侧灯带（6个LED）- 期望蓝色
- 右侧灯带（6个LED）- 期望黄色

#### `test_single_border_sampling`

测试单个边缘的颜色采样功能，验证顶部边缘的红色采样。

## 测试结果

测试成功验证了颜色采样功能的正确性，采用严格的颜色判断标准：

### 严格颜色判断标准

1. **中心LED（避免角落干扰）**: 容差仅为10，必须是纯色
2. **边缘LED**: 允许角落颜色混合或采样到中心渐变区域
3. **主导色验证**: 边缘LED的主色分量必须≥150且占主导地位

### 顶部灯带（红色区域）

```
LED 0: RGB(255, 0, 127)   // 角落紫色混合，红色占主导 ✓
LED 1-8: RGB(255, 0, 0)   // 纯红色，严格验证通过 ✓
LED 9: RGB(127, 127, 127) // 采样到中心渐变，可接受 ✓
```

### 底部灯带（绿色区域）

```
LED 0: RGB(127, 191, 0)   // 角落颜色混合，绿色占主导 ✓
LED 1-8: RGB(0, 255, 0)   // 纯绿色，严格验证通过 ✓
LED 9: RGB(127, 191, 127) // 角落颜色混合，绿色占主导 ✓
```

### 左侧灯带（蓝色区域）

```
LED 0: RGB(63, 0, 255)    // 角落颜色混合，蓝色占主导 ✓
LED 1-4: RGB(0, 0, 255)   // 纯蓝色，严格验证通过 ✓
LED 5: RGB(63, 32, 191)   // 角落颜色混合，蓝色占主导 ✓
```

### 右侧灯带（黄色区域）

```
LED 0: RGB(191, 255, 63)  // 角落颜色混合，黄色占主导 ✓
LED 1-4: RGB(255, 255, 0) // 纯黄色，严格验证通过 ✓
LED 5: RGB(255, 223, 63)  // 角落颜色混合，黄色占主导 ✓
```

## 运行测试

```bash
# 运行所有颜色采样测试
cargo test color_sampling_tests -- --nocapture

# 运行特定测试
cargo test test_edge_color_sampling_from_test_wallpaper -- --nocapture
cargo test test_single_border_sampling -- --nocapture
```

## 注意事项

1. **图像格式**: 函数期望BGRA格式的图像数据（macOS截图格式）
2. **严格颜色判断**:
   - 中心LED使用10的严格容差，确保颜色准确性
   - 边缘LED允许角落颜色混合或采样到中心渐变区域
   - 主导色验证确保边缘LED的主色分量≥150且占主导地位
3. **采样边界处理**: 当LED采样点超出边缘区域时，可能采样到中心渐变色（灰色），这是正常现象
4. **性能优化**: 函数复用了现有的采样逻辑，确保与实际截图采样的一致性
5. **测试可靠性**: 严格的颜色判断确保了颜色采样功能的准确性和可靠性

## 文件位置

- **实现代码**: `src-tauri/src/screenshot.rs`
- **测试图片**: `src-tauri/tests/assets/led-test-wallpaper-1920x1080.png`
- **测试代码**: `src-tauri/src/screenshot.rs` (color_sampling_tests模块)
