# yoube

A Tauri 2 + React 19 desktop client for YouTube. A monorepo managed by a
Cargo workspace and a pnpm workspace.

## Structure

- `apps/desktop/` — Tauri + React frontend
- `crates/` — Rust backend crates (`yoube-core`, `yoube-yt-dlp`, `yoube-filter`,
  `yoube-dns`, `yoube-storage`)
- [`justfile`](justfile) — task runner recipes: `just dev`, `just build`,
  `just test`, `just fmt`, `just lint`

## Tooling

- Rust 1.85+ (edition 2024)
- Node >= 20.10, pnpm 9.12.0
- [just](https://github.com/casey/just)
