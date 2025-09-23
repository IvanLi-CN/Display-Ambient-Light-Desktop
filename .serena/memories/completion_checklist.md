# Completion Checklist

Use this before finalizing a task/PR:

## Code Quality
- TypeScript passes typecheck: `bun run lint`.
- Rust passes Clippy: `bun run lint:rust` (no new warnings/errors).
- Rust code formatted: `bun run fmt:rust`.
- Markdown formatted: `bun run fmt:md` (where applicable).
- Rust tests pass (if affected): `bun run test:rust`.

## Builds
- Frontend builds: `bun run build`.
- Rust compiles: `cd src-tauri && cargo check --all-targets --all-features`.
- Manual smoke test in dev (`bunx tauri dev`) for user‑visible changes.

## Docs & UI
- Update README/docs and in‑app strings if behavior/UI changed.
- Add screenshots/GIFs for UI changes in the PR.

## Security & Config
- No secrets committed; configs remain in user data dir.
- Validate inputs on IPC/HTTP routes for new endpoints.

## Git Hygiene
- Conventional Commit message in English (≤ 72 chars subject).
- Small, focused PR; link issue (e.g., `Closes #123`).
- Hooks installed (`bun run prepare`) and pre‑push passes automatically.