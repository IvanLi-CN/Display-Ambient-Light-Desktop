# Project Overview

- Name: Display Ambient Light Desktop (package: `ambient-light-control`)
- Purpose: Desktop app that samples screen content in real time and drives addressable LED strips (WS2812B/SK6812) to create ambient lighting; supports multi‑monitor, calibration, and device/network control.
- Primary OS: macOS 13+ (Darwin). Others may be limited.

## Tech Stack
- Frontend: Solid.js + TypeScript, Vite, Tailwind CSS 4, DaisyUI, @solidjs/router.
- Desktop wrapper: Tauri 2.
- Rust backend: async runtime (tokio), HTTP server (axum + tower/tower-http + hyper), WebSocket (tokio-tungstenite), API docs (utoipa + swagger‑ui), logging (env_logger/paris), config (toml/serde), color (color_space), time/chrono, discovery (mdns-sd), display control (ddc-hi), audio (coreaudio-rs), screen capture (screen-capture-kit), image encoding (image), UUID, Tauri plugins (shell/deep-link/opener).

## Repo Structure (top level)
- `src/` — Solid.js frontend (components, stores, models, utils, i18n, etc.).
- `src-tauri/` — Rust backend (app logic, HTTP/WebSocket, screen/LED, tests).
- `public/` — static assets for Vite.
- `docs/` — design notes & internal docs.
- `dist/` — frontend build output.
- `scripts/` — helper scripts.
- Other notable files: `package.json`, `tsconfig.json`, `vite.config.ts`, `lefthook.yml`, `.markdownlint-cli2.yaml`, `rust-toolchain.toml`.

## Notable Paths
- App data (macOS): `~/Library/Application Support/cc.ivanli.ambient-light.desktop/`
  - `config.toml`, `led_strips.json`, `color_calibration.json`.

## Entrypoints
- Frontend mount: `src/index.tsx` → `#root`.
- App shell: `src/App.tsx`.
- Rust main: `src-tauri/src/main.rs`.

## Dev Server Ports
- Vite server port is configured to `24100` in `vite.config.ts` (strict port).