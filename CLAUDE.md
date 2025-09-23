# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Essential Commands

### Development

```bash
# Install dependencies
bun install

# Start desktop development (frontend + backend in Tauri)
bunx tauri dev

# Frontend-only development
bun run dev

# Backend-only development with browser frontend
bun run dev:browser

# Headless backend only
bun run dev:headless
```

### Build & Deploy

```bash
# Frontend build
bun run build

# Desktop application build
bunx tauri build

# Preview production build
bun run serve
```

### Linting & Testing

```bash
# TypeScript linting (no emit check)
bun run lint

# Rust linting
bun run lint:rust

# Rust formatting
bun run fmt:rust

# Rust tests
bun run test:rust

# Markdown formatting
bun run fmt:md
```

## Architecture Overview

This is a **Tauri 2.0 desktop application** for ambient lighting control with a **Solid.js frontend** and **Rust backend**. The application captures screen content from multiple displays and controls LED strips to create ambient lighting effects.

### Core Components

**Frontend (Solid.js + TypeScript)**

- **Router**: Uses `@solidjs/router` with hash-based routing
- **State Management**: Reactive stores in `src/stores/` using Solid.js signals
- **UI**: Tailwind CSS 4 + DaisyUI components
- **API Communication**: HTTP REST API calls to backend + WebSocket for real-time data

**Backend (Rust + Tauri)**

- **HTTP API Server**: Axum-based REST API on port 24101 with Swagger documentation
- **WebSocket Server**: Real-time screen streaming on port 24102
- **Screen Capture**: macOS-native screen capture using `screen-capture-kit`
- **LED Control**: UDP and network communication for LED strip control
- **Display Management**: Multi-monitor detection and configuration

### Key Architecture Patterns

1. **Dual API Approach**: Both Tauri IPC and HTTP REST API (HTTP preferred for new features)
2. **Real-time Communication**: WebSocket for screen streaming, Tauri events for state changes
3. **Cross-platform Display Handling**: Stable display IDs with ConfigManagerV2
4. **Modular Rust Backend**: Separate modules for ambient light, display, RPC, screenshots
5. **Configuration Management**: TOML-based config with automatic migration between versions

### Navigation & Routing

- **Main Routes**: `/info`, `/led-strips-configuration`, `/color-calibration`, `/led-strip-test`, `/settings`
- **Display-specific Routes**: `/led-strips-configuration/display/{id}`
- **Deep Link Support**: `ambient-light://` protocol for external navigation
- **Tray Menu Integration**: System tray with toggle controls and navigation

### State Management Architecture

- **Frontend**: Reactive stores using Solid.js signals in `src/stores/`
- **Backend**: Global state managers with async/await patterns
- **Configuration**: V2 config system with V1 compatibility adapter
- **Real-time Updates**: Tauri events bridge backend state to frontend

## Project Structure

```
src/                    # Solid.js frontend
├── components/         # UI components organized by feature
├── stores/            # Reactive state management
├── models/            # TypeScript data models
├── contexts/          # Solid.js contexts
└── i18n/             # Internationalization

src-tauri/             # Rust backend
├── src/
│   ├── ambient_light/ # Core lighting control logic
│   ├── display/       # Multi-monitor management
│   ├── rpc/          # Network communication
│   ├── screenshot/    # Screen capture functionality
│   └── main.rs       # Application entry point
└── tauri.conf.json   # Tauri configuration
```

## Development Guidelines

### Frontend Development

- Components use Solid.js with TypeScript
- Styling with Tailwind CSS 4 utilities
- State management through reactive stores
- API calls via `adaptiveApi` service adapter
- Hot reload supported in dev mode

### Backend Development

- Rust modules organized by functionality
- Async/await patterns with tokio runtime
- Global state managers using `OnceCell` pattern
- HTTP API preferred over Tauri commands for new features
- WebSocket for real-time streaming

### Configuration System

- **V2 Config**: New stable display ID system
- **V1 Compatibility**: Adapter maintains frontend compatibility
- **Auto-migration**: Configs automatically updated between versions
- **File Location**: `~/Library/Application Support/cc.ivanli.ambient-light.desktop/`

### Running Modes

- **Desktop Mode**: Full Tauri application with system tray
- **Browser Mode**: Backend-only with web frontend via `bun run dev:browser`
- **Headless Mode**: API-only mode via `bun run dev:headless`

## Code Style & Conventions

### Commit Messages

- **Format**: Conventional Commits in English only
- **Validation**: commitlint with Chinese character detection
- **Example**: `feat(led): add SK6812 gamma correction`

### Code Formatting

- **TypeScript**: 2-space indentation, PascalCase components, camelCase variables
- **Rust**: Follow rustfmt, snake_case modules, PascalCase types
- **Pre-commit Hooks**: Lefthook validates formatting and runs tests

### Git Workflow

- Pre-commit: Frontend/backend linting, formatting, tests
- Pre-push: Build validation for both frontend and backend
- Conventional commits enforced with English-only validation
