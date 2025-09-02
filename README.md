# Display Ambient Light Desktop App

[![Build](https://github.com/IvanLi-CN/Display-Ambient-Light-Desktop/workflows/Build%20Desktop%20App/badge.svg)](https://github.com/IvanLi-CN/Display-Ambient-Light-Desktop/actions/workflows/build.yml)
[![CI](https://github.com/IvanLi-CN/Display-Ambient-Light-Desktop/workflows/CI/badge.svg)](https://github.com/IvanLi-CN/Display-Ambient-Light-Desktop/actions/workflows/ci.yml)
[![Release](https://github.com/IvanLi-CN/Display-Ambient-Light-Desktop/workflows/Release/badge.svg)](https://github.com/IvanLi-CN/Display-Ambient-Light-Desktop/actions/workflows/release.yml)

A desktop application built with Tauri 2.0 for ambient light control, supporting multi-monitor screen sampling and LED strip control to create immersive ambient lighting effects. ✨

## ✨ Features

- 🖥️ **Multi-Monitor Support** - Automatic detection and configuration of multiple displays
- 🎨 **Real-time Screen Sampling** - High-performance screen content capture and color analysis
- 💡 **LED Strip Control** - Configurable LED strip layout and mapping support
- ⚖️ **White Balance Calibration** - Built-in white balance adjustment tool with fullscreen mode
- 🎛️ **Intuitive Configuration Interface** - Modern UI with drag-and-drop configuration support
- 🔧 **Hardware Integration** - Display brightness control and audio device management
- 📡 **Network Communication** - UDP and WebSocket communication support

## 🛠️ Tech Stack

### Frontend

- **Framework**: [Solid.js](https://solidjs.com/) - High-performance reactive UI framework
- **Build Tool**: [Vite](https://vitejs.dev/) - Fast frontend build tool
- **Styling**: [Tailwind CSS](https://tailwindcss.com/) + [DaisyUI](https://daisyui.com/) - Modern UI component library
- **Routing**: [@solidjs/router](https://github.com/solidjs/solid-router) - Client-side routing
- **Language**: TypeScript - Type-safe JavaScript

### Backend

- **Framework**: [Tauri 2.0](https://tauri.app/) - Cross-platform desktop app framework
- **Language**: Rust - High-performance systems programming language
- **Screen Capture**: [screen-capture-kit](https://crates.io/crates/screen-capture-kit) - macOS native screen capture
- **Display Control**: [ddc-hi](https://crates.io/crates/ddc-hi) - DDC/CI display control
- **Audio**: [coreaudio-rs](https://crates.io/crates/coreaudio-rs) - macOS audio system integration
- **Networking**: tokio + tokio-tungstenite - Async network communication

## 📋 System Requirements

- **Operating System**: macOS 13.0+ (primary supported platform)
- **Memory**: 4GB+ recommended
- **Graphics**: Hardware-accelerated graphics card
- **Network**: For device discovery and communication

## 📦 Installation

### Download Pre-built Binaries

1. Go to the [Releases page](https://github.com/IvanLi-CN/Display-Ambient-Light-Desktop/releases)
2. Download the latest `.dmg` file for macOS

### macOS Installation Notes

⚠️ **Important**: This application uses ad-hoc code signing and will show a security warning on first launch.

**First-time installation:**

1. Download and open the `.dmg` file
2. Drag the app to your Applications folder
3. **Do NOT double-click to open** - this will show an error
4. Instead, **right-click** on the app and select **"Open"**
5. Click **"Open"** in the security dialog that appears
6. The app will now launch normally

**Subsequent launches:**

- You can now double-click the app normally
- No more security warnings will appear

**Alternative method** (if the above doesn't work):

```bash
# Remove quarantine attribute (run in Terminal)
xattr -d com.apple.quarantine /Applications/Ambient\ Light\ Control.app
```

## 🚀 Development Setup

### Prerequisites

1. **Install Rust**

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Install Node.js and Bun**

   ```bash
   # Install Node.js 22 LTS (recommended using nvm)
   curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
   nvm install 22
   nvm use 22

   # Install Bun
   curl -fsSL https://bun.sh/install | bash
   ```

3. **Install Tauri CLI**

   ```bash
   cargo install @tauri-apps/cli@next
   ```

### Development Setup

1. **Clone the project**

   ```bash
   git clone git@github.com:IvanLi-CN/Display-Ambient-Light-Desktop.git
   cd Display-Ambient-Light-Desktop
   ```

2. **Install dependencies**

   ```bash
   bun install
   ```

3. **Start development server**

   ```bash
   bun tauri dev
   ```

### Production Build

```bash
# Build the application
bun tauri build

# Build artifacts are located in src-tauri/target/release/bundle/
```

## 📱 Application Interface

### Main Pages

1. **System Info** (`/info`) - Display system and hardware information
2. **Display Info** (`/displays`) - Monitor status and configuration
3. **LED Strip Configuration** (`/led-strips-configuration`) - LED strip layout and mapping configuration
4. **White Balance** (`/white-balance`) - Color calibration and white balance adjustment

### Core Features

- **Real-time Screen Preview** - WebSocket streaming of screen content
- **LED Mapping Configuration** - Visual configuration of LED strip positions and quantities
- **Color Calibration** - RGB adjustment panel with fullscreen comparison mode
- **Device Management** - Automatic discovery and management of LED control devices

## 🔧 Configuration Files

Application configuration is stored in the user directory:

```text
~/Library/Application Support/cc.ivanli.ambient-light.desktop/
├── config.toml          # Main configuration file
├── led_strips.json      # LED strip configuration
└── color_calibration.json # Color calibration data
```

## 🎯 Development Guide

### Project Structure

```text
desktop/
├── src/                 # Frontend source code (Solid.js)
│   ├── components/      # UI components
│   ├── stores/         # State management
│   ├── models/         # Data models
│   └── contexts/       # React Context
├── src-tauri/          # Backend source code (Rust)
│   ├── src/
│   │   ├── ambient_light/  # Ambient light control
│   │   ├── display/        # Display management
│   │   ├── rpc/           # Network communication
│   │   └── screenshot/    # Screen capture
│   └── tauri.conf.json    # Tauri configuration
└── package.json        # Frontend dependencies
```

### Development Workflow

1. **Frontend Development**: Modify files under `src/`, supports hot reload
2. **Backend Development**: Modify files under `src-tauri/src/`, requires dev server restart
3. **Configuration Changes**: Restart required after modifying `tauri.conf.json`

### Debugging Tips

- Use browser developer tools to debug frontend
- Use `console.log` and Rust's `println!` for debugging
- Check Tauri console output for backend logs

## 🚧 Development Builds

Automated development builds are created for every commit to the `main` branch. These builds are available as pre-releases on the [GitHub Releases](../../releases) page.

### Features

- **Automatic versioning** with timestamp and commit hash
- **macOS builds** with Universal binary support (Intel and Apple Silicon)
- **Pre-release artifacts** for testing latest changes
- **Auto-cleanup** of old development builds (keeps latest 10)

### Usage

1. Visit the [Releases page](../../releases)
2. Look for releases tagged with `dev-` prefix
3. Download the macOS DMG installer
4. **Note**: Development builds may contain bugs and are not recommended for production use

For more details, see [Development Builds Documentation](docs/development-builds.md).

## 🤝 Contributing

1. Fork the project
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the GPLv3 License - see the [LICENSE](LICENSE) file for details.

## 🔗 Related Links

- [Tauri Official Documentation](https://tauri.app/)
- [Solid.js Official Documentation](https://solidjs.com/)
- [Rust Official Documentation](https://doc.rust-lang.org/)
- [Tailwind CSS Documentation](https://tailwindcss.com/docs)

## 📞 Support

If you encounter issues or have suggestions, please:

- Create an [Issue](../../issues)
- Check the [Wiki](../../wiki) for more information
- Contact the developer

---

**Note**: This application is primarily optimized for macOS platform, support for other platforms may be limited.
