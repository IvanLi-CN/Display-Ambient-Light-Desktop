---
type: "always_apply"
---

未经用户批准，不得 push 代码。
commit 时，不得使用 `--no-verify`
使用 bun dev:browser 运行开发环境（同时运行 web 前端和 rust 后端），如果你要修改后端代码，请分别运行前端和后端，避免冲突。避免使用 tauri 命令启动桌面程序。
灯带配置文件位于 `/Users/ivan/Library/Application Support/cc.ivanli.ambient_light/config_v2.toml`，白羽只能使用 cat 命令读取文件做为参考，不得直接修改这个文件。

### **启动命令**

| 命令 | 前端 | 后端 | GUI | 服务端口 |
|------|------|------|-----|----------|
| `bun run dev:browser` | ✅ Vite (24100) | ✅ Browser模式 | ❌ | HTTP:24101, WS:24102 |
| `bun run tauri dev` | ✅ Vite | ✅ Desktop模式 | ✅ | 桌面应用 |
| `bun run dev:headless` | ❌ | ✅ Headless模式 | ❌ | HTTP:24101, WS:24102 |
| `bun run tauri:browser` | ❌ | ✅ Browser模式 | ❌ | HTTP:24101, WS:24102 |
| `bun run tauri:headless` | ❌ | ✅ Headless模式 | ❌ | HTTP:24101, WS:24102 |
| `bun run dev` | ✅ Vite (24100) | ❌ | ❌ | 仅前端 |

### **后端模式说明**

- **Desktop模式**: 启动Tauri桌面应用窗口
- **Browser模式**: 后端服务器，提示在浏览器访问前端  
- **Headless模式**: 纯API服务器，无前端提示
