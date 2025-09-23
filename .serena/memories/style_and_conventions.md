# Style and Conventions

## TypeScript / Solid.js
- Indent: 2 spaces. Strict TS (`tsconfig.json` sets `strict`, `noEmit`, `isolatedModules`).
- Components: `PascalCase` (e.g., `App.tsx`). Variables/functions: `camelCase`. Utility files: `kebab-case`.
- JSX: `jsx: preserve`, `jsxImportSource: solid-js`.
- Routing via `@solidjs/router`; keep routes defined in `src/index.tsx` and top-level layout in `src/App.tsx`.
- Styling: Tailwind CSS 4 utilities preferred; keep any custom styles in `src/styles.css`. Use DaisyUI components.

## Rust
- Formatting: `rustfmt` (enforced in hooks).
- Linting: `cargo clippy --all-targets --all-features` with `-D clippy::correctness -D clippy::suspicious -W clippy::complexity -W clippy::perf -W clippy::style`.
- Naming: modules `snake_case`, types `PascalCase`, constants `SCREAMING_SNAKE_CASE`.
- Tests: place under `src-tauri/**` using `#[cfg(test)]` for unit/integration tests.

## Markdown
- Follows `.markdownlint-cli2.yaml` (ATX headings, list items with `-`). Some markdown auto-fixing is available via `bun run fmt:md`.

## Commits and PRs
- Conventional Commits, English only; subject ≤ 72 chars, no trailing period; include scope when useful (e.g., `feat(led): add SK6812 gamma correction`).
- Pre-push must pass: frontend build (`bun run build`) and Rust build check (`cargo check`).
- PRs: clear description, link issue (e.g., `Closes #123`), include UI screenshots/GIFs and test impact.