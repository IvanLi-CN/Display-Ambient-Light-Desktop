# 页面导航功能文档

Ambient Light Control 应用支持通过命令行参数和 URL scheme 直接打开特定页面，这大大提升了开发和测试的便捷性。

## ✅ 功能状态

- ✅ **命令行参数导航** - 完全实现并测试通过
- ✅ **URL Scheme 协议处理** - 完全实现
- ✅ **深度链接支持** - 使用 Tauri 2.0 深度链接插件
- ✅ **前端导航服务** - 提供类型安全的导航 API
- ✅ **自动化测试** - 包含完整的测试脚本
- ⚠️ **URL Scheme 注册** - 需要安装应用后才能使用

## 功能概述

### 1. 命令行参数导航
通过 `--page` 参数直接启动应用并跳转到指定页面。

### 2. URL Scheme 导航
通过 `ambient-light://` URL scheme 从外部应用或浏览器直接打开特定页面。

### 3. 程序内导航
通过前端 NavigationService 在应用内部进行页面导航。

## 支持的页面

| 页面名称 | 描述 | 路由路径 |
|---------|------|----------|
| `info` | 基本信息页面 | `#/info` |
| `led-strips-configuration` | LED 灯带配置页面 | `#/led-strips-configuration` |
| `white-balance` | 白平衡校准页面 | `#/white-balance` |
| `led-strip-test` | LED 灯带测试页面 | `#/led-strip-test` |
| `led-data-sender-test` | LED 数据发送测试页面 | `#/led-data-sender-test` |
| `settings` | 设置页面 | `#/settings` |

## 使用方法

### 命令行参数

#### 生产环境（已构建的应用）

```bash
# 启动应用并打开信息页面
./Ambient\ Light\ Control.app/Contents/MacOS/Ambient\ Light\ Control --page info

# 启动应用并打开 LED 配置页面
./Ambient\ Light\ Control.app/Contents/MacOS/Ambient\ Light\ Control --page led-strips-configuration

# 启动应用并打开单屏灯带配置页面
./Ambient\ Light\ Control.app/Contents/MacOS/Ambient\ Light\ Control --page led-strips-configuration --display 1

# 启动应用并打开设置页面
./Ambient\ Light\ Control.app/Contents/MacOS/Ambient\ Light\ Control --page settings
```

#### 开发环境

由于 `npm run tauri dev` 无法直接传递命令行参数，在开发模式下需要使用环境变量：

```bash
# 启动开发服务器并打开信息页面
TAURI_DEV_PAGE=info npm run tauri dev

# 启动开发服务器并打开 LED 配置页面
TAURI_DEV_PAGE=led-strips-configuration npm run tauri dev

# 启动开发服务器并打开单屏灯带配置页面
TAURI_DEV_PAGE=led-strips-configuration TAURI_DEV_DISPLAY=1 npm run tauri dev

# 启动开发服务器并打开设置页面
TAURI_DEV_PAGE=settings npm run tauri dev
```

**支持的环境变量：**

- `TAURI_DEV_PAGE` - 指定要打开的页面名称
- `TAURI_DEV_DISPLAY` - 指定显示器ID（用于单屏配置页面）

### URL Scheme

```bash
# 通过 URL scheme 打开信息页面
open "ambient-light://navigate/info"

# 通过 URL scheme 打开 LED 配置页面
open "ambient-light://navigate/led-strips-configuration"

# 通过 URL scheme 打开设置页面
open "ambient-light://navigate/settings"
```

### 在前端代码中使用

```typescript
import { NavigationService, navigateToPage } from '../services/navigation-service';

// 方法 1: 使用静态方法
await NavigationService.navigateToInfo();
await NavigationService.navigateToSettings();

// 方法 2: 使用便捷函数
await navigateToPage('led-strip-test');

// 方法 3: 使用 URL scheme
import { AmbientLightUrlScheme } from '../services/navigation-service';
const url = AmbientLightUrlScheme.createNavigationUrl('white-balance');
await AmbientLightUrlScheme.openPageViaUrlScheme('settings');
```

## 开发和测试

### 自动化测试

运行测试脚本来验证所有导航功能：

```bash
# 运行导航功能测试
./scripts/test-navigation.sh
```

### 手动测试

1. **构建应用**：
   ```bash
   pnpm tauri build
   ```

2. **测试命令行参数**：
   ```bash
   # 替换为实际的应用路径
   ./src-tauri/target/release/bundle/macos/Ambient\ Light\ Control.app/Contents/MacOS/Ambient\ Light\ Control --page info
   ```

3. **测试 URL scheme**：
   ```bash
   open "ambient-light://navigate/settings"
   ```

### 开发模式测试

在开发模式下，可以通过以下方式测试：

```bash
# 启动开发服务器
pnpm tauri dev -- --page settings

# 或者在另一个终端中测试 URL scheme
open "ambient-light://navigate/info"
```

## 实现细节

### 后端实现

1. **命令行参数解析**：在 `main.rs` 中解析 `--page` 参数
2. **URL Scheme 处理**：扩展现有的 `handle_ambient_light_protocol` 函数
3. **页面导航命令**：新增 `navigate_to_page` Tauri 命令

### 前端实现

1. **NavigationService**：封装页面导航逻辑
2. **AmbientLightUrlScheme**：处理 URL scheme 生成和调用
3. **类型安全**：提供页面名称验证和类型检查

### 配置

在 `tauri.conf.json` 中注册了 URL scheme：

```json
{
  "bundle": {
    "macOS": {
      "urlSchemes": [
        {
          "name": "Ambient Light Control",
          "schemes": ["ambient-light"],
          "role": "Editor"
        }
      ]
    }
  }
}
```

## 错误处理

- 无效的页面名称会返回错误并默认跳转到信息页面
- URL scheme 调用失败会在控制台输出错误信息
- 命令行参数解析失败会被忽略，应用正常启动

## 注意事项

1. **URL Scheme 注册**：首次使用 URL scheme 时，系统可能会询问是否允许应用处理该协议
2. **权限要求**：某些系统可能需要用户授权才能使用 URL scheme
3. **开发环境**：在开发环境中，URL scheme 可能需要重新注册
4. **页面名称**：页面名称区分大小写，请使用文档中列出的确切名称

## 扩展功能

### 添加新页面支持

1. 在 `navigate_to_page` 函数中添加新的页面映射
2. 在 `NavigationService` 中添加对应的方法
3. 更新文档和测试脚本

### 支持页面参数

未来可以扩展支持页面参数，例如：
```
ambient-light://navigate/led-strips-configuration?display=1
```

这将需要修改 URL 解析逻辑和页面导航处理。
