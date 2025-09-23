# Suggested Commands

## Setup
- Install deps: `bun install` (requires Node ≥ 22 and Bun ≥ 1).
- Install git hooks (once): `bun run prepare` (Lefthook).

## Development
- Frontend only (Vite @ port 24100): `bun run dev`
- Desktop app (Tauri dev): `bunx tauri dev`
- Split mode (browser FE + Rust BE headless): `bun run dev:browser`
- Backend headless (no Tauri shell): `bun run dev:headless`
- Backend in browser mode: `bun run tauri:browser`
- Env logging for Rust backend: `RUST_LOG=info bun run dev:headless`

## Build & Preview
- Frontend build: `bun run build`
- Frontend preview: `bun run serve`
- Desktop release build: `bunx tauri build`

## Lint, Format, Test
- TypeScript typecheck: `bun run lint`
- Rust lint (Clippy): `bun run lint:rust`
- Rust fmt: `bun run fmt:rust`
- Markdown lint/fix: `bun run fmt:md` (lint task is currently skip in hooks)
- Rust tests: `bun run test:rust`

## Pre-push Baseline (must pass)
- Frontend build: `bun run build`
- Rust compile check: `cd src-tauri && cargo check --all-targets --all-features`

## Darwin Utilities (handy)
- Open file/URL/app: `open <path|url>`
- Clipboard: `pbcopy` / `pbpaste`
- Show app bundle contents: `open -R /Applications/Ambient\ Light\ Control.app`

## Notes
- Vite server port is fixed to `24100` (`vite.config.ts`) with `strictPort: true`.
- Tauri CLI is also available via `npx tauri` or `cargo install @tauri-apps/cli` (but prefer `bunx tauri`).