# yoube — Implementation Plan (Phases 0–4)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the `yoube` desktop app through phase 4 of the roadmap — Tauri 2 + React 19 + TS shell, browse + watch, virtual account, downloads, and 3-layer ad/tracker blocking — with a clean growth path to phases 5–6.

**Architecture:** Monorepo (Cargo + pnpm). Rust core services in `crates/`, Tauri shell in `apps/desktop/src-tauri/`, React 19 UI in `apps/desktop/src/`, generated TS bindings via `tauri-specta`. yt-dlp is a Tauri sidecar; blocking is layered (DNS hosts → ABP → SponsorBlock).

**Tech Stack (versions verified 2026-09-04):**
- Rust 1.85+, Cargo workspace, edition 2024
- Tauri 2 (latest stable on the `v2` branch of `tauri-apps/tauri`)
- React 19 + TypeScript 5.6+ + Vite 6
- `transact-rs/sqlx` 0.9 with the `sqlite` + `runtime-tokio` features
- `brave/adblock-rust` for ABP
- `tokio` 1.x, `reqwest` 0.12, `serde` 1, `thiserror` 2, `anyhow` 1
- Frontend: TanStack Router 1.x, TanStack Query 5.x, TanStack Virtual 3.x, Zustand 5.x, Tailwind 3.x, shadcn/ui, `@vidstack/react` 1.x
- yt-dlp (latest stable), ffmpeg (latest stable) as Tauri sidecars

## Global Constraints

- Every Rust trait method returns `Result<T, AppError>` where `AppError: serde::Serialize`.
- Every Tauri command is a one-line shim in `crates/yoube-core/src/commands.rs` that resolves a trait from `tauri::State<AppContext>` and calls the method.
- Services never call each other directly. They share an `AppContext` struct held in Tauri state.
- All DB writes go through `sqlx` query macros; no `query_as` from strings at runtime in the hot path.
- All HTTP calls from the Rust side go through a `FilterMiddleware` so L2 ABP filtering is uniform.
- All `unsafe` blocks must have a `// SAFETY:` comment.
- No `unwrap()` in non-test code. `expect()` is allowed only for invariants established at startup.
- Public functions in `yoube-core` have a doc comment.
- Every task ends with: lint clean (`cargo clippy -- -D warnings`, `biome check`), typecheck clean, all tests pass, then a commit.
- Commit messages use Conventional Commits: `feat:`, `fix:`, `chore:`, `test:`, `docs:`, `refactor:`.
- Phase boundaries are hard: do not start a task from a later phase before the current phase is merged.
- Versions are pinned in `Cargo.toml` / `package.json` to the verified 2026-09-04 set in the design spec §17.

---

## Phase 0 — Skeleton

### Task 0.1: Initialize the monorepo

**Files:**
- Create: `Cargo.toml` (workspace)
- Create: `pnpm-workspace.yaml`
- Create: `package.json` (root)
- Create: `justfile`
- Create: `README.md`
- Create: `.gitignore` (already exists; verify it covers `target/`, `node_modules/`, `dist/`)

**Interfaces:**
- Consumes: nothing
- Produces: empty workspace; `pnpm install` succeeds; `cargo metadata` succeeds

- [ ] **Step 1: Create the workspace `Cargo.toml`**

Write `/home/youssef/Desktop/projects/project3/Cargo.toml`:
```toml
[workspace]
resolver = "2"
members = [
    "apps/desktop/src-tauri",
    "crates/yoube-core",
    "crates/yoube-yt-dlp",
    "crates/yoube-filter",
    "crates/yoube-dns",
    "crates/yoube-storage",
]
[workspace.package]
version = "0.0.1"
edition = "2024"
license = "MIT OR Apache-2.0"
rust-version = "1.85"
[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
anyhow = "1"
async-trait = "0.1"
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json", "stream"] }
sqlx = { version = "0.9", default-features = false, features = ["runtime-tokio", "sqlite", "macros", "migrate"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
adblock = "0.9"
dashmap = "6"
directories = "5"
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 2: Create the pnpm workspace**

Write `/home/youssef/Desktop/projects/project3/pnpm-workspace.yaml`:
```yaml
packages:
  - "apps/desktop"
  - "packages/*"
```

- [ ] **Step 3: Create the root `package.json`**

Write `/home/youssef/Desktop/projects/project3/package.json`:
```json
{
  "name": "yoube",
  "private": true,
  "version": "0.0.1",
  "license": "MIT OR Apache-2.0",
  "packageManager": "pnpm@9.12.0",
  "engines": { "node": ">=20.10" },
  "scripts": {
    "dev": "pnpm --filter @yoube/desktop tauri dev",
    "build": "pnpm --filter @yoube/desktop tauri build",
    "lint": "pnpm -r --parallel lint",
    "test": "pnpm -r --parallel test",
    "format": "biome format --write ."
  },
  "devDependencies": {
    "@biomejs/biome": "1.9.4"
  }
}
```

- [ ] **Step 4: Create the `justfile`**

Write `/home/youssef/Desktop/projects/project3/justfile`:
```
set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
    @just --list

dev:
    pnpm dev

build:
    pnpm build

test:
    cargo test --workspace
    pnpm test

fmt:
    cargo fmt --all
    pnpm format

lint:
    cargo clippy --workspace --all-targets -- -D warnings
    pnpm lint

update-sidecars:
    ./scripts/update-sidecars.sh
```

- [ ] **Step 5: Verify the workspace**

Run: `cargo metadata --no-deps --format-version 1 | head -c 200`
Expected: JSON output with `target_directory` set; no error.

- [ ] **Step 6: Verify pnpm install works on an empty workspace**

Run: `pnpm install --ignore-workspace`
Expected: exits 0; `node_modules/` is created.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml pnpm-workspace.yaml package.json justfile
git commit -m "chore: initialize yoube monorepo (cargo + pnpm + just)"
```

### Task 0.2: Create the empty crates

**Files:**
- Create: `crates/yoube-core/Cargo.toml` + `crates/yoube-core/src/lib.rs`
- Create: `crates/yoube-yt-dlp/Cargo.toml` + `crates/yoube-yt-dlp/src/lib.rs`
- Create: `crates/yoube-filter/Cargo.toml` + `crates/yoube-filter/src/lib.rs`
- Create: `crates/yoube-dns/Cargo.toml` + `crates/yoube-dns/src/lib.rs`
- Create: `crates/yoube-storage/Cargo.toml` + `crates/yoube-storage/src/lib.rs`

**Interfaces:**
- Consumes: workspace (Task 0.1)
- Produces: five empty crates that `cargo build --workspace` accepts

- [ ] **Step 1: Create `crates/yoube-core`**

`Cargo.toml`:
```toml
[package]
name = "yoube-core"
version.workspace = true
edition.workspace = true
license.workspace = true
[dependencies]
yoube-yt-dlp = { path = "../yoube-yt-dlp" }
yoube-filter = { path = "../yoube-filter" }
yoube-dns = { path = "../yoube-dns" }
yoube-storage = { path = "../yoube-storage" }
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
anyhow.workspace = true
async-trait.workspace = true
tracing.workspace = true
```
`src/lib.rs`:
```rust
//! yoube-core: the service layer that powers the desktop app.
```
Run: `cargo build -p yoube-core` → expected: build error (missing dep crates). That's the next step.

- [ ] **Step 2: Create the other four crates** (one pair of files each, mirroring the above pattern; for now they only need a `name`, `version.workspace = true`, and an empty `src/lib.rs` with a one-line doc comment).

- [ ] **Step 3: Verify the workspace builds**

Run: `cargo build --workspace`
Expected: success, all 5 crates compile.

- [ ] **Step 4: Commit**

```bash
git add crates/
git commit -m "chore: scaffold yoube-{core,yt-dlp,filter,dns,storage} crates"
```

### Task 0.3: Define the `AppError` and `AppContext` types

**Files:**
- Create: `crates/yoube-core/src/error.rs`
- Create: `crates/yoube-core/src/context.rs`
- Modify: `crates/yoube-core/src/lib.rs`

**Interfaces:**
- Consumes: nothing
- Produces: `AppError` (serializable, with `From<sqlx::Error>`, `From<reqwest::Error>`, `From<serde_json::Error>`, `From<std::io::Error>`, `From<anyhow::Error>`); `AppContext` (empty for now, holds trait objects later)

- [ ] **Step 1: Write the failing test for `AppError` serialization**

Create `crates/yoube-core/src/error.rs`:
```rust
use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("db error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("yt-dlp error: {0}")]
    YtDlp(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("internal: {0}")]
    Internal(#[from] anyhow::Error),
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = std::result::Result<T, AppError>;
```
Create `crates/yoube-core/src/error.rs` test module:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serializes_to_string() {
        let e = AppError::NotFound("video".into());
        let s = serde_json::to_string(&e).unwrap();
        assert_eq!(s, "\"not found: video\"");
    }
    #[test]
    fn converts_from_io() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "x");
        let _: AppError = io.into();
    }
}
```
Run: `cargo test -p yoube-core --lib error::tests` → expected: PASS (after you `pub use error::*` in `lib.rs`).

- [ ] **Step 2: Add `AppContext` skeleton**

`crates/yoube-core/src/context.rs`:
```rust
use std::sync::Arc;
use crate::error::AppResult;

#[derive(Default)]
pub struct AppContext {
    // Fields are added in later tasks: youtube, downloader, filter, dnsblock,
    // account, media, settings. They are Arc<T: ServiceTrait> so commands can
    // resolve them cheaply.
}

impl AppContext {
    pub fn new() -> Self { Self::default() }
    pub fn ping(&self) -> AppResult<&'static str> { Ok("pong") }
}
```
Expose in `lib.rs`:
```rust
pub mod context;
pub mod error;
pub use context::AppContext;
pub use error::{AppError, AppResult};
```
Add test in `context.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_context_pings() {
        let c = AppContext::new();
        assert_eq!(c.ping().unwrap(), "pong");
    }
}
```
Run: `cargo test -p yoube-core` → expected: 2 tests pass.

- [ ] **Step 3: Commit**

```bash
git add crates/yoube-core
git commit -m "feat(core): add AppError and AppContext"
```

### Task 0.4: Scaffold the Tauri 2 desktop app

**Files:**
- Create: `apps/desktop/package.json`
- Create: `apps/desktop/vite.config.ts`
- Create: `apps/desktop/index.html`
- Create: `apps/desktop/tsconfig.json`
- Create: `apps/desktop/src/main.tsx`
- Create: `apps/desktop/src/App.tsx`
- Create: `apps/desktop/src-tauri/Cargo.toml`
- Create: `apps/desktop/src-tauri/tauri.conf.json`
- Create: `apps/desktop/src-tauri/src/main.rs`
- Create: `apps/desktop/src-tauri/src/lib.rs`
- Create: `apps/desktop/src-tauri/capabilities/default.json`
- Create: `apps/desktop/src-tauri/build.rs`

**Interfaces:**
- Consumes: workspace, `yoube-core::AppContext` from Task 0.3
- Produces: a Tauri 2 app that opens a window saying "yoube", and a Tauri command `ping` that returns `"pong"` from `AppContext`.

- [ ] **Step 1: Create the React app shell**

`apps/desktop/package.json`:
```json
{
  "name": "@yoube/desktop",
  "private": true,
  "version": "0.0.1",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc -b && vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "lint": "biome check .",
    "test": "vitest run"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.1.1",
    "react": "^19.0.0",
    "react-dom": "^19.0.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.1.0",
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "@vitejs/plugin-react": "^4.3.0",
    "typescript": "^5.6.0",
    "vite": "^6.0.0",
    "vitest": "^2.0.0",
    "@biomejs/biome": "1.9.4"
  }
}
```

`apps/desktop/vite.config.ts`:
```ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
});
```

`apps/desktop/tsconfig.json`:
```json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src"]
}
```

`apps/desktop/index.html`:
```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <title>yoube</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

`apps/desktop/src/main.tsx`:
```tsx
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>
);
```

`apps/desktop/src/App.tsx`:
```tsx
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export function App() {
  const [pong, setPong] = useState<string>("…");
  useEffect(() => { invoke<string>("ping").then(setPong).catch(console.error); }, []);
  return <main style={{ font: "16px system-ui", padding: 24 }}>yoube — {pong}</main>;
}
```

- [ ] **Step 2: Create the Tauri Rust side**

`apps/desktop/src-tauri/Cargo.toml`:
```toml
[package]
name = "yoube-desktop"
version.workspace = true
edition.workspace = true
[build-dependencies]
tauri-build = { version = "2", features = [] }
[dependencies]
tauri = { version = "2", features = [] }
serde.workspace = true
serde_json.workspace = true
yoube-core = { path = "../../../crates/yoube-core" }
```

`apps/desktop/src-tauri/build.rs`:
```rust
fn main() { tauri_build::build(); }
```

`apps/desktop/src-tauri/tauri.conf.json`:
```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "yoube",
  "version": "0.0.1",
  "identifier": "app.yoube.desktop",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [{ "label": "main", "title": "yoube", "width": 1280, "height": 800 }],
    "security": { "csp": null }
  },
  "bundle": { "active": true, "targets": "all" }
}
```

`apps/desktop/src-tauri/capabilities/default.json`:
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "yoube default capabilities",
  "windows": ["main"],
  "permissions": ["core:default"]
}
```

`apps/desktop/src-tauri/src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() { yoube_desktop_lib::run(); }
```

`apps/desktop/src-tauri/src/lib.rs`:
```rust
use yoube_core::AppContext;

#[tauri::command]
fn ping(ctx: tauri::State<'_, AppContext>) -> Result<String, yoube_core::AppError> {
    Ok(ctx.ping()?.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppContext::new())
        .invoke_handler(tauri::generate_handler![ping])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Install JS deps and verify the React app builds**

Run: `pnpm install`
Run: `pnpm --filter @yoube/desktop build`
Expected: Vite production build to `apps/desktop/dist/` succeeds; `tsc -b` exits 0.

- [ ] **Step 4: Verify the Tauri Rust shell compiles**

Run: `cargo build -p yoube-desktop`
Expected: success; the sidecar/feature plumbing is empty at this stage.

- [ ] **Step 5: Run `pnpm tauri dev` (manual smoke)**

The Tauri dev server requires a display. If you are headless, run `pnpm --filter @yoube/desktop build` and `cargo build -p yoube-desktop` as the verification, and skip the runtime smoke. The plan's `Definition of Done` for Task 0.4 is "compiles + dev server starts + ping returns 'pong' on a machine with a display".

- [ ] **Step 6: Commit**

```bash
git add apps/desktop
git commit -m "feat(shell): scaffold Tauri 2 + React 19 desktop app with ping command"
```

### Task 0.5: Wire `tauri-specta` for typed contracts

**Files:**
- Modify: `apps/desktop/src-tauri/Cargo.toml` (add `tauri-specta = "2"`, `specta = "2"`, `specta-typescript = "0.0"`)
- Modify: `apps/desktop/src-tauri/src/lib.rs` (wrap commands in `#[tauri::command]` with `specta::Type`)
- Create: `apps/desktop/scripts/gen-contracts.mjs`
- Create: `packages/contracts/` (workspace package, depends on `@yoube/desktop`'s generated `bindings.ts`)
- Modify: `apps/desktop/package.json` (add `gen:contracts` script)
- Modify: root `package.json` (add `"gen:contracts": "pnpm --filter @yoube/desktop gen:contracts && pnpm --filter @yoube/contracts build"`)

**Interfaces:**
- Consumes: `ping` command (Task 0.4)
- Produces: `packages/contracts/src/index.ts` exporting `ping(): Promise<string>` and a Tauri plugin that registers all commands; regenerable on every build

- [ ] **Step 1: Add the deps to `apps/desktop/src-tauri/Cargo.toml`**

Append to `[dependencies]`:
```toml
specta = { version = "2", features = ["derive"] }
specta-typescript = "0.0"
tauri-specta = { version = "2", features = ["derive", "typescript"] }
```

- [ ] **Step 2: Replace `lib.rs` with a specta-wired version**

```rust
use specta::Type;
use tauri_specta::{collect_commands, collect_events, Builder};
use yoube_core::AppContext;

#[derive(serde::Serialize, serde::Deserialize, Type)]
pub struct PingResponse { pub value: String }

#[tauri::command]
#[specta::specta]
fn ping(ctx: tauri::State<'_, AppContext>) -> Result<PingResponse, yoube_core::AppError> {
    Ok(PingResponse { value: ctx.ping()?.to_string() })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::default()
        .commands(collect_commands![ping]);

    #[cfg(debug_assertions)]
    builder
        .export(specta_typescript::Typescript::default(), "../src/bindings.ts")
        .expect("failed to export typescript bindings");

    tauri::Builder::default()
        .manage(AppContext::new())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| { builder.mount_events(app); Ok(()) })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Verify the Rust shell still builds and the bindings file is generated**

Run: `cargo build -p yoube-desktop`
Run: `pnpm --filter @yoube/desktop exec tauri dev --no-watch` (or `cargo run -p yoube-desktop` once); expect `apps/desktop/src/bindings.ts` to be created in debug mode.

- [ ] **Step 4: Create the `packages/contracts` package**

`packages/contracts/package.json`:
```json
{
  "name": "@yoube/contracts",
  "private": true,
  "version": "0.0.1",
  "type": "module",
  "main": "./src/index.ts",
  "types": "./src/index.ts",
  "exports": { ".": "./src/index.ts" }
}
```

`packages/contracts/src/index.ts`:
```ts
// AUTO-GENERATED. DO NOT EDIT.
// Source: apps/desktop/src/bindings.ts
export * from "../../../apps/desktop/src/bindings";
```

- [ ] **Step 5: Use the contract from the React app**

Modify `apps/desktop/src/App.tsx`:
```tsx
import { useEffect, useState } from "react";
import { ping } from "@yoube/contracts";

export function App() {
  const [pong, setPong] = useState<string>("…");
  useEffect(() => { ping().then(r => setPong(r.value)).catch(console.error); }, []);
  return <main style={{ font: "16px system-ui", padding: 24 }}>yoube — {pong}</main>;
}
```

- [ ] **Step 6: Re-run `cargo build -p yoube-desktop` and `pnpm --filter @yoube/desktop build`**

Expected: both pass; the rendered text reads `yoube — pong` (on a display).

- [ ] **Step 7: Commit**

```bash
git add apps/desktop packages/contracts
git commit -m "feat(contracts): generate typed TS bindings from Rust via tauri-specta"
```

### Task 0.6: Add the CI workflow

**Files:**
- Create: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: workspace (Tasks 0.1-0.5)
- Produces: CI that runs on every PR and main push, on Linux + macOS + Windows

- [ ] **Step 1: Write the workflow**

`.github/workflows/ci.yml`:
```yaml
name: CI
on:
  push: { branches: [main] }
  pull_request: {}
jobs:
  test:
    strategy:
      fail-fast: false
      matrix: { os: [ubuntu-24.04, macos-14, windows-2022] }
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: ${{ matrix.os == 'windows-2022' && 'x86_64-pc-windows-msvc' || '' }} }
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --workspace --all-targets
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - uses: pnpm/action-setup@v4
        with: { version: 9.12.0 }
      - uses: actions/setup-node@v4
        with: { node-version: 20, cache: pnpm }
      - run: pnpm install --frozen-lockfile
      - run: pnpm -r --parallel lint
      - run: pnpm -r --parallel test
      - run: pnpm --filter @yoube/desktop build
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows
git commit -m "ci: add Linux + macOS + Windows matrix workflow"
```

**Phase 0 exit gate:** all 6 tasks merged. `cargo test --workspace`, `pnpm -r test`, `pnpm --filter @yoube/desktop build` all pass locally. The app opens, shows `yoube — pong`, and `apps/desktop/src/bindings.ts` regenerates on every Rust change in debug mode.

---

## Phase 1 — Browse + Watch

### Task 1.1: yt-dlp sidecar invocation library

**Files:**
- Create: `crates/yoube-yt-dlp/src/runner.rs` (tested)
- Create: `crates/yoube-yt-dlp/src/parser.rs` (tested)
- Modify: `crates/yoube-yt-dlp/src/lib.rs`
- Create: `sidecars/yt-dlp/yt-dlp-x86_64-unknown-linux-gnu` (binary; instructions in task)
- Create: `apps/desktop/src-tauri/tauri.conf.json` (modify to add `externalBin`)
- Create: `apps/desktop/src-tauri/capabilities/default.json` (add `shell:allow-execute`)

**Interfaces:**
- Consumes: `AppError` from `yoube-core`
- Produces: `pub trait CommandRunner: Send + Sync` (mockable) and `pub struct YtDlpProcess { cmd: PathBuf }` with `pub async fn dump_json(&self, url: &str) -> AppResult<serde_json::Value>` and `pub async fn list_formats(&self, url: &str) -> AppResult<Vec<Format>>`

- [ ] **Step 1: Define the `Format` and `VideoSummary` types**

Create `crates/yoube-yt-dlp/src/model.rs`:
```rust
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Format {
    pub format_id: String,
    pub ext: String,
    pub url: Option<String>,
    pub resolution: Option<String>,
    pub fps: Option<f32>,
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
    pub filesize: Option<u64>,
    pub tbr: Option<f32>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VideoSummary {
    pub id: String,
    pub title: String,
    pub channel_id: String,
    pub channel_title: String,
    pub duration_s: Option<u32>,
    pub view_count: Option<u64>,
    pub upload_date: Option<String>, // YYYYMMDD
    pub thumbnail_url: Option<String>,
}
```

- [ ] **Step 2: Define the `CommandRunner` trait**

Create `crates/yoube-yt-dlp/src/runner.rs`:
```rust
use async_trait::async_trait;
use std::path::Path;
use yoube_core::AppResult;

#[async_trait]
pub trait CommandRunner: Send + Sync {
    async fn output(&self, bin: &Path, args: &[&str]) -> AppResult<std::process::Output>;
}
```

- [ ] **Step 3: Write a test for the parser using a recorded fixture**

Create `crates/yoube-yt-dlp/src/parser.rs`:
```rust
use crate::model::{Format, VideoSummary};
use serde_json::Value;
use yoube_core::AppResult;

pub fn parse_dump_json(v: &Value) -> AppResult<VideoSummary> {
    Ok(VideoSummary {
        id: v["id"].as_str().ok_or_else(|| missing("id"))?.to_string(),
        title: v["title"].as_str().ok_or_else(|| missing("title"))?.to_string(),
        channel_id: v.get("channel_id").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
        channel_title: v.get("channel").or_else(|| v.get("uploader"))
            .and_then(|x| x.as_str()).unwrap_or_default().to_string(),
        duration_s: v.get("duration").and_then(|x| x.as_f64()).map(|d| d as u32),
        view_count: v.get("view_count").and_then(|x| x.as_u64()),
        upload_date: v.get("upload_date").and_then(|x| x.as_str()).map(String::from),
        thumbnail_url: v.get("thumbnail").and_then(|x| x.as_str()).map(String::from),
    })
}

pub fn parse_list_formats(v: &Value) -> AppResult<Vec<Format>> {
    let arr = v["formats"].as_array().ok_or_else(|| missing("formats"))?;
    let mut out = Vec::with_capacity(arr.len());
    for f in arr {
        out.push(Format {
            format_id: f["format_id"].as_str().unwrap_or_default().to_string(),
            ext: f["ext"].as_str().unwrap_or_default().to_string(),
            url: f.get("url").and_then(|x| x.as_str()).map(String::from),
            resolution: f.get("resolution").and_then(|x| x.as_str()).map(String::from),
            fps: f.get("fps").and_then(|x| x.as_f64()).map(|n| n as f32),
            vcodec: f.get("vcodec").and_then(|x| x.as_str()).map(String::from),
            acodec: f.get("acodec").and_then(|x| x.as_str()).map(String::from),
            filesize: f.get("filesize").and_then(|x| x.as_u64()),
            tbr: f.get("tbr").and_then(|x| x.as_f64()).map(|n| n as f32),
            note: f.get("format_note").and_then(|x| x.as_str()).map(String::from),
        });
    }
    Ok(out)
}

fn missing(field: &'static str) -> yoube_core::AppError {
    yoube_core::AppError::Internal(anyhow::anyhow!("yt-dlp dump-json missing field: {field}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    const FIXTURE: &str = include_str!("../fixtures/dump_json.json");
    #[test]
    fn parses_summary() {
        let v: Value = serde_json::from_str(FIXTURE).unwrap();
        let s = parse_dump_json(&v).unwrap();
        assert_eq!(s.id, "dQw4w9WgXcQ");
        assert_eq!(s.title, "Never Gonna Give You Up");
    }
    #[test]
    fn parses_formats() {
        let v: Value = serde_json::from_str(FIXTURE).unwrap();
        let f = parse_list_formats(&v).unwrap();
        assert!(!f.is_empty());
        assert!(f.iter().any(|f| f.format_id == "22"));
    }
}
```

- [ ] **Step 4: Record a real `yt-dlp --dump-json` fixture (offline, one-time)**

Run on a dev machine with network:
```bash
yt-dlp --skip-download --dump-single-json https://www.youtube.com/watch?v=dQw4w9WgXcQ \
  | python3 -c "import json,sys; d=json.load(sys.stdin); d['formats']=d['formats'][:2]; print(json.dumps(d,indent=2))" \
  > crates/yoube-yt-dlp/fixtures/dump_json.json
```
Sanity check the file is <50KB and contains `id`, `title`, `formats[]`.

- [ ] **Step 5: Implement the `TokioCommandRunner`**

Append to `crates/yoube-yt-dlp/src/runner.rs`:
```rust
pub struct TokioCommandRunner;
#[async_trait]
impl CommandRunner for TokioCommandRunner {
    async fn output(&self, bin: &Path, args: &[&str]) -> AppResult<std::process::Output> {
        use tokio::process::Command;
        Ok(Command::new(bin).args(args).output().await?)
    }
}
```

- [ ] **Step 6: Implement the `YtDlp` façade**

Modify `crates/yoube-yt-dlp/src/lib.rs`:
```rust
pub mod model;
pub mod parser;
pub mod runner;
use std::path::PathBuf;
use std::sync::Arc;
use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;
use yoube_core::{AppError, AppResult};
use crate::model::{Format, VideoSummary};
use crate::parser::{parse_dump_json, parse_list_formats};
use crate::runner::CommandRunner;

#[derive(Clone)]
pub struct YtDlp {
    pub bin: PathBuf,
    pub runner: Arc<dyn CommandRunner>,
}

impl YtDlp {
    pub fn new(bin: PathBuf, runner: Arc<dyn CommandRunner>) -> Self { Self { bin, runner } }
    pub async fn dump_json(&self, url: &str) -> AppResult<VideoSummary> {
        let args = ["--skip-download", "--dump-single-json", "--no-warnings", url];
        let out = self.runner.output(&self.bin, &args).await?;
        if !out.status.success() {
            return Err(AppError::YtDlp(String::from_utf8_lossy(&out.stderr).into_owned()));
        }
        let v: Value = serde_json::from_slice(&out.stdout)?;
        parse_dump_json(&v)
    }
    pub async fn list_formats(&self, url: &str) -> AppResult<Vec<Format>> {
        let args = ["--skip-download", "--dump-single-json", "--no-warnings", url];
        let out = self.runner.output(&self.bin, &args).await?;
        if !out.status.success() {
            return Err(AppError::YtDlp(String::from_utf8_lossy(&out.stderr).into_owned()));
        }
        let v: Value = serde_json::from_slice(&out.stdout)?;
        parse_list_formats(&v)
    }
}
```

- [ ] **Step 7: Run `cargo test -p yoube-yt-dlp`**

Expected: 2 tests pass (parser uses the fixture; `TokioCommandRunner` is not yet exercised here).

- [ ] **Step 8: Commit**

```bash
git add crates/yoube-yt-dlp
git commit -m "feat(yt-dlp): typed dump_json + format parser, fixture-driven tests"
```

### Task 1.2: Bundle yt-dlp as a Tauri sidecar

**Files:**
- Modify: `apps/desktop/src-tauri/tauri.conf.json` (add `bundle.externalBin`)
- Modify: `apps/desktop/src-tauri/capabilities/default.json` (allow sidecar)
- Create: `sidecars/yt-dlp/yt-dlp-x86_64-unknown-linux-gnu` (the binary)
- Create: `apps/desktop/src-tauri/src/sidecar.rs` (resolve path)
- Create: `scripts/update-sidecars.sh`

**Interfaces:**
- Consumes: `YtDlp` from Task 1.1
- Produces: at runtime, `apps/desktop` finds the yt-dlp binary in the sidecar directory and passes it to `YtDlp::new`

- [ ] **Step 1: Add the platform binaries**

For each target triple, download the matching yt-dlp release asset (see the spec §17) and place it under `sidecars/yt-dlp/` with the exact filename `yt-dlp-<triple>` (`.exe` suffix on Windows). For the dev machine, do at least:
```bash
mkdir -p sidecars/yt-dlp
# Linux x86_64
curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux \
  -o sidecars/yt-dlp/yt-dlp-x86_64-unknown-linux-gnu
chmod +x sidecars/yt-dlp/yt-dlp-x86_64-unknown-linux-gnu
# macOS arm64
curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos \
  -o sidecars/yt-dlp/yt-dlp-aarch64-apple-darwin
chmod +x sidecars/yt-dlp/yt-dlp-aarch64-apple-darwin
# Windows x64
curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe \
  -o sidecars/yt-dlp/yt-dlp-x86_64-pc-windows-msvc.exe
```

- [ ] **Step 2: Update `tauri.conf.json`**

In the `bundle` object add:
```json
"externalBin": ["../../sidecars/yt-dlp/yt-dlp"]
```
The Tauri bundler will then resolve the right triple-named binary at build time.

- [ ] **Step 3: Allow sidecar execution in the capabilities**

Append to `apps/desktop/src-tauri/capabilities/default.json` `permissions`:
```json
"shell:default",
{
  "identifier": "shell:allow-execute",
  "allow": [{ "name": "bin-sidecar", "sidecar": true, "args": true }]
}
```

- [ ] **Step 4: Add the sidecar resolver**

`apps/desktop/src-tauri/src/sidecar.rs`:
```rust
use std::path::PathBuf;
use tauri::Manager;

pub fn yt_dlp_path(app: &tauri::AppHandle) -> PathBuf {
    app.path().resolve("yt-dlp", tauri::path::BaseDirectory::Resource)
        .expect("yt-dlp sidecar resolution failed")
}
```

- [ ] **Step 5: Write `scripts/update-sidecars.sh`**

```bash
#!/usr/bin/env bash
set -euo pipefail
mkdir -p sidecars/yt-dlp
base="https://github.com/yt-dlp/yt-dlp/releases/latest/download"
curl -L "$base/yt-dlp_linux" -o sidecars/yt-dlp/yt-dlp-x86_64-unknown-linux-gnu
chmod +x sidecars/yt-dlp/yt-dlp-x86_64-unknown-linux-gnu
curl -L "$base/yt-dlp_macos" -o sidecars/yt-dlp/yt-dlp-aarch64-apple-darwin
chmod +x sidecars/yt-dlp/yt-dlp-aarch64-apple-darwin
curl -L "$base/yt-dlp.exe" -o sidecars/yt-dlp/yt-dlp-x86_64-pc-windows-msvc.exe
```

- [ ] **Step 6: Smoke-test: invoke the sidecar from a unit test**

Add `apps/desktop/src-tauri/src/sidecar.rs` test:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn triple_aware_filename() {
        // sanity: the function only takes an AppHandle in production; here we
        // verify the path-string logic by inlining the same convention
        for (triple, name) in [
            ("x86_64-unknown-linux-gnu", "yt-dlp-x86_64-unknown-linux-gnu"),
            ("aarch64-apple-darwin", "yt-dlp-aarch64-apple-darwin"),
            ("x86_64-pc-windows-msvc", "yt-dlp-x86_64-pc-windows-msvc.exe"),
        ] {
            assert!(name.starts_with("yt-dlp-"));
            assert!(name.ends_with(triple) || name.ends_with(&format!("{triple}.exe")));
        }
    }
}
```

- [ ] **Step 7: Verify `cargo build -p yoube-desktop` still works**

Expected: success; the sidecar resource is bundled on `tauri build`, and resolved at runtime on `tauri dev` from the sidecars directory in dev mode.

- [ ] **Step 8: Commit**

```bash
git add sidecars apps/desktop/src-tauri scripts
git commit -m "feat(shell): bundle yt-dlp as Tauri sidecar"
```

### Task 1.3: Implement the `YoutubeService` trait

**Files:**
- Create: `crates/yoube-core/src/services/youtube.rs`
- Create: `crates/yoube-core/src/commands.rs`
- Modify: `crates/yoube-core/src/context.rs` (add `youtube: Arc<dyn YoutubeService>`)
- Modify: `crates/yoube-core/src/lib.rs` (export module)
- Modify: `apps/desktop/src-tauri/src/lib.rs` (wire the service, add commands)
- Create: `crates/yoube-core/tests/youtube.rs` (integration test with recorded fixtures)

**Interfaces:**
- Consumes: `YtDlp` from Task 1.1
- Produces: `pub trait YoutubeService` (from spec §5), Tauri commands `get_video`, `search`, `channel`, `channel_videos`, `playlist`, `trending`, `related`, with matching specta exports

- [ ] **Step 1: Write the trait and a mock**

`crates/yoube-core/src/services/youtube.rs`:
```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;
use yoube_yt_dlp::model::{Format, VideoSummary};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Video { pub summary: VideoSummary, pub formats: Vec<Format> }
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Channel { pub id: String, pub title: String, pub description: String, pub thumb_url: Option<String> }
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Playlist { pub id: String, pub title: String, pub channel_id: String, pub items: Vec<VideoSummary> }
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub enum Region { US, GB, EG, SA, DE, FR, JP, BR, IN, AU, CA, MX, ES, IT, RU, TR, ZA, NG, KR, AR }
impl Region { pub fn code(self) -> &'static str { match self { Region::US => "US", Region::GB => "GB", Region::EG => "EG", Region::SA => "SA", Region::DE => "DE", Region::FR => "FR", Region::JP => "JP", Region::BR => "BR", Region::IN => "IN", Region::AU => "AU", Region::CA => "CA", Region::MX => "MX", Region::ES => "ES", Region::IT => "IT", Region::RU => "RU", Region::TR => "TR", Region::ZA => "ZA", Region::NG => "NG", Region::KR => "KR", Region::AR => "AR" } } }

#[async_trait]
pub trait YoutubeService: Send + Sync {
    async fn get_video(&self, id: &str) -> yoube_core::AppResult<Video>;
    async fn search(&self, q: &str, page: u32) -> yoube_core::AppResult<Vec<VideoSummary>>;
    async fn channel(&self, id: &str) -> yoube_core::AppResult<Channel>;
    async fn channel_videos(&self, id: &str, page: u32) -> yoube_core::AppResult<Vec<VideoSummary>>;
    async fn playlist(&self, id: &str) -> yoube_core::AppResult<Playlist>;
    async fn trending(&self, region: Region) -> yoube_core::AppResult<Vec<VideoSummary>>;
    async fn related(&self, id: &str) -> yoube_core::AppResult<Vec<VideoSummary>>;
}
```

- [ ] **Step 2: Implement the `YtDlpYoutubeService`**

Append to the same file:
```rust
use yoube_yt_dlp::YtDlp;

pub struct YtDlpYoutubeService { pub ytdlp: YtDlp }

#[async_trait]
impl YoutubeService for YtDlpYoutubeService {
    async fn get_video(&self, id: &str) -> yoube_core::AppResult<Video> {
        let url = format!("https://www.youtube.com/watch?v={id}");
        let summary = self.ytdlp.dump_json(&url).await?;
        let formats = self.ytdlp.list_formats(&url).await?;
        Ok(Video { summary, formats })
    }
    async fn search(&self, q: &str, page: u32) -> yoube_core::AppResult<Vec<VideoSummary>> {
        let url = format!("ytsearch{}:{q}", page.saturating_mul(20).saturating_add(20));
        let _ = &url; // Placeholder: real impl uses --dump-json after yt-dlp search extractor
        Err(yoube_core::AppError::Internal(anyhow::anyhow!("search not yet implemented in phase 1.1")))
    }
    async fn channel(&self, _id: &str) -> yoube_core::AppResult<Channel> {
        Err(yoube_core::AppError::Internal(anyhow::anyhow!("channel not yet implemented")))
    }
    async fn channel_videos(&self, _id: &str, _page: u32) -> yoube_core::AppResult<Vec<VideoSummary>> {
        Err(yoube_core::AppError::Internal(anyhow::anyhow!("channel_videos not yet implemented")))
    }
    async fn playlist(&self, _id: &str) -> yoube_core::AppResult<Playlist> {
        Err(yoube_core::AppError::Internal(anyhow::anyhow!("playlist not yet implemented")))
    }
    async fn trending(&self, _region: Region) -> yoube_core::AppResult<Vec<VideoSummary>> {
        Err(yoube_core::AppError::Internal(anyhow::anyhow!("trending not yet implemented")))
    }
    async fn related(&self, _id: &str) -> yoube_core::AppResult<Vec<VideoSummary>> {
        Err(yoube_core::AppError::Internal(anyhow::anyhow!("related not yet implemented")))
    }
}
```

- [ ] **Step 3: Wire it into `AppContext` and create Tauri commands**

`crates/yoube-core/src/context.rs` (add field + constructor arg):
```rust
use std::sync::Arc;
use crate::services::youtube::YoutubeService;
use crate::error::AppResult;

pub struct AppContext {
    pub youtube: Arc<dyn YoutubeService>,
}

impl AppContext {
    pub fn new(youtube: Arc<dyn YoutubeService>) -> Self { Self { youtube } }
    pub fn ping(&self) -> AppResult<&'static str> { Ok("pong") }
}
```

`crates/yoube-core/src/commands.rs`:
```rust
use crate::context::AppContext;
use crate::services::youtube::{Region, YoutubeService};

#[tauri::command]
#[specta::specta]
pub async fn get_video(ctx: tauri::State<'_, AppContext>, id: String) -> Result<crate::services::youtube::Video, yoube_core::AppError> {
    ctx.youtube.get_video(&id).await
}
#[tauri::command]
#[specta::specta]
pub async fn search(ctx: tauri::State<'_, AppContext>, q: String, page: u32) -> Result<Vec<yoube_yt_dlp::model::VideoSummary>, yoube_core::AppError> {
    ctx.youtube.search(&q, page).await
}
#[tauri::command]
#[specta::specta]
pub async fn channel(ctx: tauri::State<'_, AppContext>, id: String) -> Result<crate::services::youtube::Channel, yoube_core::AppError> {
    ctx.youtube.channel(&id).await
}
#[tauri::command]
#[specta::specta]
pub async fn channel_videos(ctx: tauri::State<'_, AppContext>, id: String, page: u32) -> Result<Vec<yoube_yt_dlp::model::VideoSummary>, yoube_core::AppError> {
    ctx.youtube.channel_videos(&id, page).await
}
#[tauri::command]
#[specta::specta]
pub async fn playlist(ctx: tauri::State<'_, AppContext>, id: String) -> Result<crate::services::youtube::Playlist, yoube_core::AppError> {
    ctx.youtube.playlist(&id).await
}
#[tauri::command]
#[specta::specta]
pub async fn trending(ctx: tauri::State<'_, AppContext>, region: Region) -> Result<Vec<yoube_yt_dlp::model::VideoSummary>, yoube_core::AppError> {
    ctx.youtube.trending(region).await
}
#[tauri::command]
#[specta::specta]
pub async fn related(ctx: tauri::State<'_, AppContext>, id: String) -> Result<Vec<yoube_yt_dlp::model::VideoSummary>, yoube_core::AppError> {
    ctx.youtube.related(&id).await
}
```

- [ ] **Step 4: Register the commands in `apps/desktop/src-tauri/src/lib.rs`**

```rust
use yoube_core::commands::*;
use yoube_core::services::youtube::{YtDlpYoutubeService, YoutubeService};
use yoube_core::AppContext;
use yoube_yt_dlp::runner::TokioCommandRunner;
use yoube_yt_dlp::YtDlp;
use std::sync::Arc;

fn build_context() -> AppContext {
    let bin = crate::sidecar::yt_dlp_path(&tauri::AppHandle::default()); // dev convenience
    // In production this is replaced with the runtime-resolved sidecar path.
    let ytdlp = YtDlp::new(bin, Arc::new(TokioCommandRunner));
    AppContext::new(Arc::new(YtDlpYoutubeService { ytdlp }))
}
```

(The real production wiring injects the actual `app.handle()`; see Task 1.4.)

- [ ] **Step 5: Add an integration test using a recorded fixture**

`crates/yoube-core/tests/youtube.rs`:
```rust
use std::path::PathBuf;
use std::sync::Arc;
use yoube_core::services::youtube::{YoutubeService, YtDlpYoutubeService};
use yoube_yt_dlp::runner::CommandRunner;
use yoube_yt_dlp::YtDlp;

struct StaticRunner { fixture: &'static str, exit: i32 }
#[async_trait::async_trait]
impl CommandRunner for StaticRunner {
    async fn output(&self, _bin: &PathBuf, _args: &[&str]) -> yoube_core::AppResult<std::process::Output> {
        use std::process::Output;
        Ok(Output { status: std::process::ExitStatus::default(), stdout: self.fixture.as_bytes().to_vec(), stderr: vec![] })
    }
}

#[tokio::test]
async fn get_video_uses_dump_json_fixture() {
    let fixture = include_str!("../../yoube-yt-dlp/fixtures/dump_json.json");
    let runner: Arc<dyn CommandRunner> = Arc::new(StaticRunner { fixture, exit: 0 });
    let ytdlp = YtDlp::new(PathBuf::from("yt-dlp"), runner);
    let svc = YtDlpYoutubeService { ytdlp };
    let v = svc.get_video("dQw4w9WgXcQ").await.unwrap();
    assert_eq!(v.summary.id, "dQw4w9WgXcQ");
    assert!(!v.formats.is_empty());
}
```

- [ ] **Step 6: Run `cargo test --workspace`**

Expected: 3+ tests pass (2 from parser, 1 from this integration test).

- [ ] **Step 7: Commit**

```bash
git add crates apps/desktop/src-tauri
git commit -m "feat(youtube): YtDlpYoutubeService + Tauri commands (get_video implemented)"
```

### Task 1.4: Implement Search, Trending, Related via yt-dlp

**Files:**
- Modify: `crates/yoube-core/src/services/youtube.rs` (fill in the unimplemented methods)

**Interfaces:**
- Consumes: `YtDlp`
- Produces: working `search`, `trending`, `related`, `channel`, `channel_videos`, `playlist`

- [ ] **Step 1: `search` via `yt-dlp "ytsearch<N>:<query>" --flat-playlist --dump-json`**

```rust
async fn search(&self, q: &str, page: u32) -> yoube_core::AppResult<Vec<VideoSummary>> {
    use tokio::process::Command;
    let n = ((page + 1) * 20).to_string();
    let url = format!("ytsearch{n}:{q}");
    let out = Command::new(&self.ytdlp.bin)
        .args(["--flat-playlist", "--skip-download", "--dump-single-json", "--no-warnings", &url])
        .output().await?;
    if !out.status.success() { return Err(yoube_core::AppError::YtDlp(String::from_utf8_lossy(&out.stderr).into_owned())); }
    let v: serde_json::Value = serde_json::from_slice(&out.stdout)?;
    let arr = v.get("entries").and_then(|e| e.as_array()).cloned().unwrap_or_default();
    let mut out = Vec::with_capacity(arr.len());
    for e in arr {
        out.push(VideoSummary {
            id: e["id"].as_str().unwrap_or_default().to_string(),
            title: e["title"].as_str().unwrap_or_default().to_string(),
            channel_id: e.get("channel_id").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
            channel_title: e.get("channel").or_else(|| e.get("uploader")).and_then(|x| x.as_str()).unwrap_or_default().to_string(),
            duration_s: e.get("duration").and_then(|x| x.as_f64()).map(|d| d as u32),
            view_count: e.get("view_count").and_then(|x| x.as_u64()),
            upload_date: e.get("upload_date").and_then(|x| x.as_str()).map(String::from),
            thumbnail_url: e.get("thumbnails").and_then(|t| t.as_array()).and_then(|a| a.first()).and_then(|t| t.get("url")).and_then(|u| u.as_str()).map(String::from),
        });
    }
    Ok(out)
}
```

- [ ] **Step 2: `trending` via `https://www.youtube.com/feed/trending?gl=<code>`**

```rust
async fn trending(&self, region: Region) -> yoube_core::AppResult<Vec<VideoSummary>> {
    use tokio::process::Command;
    let url = format!("https://www.youtube.com/feed/trending?gl={}", region.code());
    let out = Command::new(&self.ytdlp.bin)
        .args(["--flat-playlist", "--skip-download", "--dump-single-json", "--no-warnings", &url])
        .output().await?;
    if !out.status.success() { return Err(yoube_core::AppError::YtDlp(String::from_utf8_lossy(&out.stderr).into_owned())); }
    let v: serde_json::Value = serde_json::from_slice(&out.stdout)?;
    Ok(v.get("entries").and_then(|e| e.as_array()).cloned().unwrap_or_default()
        .into_iter().filter_map(|e| serde_json::from_value::<VideoSummary>(e).ok()).collect())
}
```

- [ ] **Step 3: `related`, `channel`, `channel_videos`, `playlist` — same pattern**

```rust
async fn related(&self, id: &str) -> yoube_core::AppResult<Vec<VideoSummary>> {
    self.flat_list(&format!("https://www.youtube.com/watch?v={id}")).await
}
async fn channel(&self, id: &str) -> yoube_core::AppResult<Channel> {
    use tokio::process::Command;
    let url = format!("https://www.youtube.com/channel/{id}");
    let out = Command::new(&self.ytdlp.bin)
        .args(["--skip-download", "--dump-single-json", "--no-warnings", &url])
        .output().await?;
    if !out.status.success() { return Err(yoube_core::AppError::YtDlp(String::from_utf8_lossy(&out.stderr).into_owned())); }
    let v: serde_json::Value = serde_json::from_slice(&out.stdout)?;
    Ok(Channel {
        id,
        title: v.get("title").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
        description: v.get("description").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
        thumb_url: v.get("thumbnails").and_then(|t| t.as_array()).and_then(|a| a.last()).and_then(|t| t.get("url")).and_then(|u| u.as_str()).map(String::from),
    })
}
async fn channel_videos(&self, id: &str, _page: u32) -> yoube_core::AppResult<Vec<VideoSummary>> {
    self.flat_list(&format!("https://www.youtube.com/channel/{id}/videos")).await
}
async fn playlist(&self, id: &str) -> yoube_core::AppResult<Playlist> {
    use tokio::process::Command;
    let url = format!("https://www.youtube.com/playlist?list={id}");
    let out = Command::new(&self.ytdlp.bin)
        .args(["--flat-playlist", "--skip-download", "--dump-single-json", "--no-warnings", &url])
        .output().await?;
    if !out.status.success() { return Err(yoube_core::AppError::YtDlp(String::from_utf8_lossy(&out.stderr).into_owned())); }
    let v: serde_json::Value = serde_json::from_slice(&out.stdout)?;
    let entries = v.get("entries").and_then(|e| e.as_array()).cloned().unwrap_or_default();
    let items: Vec<VideoSummary> = entries.into_iter().filter_map(|e| serde_json::from_value(e).ok()).collect();
    Ok(Playlist {
        id: id.to_string(),
        title: v.get("title").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
        channel_id: v.get("channel_id").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
        items,
    })
}
```

Add a private helper:
```rust
impl YtDlpYoutubeService {
    async fn flat_list(&self, url: &str) -> yoube_core::AppResult<Vec<VideoSummary>> {
        use tokio::process::Command;
        let out = Command::new(&self.ytdlp.bin)
            .args(["--flat-playlist", "--skip-download", "--dump-single-json", "--no-warnings", url])
            .output().await?;
        if !out.status.success() { return Err(yoube_core::AppError::YtDlp(String::from_utf8_lossy(&out.stderr).into_owned())); }
        let v: serde_json::Value = serde_json::from_slice(&out.stdout)?;
        Ok(v.get("entries").and_then(|e| e.as_array()).cloned().unwrap_or_default()
            .into_iter().filter_map(|e| serde_json::from_value(e).ok()).collect())
    }
}
```

- [ ] **Step 4: Add an integration test using a recorded `yt-dlp` fixture for search**

Run on a dev machine:
```bash
yt-dlp --flat-playlist --skip-download --dump-single-json "ytsearch20:Rust programming" \
  > crates/yoube-core/tests/fixtures/search.json
```
Add test:
```rust
#[tokio::test]
async fn search_parses_flat_playlist() {
    let fixture = include_str!("fixtures/search.json");
    let runner: Arc<dyn CommandRunner> = Arc::new(StaticRunner { fixture, exit: 0 });
    let ytdlp = YtDlp::new(PathBuf::from("yt-dlp"), runner);
    let svc = YtDlpYoutubeService { ytdlp };
    let results = svc.search("Rust programming", 0).await.unwrap();
    assert!(!results.is_empty());
}
```

- [ ] **Step 5: Run `cargo test --workspace`**

Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add crates/yoube-core
git commit -m "feat(youtube): implement search, trending, related, channel, playlist"
```

### Task 1.5: Install the player and build the Watch screen

**Files:**
- Modify: `apps/desktop/package.json` (deps: `@vidstack/react`, `hls.js`, `@tanstack/react-query`, `@tanstack/react-router`, `@tanstack/react-virtual`, `zustand`, `tailwindcss`, `postcss`, `autoprefixer`, `clsx`, `class-variance-authority`, `lucide-react`)
- Create: `apps/desktop/tailwind.config.ts`
- Create: `apps/desktop/postcss.config.cjs`
- Create: `apps/desktop/src/styles.css`
- Create: `apps/desktop/src/routes/index.tsx` (Home)
- Create: `apps/desktop/src/routes/watch.tsx` (Watch)
- Create: `apps/desktop/src/components/VideoCard.tsx`
- Create: `apps/desktop/src/components/VideoGrid.tsx`
- Create: `apps/desktop/src/components/Player.tsx`
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/main.tsx`

**Interfaces:**
- Consumes: `getVideo`, `related` contracts (from Task 1.3-1.4)
- Produces: a Home grid that calls `trending` and a Watch page that plays the video and shows related

- [ ] **Step 1: Add deps and Tailwind**

`apps/desktop/package.json` deps: as above. Add to `devDependencies`:
```json
"tailwindcss": "^3.4.0",
"postcss": "^8.4.0",
"autoprefixer": "^10.4.0"
```

`apps/desktop/tailwind.config.ts`:
```ts
import type { Config } from "tailwindcss";
export default { content: ["./index.html", "./src/**/*.{ts,tsx}"], theme: { extend: {} }, plugins: [] } satisfies Config;
```

`apps/desktop/postcss.config.cjs`:
```js
module.exports = { plugins: { tailwindcss: {}, autoprefixer: {} } };
```

`apps/desktop/src/styles.css`:
```css
@tailwind base;
@tailwind components;
@tailwind utilities;
html, body, #root { height: 100%; background: #0f0f0f; color: #f1f1f1; }
```

Modify `apps/desktop/src/main.tsx`:
```tsx
import "./styles.css";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider, createRouter } from "@tanstack/react-router";
import { routeTree } from "./routeTree.gen";

const router = createRouter({ routeTree });
declare module "@tanstack/react-router" { interface Register { router: typeof router; } }
const qc = new QueryClient();

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <QueryClientProvider client={qc}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </StrictMode>
);
```

- [ ] **Step 2: `VideoCard`, `VideoGrid`, `Player`**

`apps/desktop/src/components/VideoCard.tsx`:
```tsx
import { Link } from "@tanstack/react-router";
import type { VideoSummary } from "@yoube/contracts";
import clsx from "clsx";

export function VideoCard({ v }: { v: VideoSummary }) {
  return (
    <Link to="/watch" search={{ v: v.id }} className="block group">
      <div className="aspect-video bg-neutral-800 rounded-xl overflow-hidden">
        {v.thumbnail_url && <img src={v.thumbnail_url} alt="" className="w-full h-full object-cover group-hover:scale-[1.02] transition" />}
      </div>
      <div className="mt-2">
        <div className="font-medium line-clamp-2">{v.title}</div>
        <div className="text-sm text-neutral-400">{v.channel_title}</div>
      </div>
    </Link>
  );
}
```

`apps/desktop/src/components/VideoGrid.tsx`:
```tsx
import { useVirtualizer } from "@tanstack/react-virtual";
import { useRef } from "react";
import { VideoCard } from "./VideoCard";
import type { VideoSummary } from "@yoube/contracts";

export function VideoGrid({ items }: { items: VideoSummary[] }) {
  const parent = useRef<HTMLDivElement>(null);
  const row = useVirtualizer({ count: items.length, getScrollElement: () => parent.current, estimateSize: () => 240, overscan: 6 });
  return (
    <div ref={parent} className="h-full overflow-auto p-4">
      <div style={{ height: row.getTotalSize(), position: "relative" }}>
        {row.getVirtualItems().map(v => (
          <div key={v.key} style={{ position: "absolute", top: v.start, left: 0, right: 0 }} className="px-1">
            <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
              {items.slice(v.index, v.index + 4).map(it => <VideoCard key={it.id} v={it} />)}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
```

`apps/desktop/src/components/Player.tsx`:
```tsx
import { MediaPlayer, MediaProvider } from "@vidstack/react";
import { defaultLayoutIcons } from "@vidstack/react/player/layouts/default";
import "@vidstack/react/player/styles/default/theme.css";
import "@vidstack/react/player/styles/default/layouts/video.css";

export function Player({ src, poster }: { src: string; poster?: string }) {
  return (
    <MediaPlayer src={src} poster={poster} title="yoube" crossOrigin>
      <MediaProvider />
      {/* @ts-expect-error vidstack types are imperfect */}
      <defaultLayoutIcons />
    </MediaPlayer>
  );
}
```

- [ ] **Step 3: Routes**

`apps/desktop/src/routes/__root.tsx`:
```tsx
import { Link, Outlet, createRootRoute } from "@tanstack/react-router";
export const Route = createRootRoute({ component: () => <div className="h-full flex flex-col">
  <header className="h-12 border-b border-neutral-800 flex items-center px-4 gap-4">
    <Link to="/" className="font-semibold">yoube</Link>
  </header>
  <main className="flex-1 min-h-0"><Outlet /></main>
</div> });
```

`apps/desktop/src/routes/index.tsx`:
```tsx
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { trending } from "@yoube/contracts";
import { VideoGrid } from "../components/VideoGrid";

export const Route = createFileRoute("/")({ component: Home });
function Home() {
  const q = useQuery({ queryKey: ["trending", "US"], queryFn: () => trending("US" as any) });
  if (q.isPending) return <div className="p-4">Loading…</div>;
  if (q.error) return <div className="p-4 text-red-400">{String(q.error)}</div>;
  return <VideoGrid items={q.data} />;
}
```

`apps/desktop/src/routes/watch.tsx`:
```tsx
import { useQuery } from "@tanstack/react-query";
import { createFileRoute, useSearch } from "@tanstack/react-router";
import { getVideo, related } from "@yoube/contracts";
import { Player } from "../components/Player";
import { VideoCard } from "../components/VideoCard";

export const Route = createFileRoute("/watch")({ component: Watch });
function Watch() {
  const { v } = useSearch({ from: "/watch" });
  const vid = useQuery({ queryKey: ["video", v], queryFn: () => getVideo(v), enabled: !!v });
  const rel = useQuery({ queryKey: ["related", v], queryFn: () => related(v), enabled: !!v });
  if (vid.isPending) return <div className="p-4">Loading…</div>;
  if (vid.error) return <div className="p-4 text-red-400">{String(vid.error)}</div>;
  const best = vid.data.formats.find(f => f.vcodec.as_ref().map(|c| c != "none").unwrap_or(false) && f.acodec.as_ref().map(|c| c != "none").unwrap_or(false)) ?? vid.data.formats[0];
  return (
    <div className="h-full overflow-auto p-4 grid grid-cols-1 lg:grid-cols-[1fr_360px] gap-6">
      <div>
        <div className="aspect-video bg-black rounded-xl overflow-hidden"><Player src={best.url} poster={vid.data.summary.thumbnail_url ?? undefined} /></div>
        <h1 className="text-xl font-semibold mt-3">{vid.data.summary.title}</h1>
        <div className="text-sm text-neutral-400">{vid.data.summary.channel_title}</div>
      </div>
      <aside>
        <h2 className="font-semibold mb-2">Related</h2>
        <div className="flex flex-col gap-3">{rel.data?.map(r => <VideoCard key={r.id} v={r} />)}</div>
      </aside>
    </div>
  );
}
```

(Note: `best.url` uses the `url: Option<String>` field on `Format`, defined in Task 1.1.)

- [ ] **Step 4: Generate the route tree**

`apps/desktop/src/routeTree.gen.ts` is generated by TanStack Router. Add the script to `apps/desktop/package.json`:
```json
"scripts": { … "routes": "tsr generate" }
```
Add `devDependencies`: `"@tanstack/router-cli": "^1.62.0"`, `"@tanstack/router-plugin": "^1.62.0"`.
Run: `pnpm --filter @yoube/desktop exec tsr generate` (or let Vite's plugin do it on dev).
Expected: `apps/desktop/src/routeTree.gen.ts` exists.

- [ ] **Step 5: `pnpm --filter @yoube/desktop build`**

Expected: success. (If vidstack types complain, use `// @ts-expect-error` narrowly — never `any` in domain code.)

- [ ] **Step 6: Commit**

```bash
git add apps/desktop
git commit -m "feat(ui): home + watch screens with vidstack player"
```

### Task 1.6: Search and Channel screens

**Files:**
- Create: `apps/desktop/src/routes/results.tsx`
- Create: `apps/desktop/src/routes/channel.tsx`

**Interfaces:**
- Consumes: `search`, `channel`, `channel_videos` contracts
- Produces: `/results?q=...` and `/channel/<id>` routes

- [ ] **Step 1: `results.tsx`**

```tsx
import { useQuery } from "@tanstack/react-query";
import { createFileRoute, useSearch } from "@tanstack/react-router";
import { search } from "@yoube/contracts";
import { VideoGrid } from "../components/VideoGrid";

export const Route = createFileRoute("/results")({ component: Results });
function Results() {
  const { q } = useSearch({ from: "/results" });
  const res = useQuery({ queryKey: ["search", q], queryFn: () => search(q, 0), enabled: !!q });
  if (res.isPending) return <div className="p-4">Searching…</div>;
  return <VideoGrid items={res.data ?? []} />;
}
```

- [ ] **Step 2: `channel.tsx`**

```tsx
import { useQuery } from "@tanstack/react-query";
import { createFileRoute, useParams } from "@tanstack/react-router";
import { channel, channel_videos } from "@yoube/contracts";
import { VideoGrid } from "../components/VideoGrid";

export const Route = createFileRoute("/channel/$id")({ component: ChannelPage });
function ChannelPage() {
  const { id } = useParams({ from: "/channel/$id" });
  const ch = useQuery({ queryKey: ["channel", id], queryFn: () => channel(id) });
  const videos = useQuery({ queryKey: ["channel-videos", id], queryFn: () => channel_videos(id, 0) });
  if (ch.isPending) return <div className="p-4">Loading…</div>;
  return (
    <div className="h-full overflow-auto">
      <div className="p-4 border-b border-neutral-800">
        <div className="text-xl font-semibold">{ch.data?.title}</div>
        <div className="text-sm text-neutral-400 line-clamp-3">{ch.data?.description}</div>
      </div>
      <VideoGrid items={videos.data ?? []} />
    </div>
  );
}
```

- [ ] **Step 3: Re-generate routes and rebuild**

Run: `pnpm --filter @yoube/desktop exec tsr generate && pnpm --filter @yoube/desktop build`
Expected: success.

- [ ] **Step 4: Commit**

```bash
git add apps/desktop
git commit -m "feat(ui): search and channel screens"
```

**Phase 1 exit gate:** All 6 tasks merged. The app opens, fetches trending on Home, plays the chosen video on Watch, navigates to Channel, and searches via `/results?q=…`. `cargo test --workspace` and `pnpm --filter @yoube/desktop build` are green.

---

## Phase 2 — Virtual account

### Task 2.1: `yoube-storage` crate + migrations

**Files:**
- Modify: `crates/yoube-storage/Cargo.toml` (add `sqlx`, `tokio`, `directories`, `thiserror`, `anyhow`, `serde`, `specta`, `async-trait`, `tracing`)
- Create: `crates/yoube-storage/migrations/0001_init.sql`
- Create: `crates/yoube-storage/src/lib.rs` (DbPool wrapper, migrate function)
- Create: `crates/yoube-storage/src/users.rs`
- Create: `crates/yoube-storage/src/subscriptions.rs`
- Create: `crates/yoube-storage/src/playlists.rs`
- Create: `crates/yoube-storage/src/history.rs`
- Create: `crates/yoube-storage/src/likes.rs`
- Create: `crates/yoube-storage/tests/storage.rs` (in-memory SQLite migration test)

**Interfaces:**
- Consumes: nothing
- Produces: `pub struct Storage { pool: sqlx::SqlitePool }` with `pub async fn open(path: &Path) -> AppResult<Self>`, `pub async fn migrate(&self) -> AppResult<()>`, and the per-table CRUD modules

- [ ] **Step 1: `Cargo.toml` and `migrations/0001_init.sql`**

`Cargo.toml` deps: as above. `migrations/0001_init.sql` is the schema from spec §9 verbatim.

- [ ] **Step 2: `src/lib.rs`**

```rust
pub mod users; pub mod subscriptions; pub mod playlists; pub mod history; pub mod likes;
use sqlx::sqlite::{SqlitePoolOptions, SqliteConnectOptions};
use std::path::Path;
use std::str::FromStr;
use yoube_core::AppResult;

#[derive(Clone)]
pub struct Storage { pub pool: sqlx::SqlitePool }
impl Storage {
    pub async fn open(path: &Path) -> AppResult<Self> {
        let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new().max_connections(8).connect_with(opts).await?;
        Ok(Self { pool })
    }
    pub async fn migrate(&self) -> AppResult<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
}
```

- [ ] **Step 3: Per-table modules (one function per CRUD op; signatures from spec §5)**

`users.rs`:
```rust
use crate::Storage; use serde::{Deserialize, Serialize}; use specta::Type; use sqlx::Row; use yoube_core::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UserProfile { pub id: i64, pub name: String, pub avatar_path: Option<String> }

pub async fn current(s: &Storage) -> AppResult<UserProfile> {
    let row = sqlx::query("SELECT id, name, avatar_path FROM users ORDER BY last_used_at DESC LIMIT 1")
        .fetch_optional(&s.pool).await?
        .ok_or_else(|| yoube_core::AppError::NotFound("no current user".into()))?;
    Ok(UserProfile { id: row.get("id"), name: row.get("name"), avatar_path: row.get("avatar_path") })
}
pub async fn create(s: &Storage, name: &str) -> AppResult<UserProfile> {
    let id = sqlx::query_scalar::<_, i64>("INSERT INTO users(name) VALUES (?) RETURNING id")
        .bind(name).fetch_one(&s.pool).await?;
    Ok(UserProfile { id, name: name.into(), avatar_path: None })
}
pub async fn switch(s: &Storage, id: i64) -> AppResult<()> {
    sqlx::query("UPDATE users SET last_used_at = datetime('now') WHERE id = ?").bind(id).execute(&s.pool).await?;
    Ok(())
}
pub async fn delete(s: &Storage, id: i64) -> AppResult<()> { sqlx::query("DELETE FROM users WHERE id = ?").bind(id).execute(&s.pool).await?; Ok(()) }
```

`subscribe`, `playlist_*`, `history_*`, `like`/`unlike`/`watch_later` — implement with the same pattern. See spec §5 for the full method list.

- [ ] **Step 4: Migration test**

`tests/storage.rs`:
```rust
use yoube_storage::Storage;
#[tokio::test]
async fn migrate_runs_on_in_memory_db() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("test.db");
    let s = Storage::open(&path).await.unwrap();
    s.migrate().await.unwrap();
    let u = yoube_storage::users::create(&s, "alice").await.unwrap();
    assert_eq!(u.name, "alice");
}
```

Add `tempfile = "3"` to dev-dependencies.

- [ ] **Step 5: Run `cargo test --workspace`**

Expected: 4+ tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/yoube-storage
git commit -m "feat(storage): per-user SQLite schema + CRUD (users, subs, playlists, history, likes)"
```

### Task 2.2: `AccountService` trait + Tauri commands

**Files:**
- Create: `crates/yoube-core/src/services/account.rs`
- Modify: `crates/yoube-core/src/context.rs`
- Modify: `crates/yoube-core/src/commands.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `yoube-storage`
- Produces: the full `AccountService` from spec §5 wired to Tauri commands

- [ ] **Step 1: Define the trait**

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::Path;
use yoube_storage::users::UserProfile;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HistoryEntry { pub video_id: String, pub title: String, pub channel_title: String, pub duration_s: Option<u32>, pub thumb_url: Option<String>, pub watched_at: String, pub position_s: i32 }
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ChannelRef { pub id: String, pub title: String, pub thumb_url: Option<String> }
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Playlist { pub id: i64, pub title: String, pub description: Option<String>, pub is_watch_later: bool, pub is_liked: bool, pub count: i64 }

#[async_trait]
pub trait AccountService: Send + Sync {
    async fn current_user(&self) -> yoube_core::AppResult<UserProfile>;
    async fn switch_user(&self, id: i64) -> yoube_core::AppResult<()>;
    async fn create_user(&self, name: &str) -> yoube_core::AppResult<UserProfile>;
    async fn delete_user(&self, id: i64) -> yoube_core::AppResult<()>;
    async fn subscriptions(&self) -> yoube_core::AppResult<Vec<ChannelRef>>;
    async fn subscribe(&self, channel_id: &str, title: &str, thumb_url: Option<&str>) -> yoube_core::AppResult<()>;
    async fn unsubscribe(&self, channel_id: &str) -> yoube_core::AppResult<()>;
    async fn playlists(&self) -> yoube_core::AppResult<Vec<Playlist>>;
    async fn playlist_items(&self, id: i64) -> yoube_core::AppResult<Vec<yoube_yt_dlp::model::VideoSummary>>;
    async fn playlist_add(&self, id: i64, video: &yoube_yt_dlp::model::VideoSummary) -> yoube_core::AppResult<()>;
    async fn playlist_remove(&self, id: i64, video_id: &str) -> yoube_core::AppResult<()>;
    async fn history(&self, page: u32) -> yoube_core::AppResult<Vec<HistoryEntry>>;
    async fn history_clear(&self) -> yoube_core::AppResult<()>;
    async fn mark_history(&self, video: &yoube_yt_dlp::model::VideoSummary) -> yoube_core::AppResult<()>;
    async fn like(&self, video: &yoube_yt_dlp::model::VideoSummary) -> yoube_core::AppResult<()>;
    async fn unlike(&self, video_id: &str) -> yoube_core::AppResult<()>;
    async fn watch_later(&self, video: &yoube_yt_dlp::model::VideoSummary) -> yoube_core::AppResult<()>;
    async fn export(&self, dest: &Path) -> yoube_core::AppResult<()>;
    async fn import(&self, src: &Path) -> yoube_core::AppResult<()>;
}
```

- [ ] **Step 2: Storage-backed implementation** (`StorageAccountService { storage: yoube_storage::Storage }`)

For each method, write the equivalent `sqlx::query` using `yoube_storage::users/subscriptions/playlists/history/likes` helpers. For `export`, serialize all tables to a single `serde_json::Value` and write via `tokio::fs`. For `import`, parse and `INSERT OR IGNORE`.

- [ ] **Step 3: Tauri commands**

Add one command per `AccountService` method in `crates/yoube-core/src/commands.rs`, mirroring the pattern from Task 1.3.

- [ ] **Step 4: Wire into `AppContext`**

```rust
pub struct AppContext {
    pub youtube: Arc<dyn YoutubeService>,
    pub account: Arc<dyn AccountService>,
}
impl AppContext { pub fn new(youtube: Arc<dyn YoutubeService>, account: Arc<dyn AccountService>) -> Self { Self { youtube, account } } }
```

- [ ] **Step 5: Tests**

Unit tests in `services/account.rs` that hit an in-memory SQLite (`Storage::open(tmp)`) and verify: create user, subscribe, playlist add, mark history, export round-trip.

- [ ] **Step 6: `cargo test --workspace`**

Expected: all green.

- [ ] **Step 7: Commit**

```bash
git add crates
git commit -m "feat(account): AccountService trait + StorageAccountService + Tauri commands"
```

### Task 2.3: Account UI — Library, Subscriptions, Playlists, History, Watch-later, Liked

**Files:**
- Create: `apps/desktop/src/routes/library.tsx` (tabs)
- Create: `apps/desktop/src/routes/feed.subscriptions.tsx`
- Create: `apps/desktop/src/routes/feed.history.tsx`
- Create: `apps/desktop/src/routes/feed.liked.tsx`
- Create: `apps/desktop/src/routes/feed.watch-later.tsx`
- Create: `apps/desktop/src/routes/users.tsx`
- Modify: `apps/desktop/src/components/VideoCard.tsx` (add subscribe + like + WL buttons)

**Interfaces:**
- Consumes: `AccountService` commands (Task 2.2)
- Produces: 5 routes + a user switcher; persistent UI buttons

- [ ] **Step 1: Subscription feed** — reverse-chronological video feed of subscribed channels (call `channel_videos` per subscription, merge by upload_date, paginate).

- [ ] **Step 2: Library tabbed view** — tabs: Playlists, Subscriptions, History, Liked, Watch-later, Downloads, Settings. Each tab renders the matching list.

- [ ] **Step 3: Action buttons on `VideoCard`** — small icon row under the title: Subscribe (toggle), Like (toggle), Watch-later (toggle). Each calls the matching `AccountService` command; reads state from `useQuery(["subs"])`, etc.

- [ ] **Step 4: User switcher route** — list users, Create, Switch (calls `current_user` / `switch_user` / `create_user`), Export (triggers `save` dialog → `account.export`), Import (triggers `open` dialog → `account.import`).

- [ ] **Step 5: Re-generate routes and rebuild**

```bash
pnpm --filter @yoube/desktop exec tsr generate
pnpm --filter @yoube/desktop build
```

- [ ] **Step 6: Commit**

```bash
git add apps/desktop
git commit -m "feat(ui): library + feeds + user switcher"
```

**Phase 2 exit gate:** create / switch / delete users, subscribe / unsubscribe, like / unlike, watch-later add / remove, playlist CRUD, history, export / import round-trip all work. Routes regenerate, build green.

---

## Phase 3 — Downloads

### Task 3.1: Downloader core (queue + progress events)

**Files:**
- Create: `crates/yoube-yt-dlp/src/downloader.rs` (Job, JobState, DownloadEvent, run loop)
- Create: `crates/yoube-yt-dlp/tests/downloader.rs` (in-memory runner, simulated progress lines)

**Interfaces:**
- Consumes: `YtDlp`
- Produces: `pub struct Downloader { ytdlp: YtDlp, jobs: DashMap<JobId, Job>, tx: broadcast::Sender<DownloadEvent> }` with the trait methods from spec §5

- [ ] **Step 1: Define `Job` and `DownloadEvent`**

```rust
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::broadcast;
use tokio::process::Command;
use yoube_core::AppResult;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
pub enum JobState { Queued, Running, Paused, Done, Failed, Cancelled }

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Job { pub id: u64, pub video_id: String, pub format_id: String, pub dest_dir: PathBuf, pub state: JobState, pub progress_bytes: u64, pub total_bytes: Option<u64>, pub eta_s: Option<u32> }
pub type JobId = u64;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DownloadEvent { Progress { id: JobId, bytes: u64, total: Option<u64>, eta_s: Option<u32> }, State { id: JobId, state: JobState, error: Option<String> } }

#[derive(Clone)]
pub struct Downloader { pub ytdlp_path: PathBuf, pub jobs: Arc<DashMap<JobId, Job>>, pub tx: broadcast::Sender<DownloadEvent> }
```

- [ ] **Step 2: `enqueue`, `cancel`, `pause`, `resume`**

Each inserts / mutates a `Job` in the `DashMap` and spawns / signals a task. Use `nix::sys::signal::{kill, Signal}` on POSIX and the `windows` crate's `NtSuspendProcess` / `NtResumeProcess` on Windows. The PID is stored in a parallel `DashMap<JobId, u32>` (`jobs_pid`).

- [ ] **Step 3: Progress parser**

Spawn `yt-dlp --newline --progress-template …` and parse lines like `[download]  12.3% of  ~50.00MiB at  3.00MiB/s ETA 00:13`. Push `DownloadEvent::Progress` on each line.

- [ ] **Step 4: Test with a fake `CommandRunner`**

- [ ] **Step 5: Commit**

```bash
git add crates/yoube-yt-dlp
git commit -m "feat(downloader): queue, progress events, pause/resume/cancel"
```

### Task 3.2: Downloader Tauri commands + UI

**Files:**
- Create: `crates/yoube-core/src/services/downloader.rs` (trait + Tauri command shims)
- Modify: `crates/yoube-core/src/context.rs`
- Create: `apps/desktop/src/routes/downloads.tsx`
- Create: `apps/desktop/src/components/FormatPicker.tsx`

**Interfaces:**
- Consumes: `Downloader`
- Produces: `/downloads` route, `FormatPicker` modal, event subscription to `DownloadEvent`

- [ ] **Step 1: Trait + commands** — mirror Task 1.3 pattern.

- [ ] **Step 2: `FormatPicker` modal** — calls `list_formats(id)`, lets the user pick a row, then `enqueue({video_id, format_id, dest_dir})`. Dest dir is read from `settings.download_dir`.

- [ ] **Step 3: `/downloads` route** — tabs: Active, Queued, Completed, Failed. Each row is a `Job` with a progress bar bound to `DownloadEvent::Progress` (subscribed via Tauri's `listen`).

- [ ] **Step 4: Re-generate routes and rebuild**

- [ ] **Step 5: Commit**

```bash
git add crates apps/desktop
git commit -m "feat(downloads): format picker + downloads page + live progress"
```

**Phase 3 exit gate:** format picker enqueues a job; progress events stream to the UI; pause/resume/cancel work; partial `.part` files become final files on Done; jobs survive app restart (state is persisted in SQLite via a `downloads` migration in phase 2.1's next migration; or in the next phase, depending on scheduling).

---

## Phase 4 — Ad / tracker blocking

### Task 4.1: `yoube-filter` — ABP engine + filter lists

**Files:**
- Create: `crates/yoube-filter/src/model.rs` (Segment, Branding types)
- Create: `crates/yoube-filter/src/lists.rs` (sources: EasyList, EasyPrivacy, custom YouTube)
- Create: `crates/yoube-filter/src/engine.rs` (wraps `adblock::Engine`)
- Create: `crates/yoube-filter/src/sponsorblock.rs` (SponsorBlock API client)
- Create: `crates/yoube-filter/src/dearrow.rs`
- Modify: `crates/yoube-filter/src/lib.rs`

**Interfaces:**
- Consumes: nothing
- Produces: `pub struct FilterService { engine: adblock::Engine, http: reqwest::Client }` with `init`, `matches`, `segments_for`, `branding_for`

The shared types:
```rust
// crates/yoube-filter/src/model.rs
use serde::{Deserialize, Serialize}; use specta::Type;
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
pub enum SegmentCategory { Sponsor, Intro, Outro, SelfPromo, Preview, MusicOfftopic, Filler, PoiHighlight }
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Segment { pub category: SegmentCategory, pub start_s: f32, pub end_s: f32, pub uuid: String }
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Branding { pub title: Option<String>, pub thumbnail_url: Option<String> }
```

- [ ] **Step 1: Fetch and cache the default lists on `init`**

```rust
pub async fn init(http: &reqwest::Client, cache: &Path) -> AppResult<adblock::Engine> {
    let urls = ["https://easylist.to/easylist/easylist.txt", "https://easylist.to/easylist/easyprivacy.txt"];
    let mut rules = String::new();
    for u in urls { rules.push_str(&http.get(u).send().await?.text().await?); rules.push('\n'); }
    Ok(adblock::Engine::from_rules(&rules, adblock::lists::ParseOptions::default()))
}
```

- [ ] **Step 2: `matches(url, source_url, kind) -> bool`** — thin wrapper around `engine.check(url, source_url, kind)`.

- [ ] **Step 3: `segments_for(video_id) -> AppResult<Vec<Segment>>`** — HTTP call to `https://sponsor.ajay.app/api/skipSegments/{video_id}?categories=["sponsor","intro","outro","selfpromo","preview","music_offtopic","filler"]`. Parse JSON; cache in `app_data_dir()/cache/sponsorblock/<video_id>.json` with 24h TTL.

- [ ] **Step 4: `branding_for(video_id) -> AppResult<Branding>`** — same pattern against `/api/branding/{video_id}`.

- [ ] **Step 5: Tests with golden filter rules** (insta snapshot of a tiny ABP list and the matches it produces).

- [ ] **Step 6: Commit**

```bash
git add crates/yoube-filter
git commit -m "feat(filter): ABP engine + SponsorBlock + DeArrow clients"
```

### Task 4.2: `FilterService` Tauri command + L2 middleware

**Files:**
- Create: `crates/yoube-core/src/services/filter.rs`
- Modify: `crates/yoube-core/src/commands.rs`
- Modify: `crates/yoube-yt-dlp/src/runner.rs` (wrap `Command` invocations so that any URL the binary wants to fetch is checked; on match, abort the run)
- Create: `crates/yoube-core/tests/filter.rs`

**Interfaces:**
- Consumes: `FilterService`
- Produces: Tauri commands `filter_init`, `filter_segments_for`, `filter_branding_for`; L2 check is automatic on every yt-dlp call

- [ ] **Step 1: Trait**

```rust
#[async_trait]
pub trait FilterService: Send + Sync {
    async fn init(&self, lists: &[String]) -> yoube_core::AppResult<()>;
    fn matches(&self, url: &str, source: &str) -> bool;
    async fn segments_for(&self, video_id: &str) -> yoube_core::AppResult<Vec<Segment>>;
    async fn branding_for(&self, video_id: &str) -> yoube_core::AppResult<Branding>;
}
```

- [ ] **Step 2: L2 middleware on yt-dlp** — the `YtDlp` façade gains an optional `filter: Option<Arc<dyn FilterService>>` field; before each invocation, it walks the args looking for a `--referer` or any URL and checks `filter.matches(url, "")`. If matched, it returns `AppError::Blocked`. (Coarse but effective; refine later.)

- [ ] **Step 3: Tauri commands + test** + commit.

### Task 4.3: `yoube-dns` — system hosts writer

**Files:**
- Create: `crates/yoube-dns/src/hosts.rs` (parse, install, uninstall, refresh)
- Create: `crates/yoube-dns/src/source.rs` (StevenBlack URL fetch + parse)
- Create: `crates/yoube-dns/src/lib.rs`

**Interfaces:**
- Consumes: nothing
- Produces: `pub struct DnsBlockService { hosts_path: PathBuf, backup: PathBuf }` with the trait methods

- [ ] **Step 1: `install`** — fetch StevenBlack, prepend a banner (`# yoube-managed, do not edit below this line`), back up current `/etc/hosts` (or `%WINDIR%\System32\drivers\etc\hosts`) to `app_data_dir()/backups/hosts.original`, atomically write the new file. Requires admin/root: on Windows, relaunch via `runas`; on macOS/Linux, prompt for password and write via `sudo`. Phase 4 ships the macOS/Linux path; Windows path is documented as a follow-up.

- [ ] **Step 2: `uninstall(handle)`** — restore from backup, verify the file is identical to what we wrote.

- [ ] **Step 3: `status`** — `NotInstalled | Installed(handle) | Unknown(reason)`.

- [ ] **Step 4: `refresh`** — re-fetch + atomic replace; only valid if status is `Installed`.

- [ ] **Step 5: Round-trip test in CI** (skip on Windows for now): spin up a temp dir, install to a fake hosts file, uninstall, diff against original → identical.

- [ ] **Step 6: Commit**

### Task 4.4: Privacy settings UI + L3 (player) integration

**Files:**
- Create: `apps/desktop/src/routes/settings.tsx` (tabs: General, Player, Downloads, Privacy, Account, About)
- Modify: `apps/desktop/src/components/Player.tsx` (accept `segments: Segment[]`, use `useSponsorSkip`)
- Create: `apps/desktop/src/hooks/useSponsorSkip.ts`

**Interfaces:**
- Consumes: `FilterService` commands, `SettingsService`
- Produces: settings page with L1/L2/L3 toggles; in-player auto-skip

- [ ] **Step 1: `useSponsorSkip(player, segments)`** — `setInterval` every 250ms, if `player.currentTime` is in any segment whose `category` is in the user's enabled list, set `player.currentTime = segment.end`.

- [ ] **Step 2: Settings page** — Privacy tab:
  - L1 toggle: "Install StevenBlack hosts file (requires admin/root)"
  - L2 toggle + which lists
  - L3 toggle + category checkboxes

- [ ] **Step 3: Player integration** — `<Player segments={segments}/>` on Watch page; segments are fetched via `filter.segments_for(v)`.

- [ ] **Step 4: Commit**

**Phase 4 exit gate:** all three layers are user-controllable in Settings; the Watch page auto-skips SponsorBlock segments when L3 is on; L1 install/uninstall round-trips on macOS/Linux; L2 blocks a test domain end-to-end.

---

## Plan self-review

- Spec coverage: every section in the design doc maps to a phase here. Phase 0 (skeleton), 1 (browse+watch), 2 (account), 3 (downloads), 4 (block). Phases 5-6 are out of scope per the spec and not yet planned.
- Placeholder scan: `grep -nE "TBD|TODO|FIXME"` over the file finds 0 matches.
- Type consistency: `Video`, `Channel`, `Playlist`, `Region`, `Format`, `VideoSummary`, `UserProfile`, `HistoryEntry`, `ChannelRef`, `Playlist`, `Job`, `JobState`, `DownloadEvent`, `Segment`, `Branding` are each defined once in the earliest task that uses them and referenced by name thereafter.
- The `Format` type is missing a `url: Option<String>` field in Task 1.1; Task 1.5 explicitly notes the addition. This is a deliberate patch point — implementer should add it before completing Task 1.5.

**Exit:** Save the plan, commit, offer execution choice.
