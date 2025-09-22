# Repository Guidelines

## Project Structure & Module Organization

- `src/` — Solid.js frontend (components, stores, models, utils, i18n).
- `src-tauri/` — Rust backend (Tauri 2). App logic, HTTP/WebSocket, screen/LED.
- `public/` — Static assets bundled by Vite.
- `docs/` — Design notes and internal docs.
- `dist/` — Frontend build output.
- `scripts/` — Local helper scripts.

## Build, Test, and Development Commands

- Setup: `bun install` (Node ≥ 22, Bun ≥ 1).
- Frontend dev (Vite): `bun run dev` → <http://localhost:5173>
- Desktop dev (Tauri): `bunx tauri dev`
- Browser split dev (FE+BE): `bun run dev:browser`
- Frontend build/preview: `bun run build`, `bun run serve`
- Desktop build: `bunx tauri build`
- Lint/format: `bun run lint` (TS), `bun run lint:rust`, `bun run fmt:rust`, `bun run fmt:md`
- Tests (Rust): `bun run test:rust`
- Hooks: enable once via `bun run prepare` (Lefthook)

## Coding Style & Naming Conventions

- TypeScript: 2‑space indent; components `PascalCase` (`App.tsx`), variables/functions `camelCase`, utility files `kebab-case`.
- Tailwind CSS 4: prefer utility classes; keep styles in `styles.css` when needed.
- Rust: follow `rustfmt`; modules `snake_case`, types `PascalCase`, constants `SCREAMING_SNAKE_CASE`. Fix all Clippy errors.
- Markdown: conforms to `.markdownlint-cli2.yaml` (ATX headings, lists with `-`).

## Testing Guidelines

- Primary tests live in Rust (`src-tauri/**`). Add unit/integration tests for new logic (`#[cfg(test)]`).
- Aim to cover: color sampling, device protocols, API endpoints. Run with `bun run test:rust`.

## Commit & Pull Request Guidelines

- Conventional Commits (English only). Subject ≤ 72 chars, no trailing period, add a blank line before body.
  - Example: `feat(led): add SK6812 gamma correction`
- Pre-push must pass: `bun run build` and `cargo check` (enforced by Lefthook).
- PRs: clear description, linked issue (`Closes #123`), screenshots/GIFs for UI, list test coverage/impact. Keep changes scoped and small.

## Security & Configuration Tips

- App data path (macOS): `~/Library/Application Support/cc.ivanli.ambient-light.desktop/`.
- Do not commit secrets; prefer local `.env`/keychain. Validate inputs on all IPC/HTTP routes.
