# yoube — Design Spec

**Date:** 2026-09-04
**Status:** Approved by user (verbal, design phase). Awaiting written-spec sign-off before implementation plan.
**Owner:** youssef

## 1. Pitch

A cross-platform desktop application that gives the user a YouTube-like experience without an account, without tracking, and without ads. It uses `yt-dlp` as a sidecar to fetch all YouTube data, plays videos with `@vidstack/react`, blocks ads and trackers in three layers, downloads media in any format `yt-dlp` supports, and stores a local virtual account (subscriptions, playlists, history, likes, watch-later) in per-user SQLite.

## 2. Goals & non-goals

### Goals
- Look and feel like YouTube (familiar information architecture: Home / Search / Watch / Channel / Subscriptions / Library)
- Never require a user account; all state is local and exportable
- Block YouTube/Google ads and trackers at three layers (DNS, in-app network filter, in-player segment skip)
- Download any video in any format `yt-dlp` supports, with pause/resume/cancel
- Be easy to maintain, extend, and rebuild a year from now
- Ship desktop builds for Linux, macOS, Windows from one codebase; mobile is a future port

### Non-goals (out of scope for this spec)
- Uploading videos, creating a YouTube account, or any kind of publishing
- Cloud sync between devices (export/import only)
- Live-stream chat (live playback only)
- Mobile builds in phase 0-5
- A web-hosted version of the app
- DRM-protected content (Netflix, etc.)

## 3. Architecture (5 layers)

| Layer | Tech | Role |
|---|---|---|
| 1. Shell | Tauri 2 | Window mgmt, single-instance, deep links `yoube://y/<id>`, autostart, updater, fs/notification/shell plugins |
| 2. UI | React 19 + TS + Vite + Tailwind + shadcn/ui + TanStack Router + TanStack Virtual | All screens, all states, all interactions |
| 3. Domain | TypeScript types generated from Rust via `tauri-specta` | Shared view-models, no `any` across the FFI boundary |
| 4. Services | Rust crate `yoube-core` (one binary, many `pub trait`s) | `youtube`, `downloader`, `filter`, `dnsblock`, `account`, `media`, `settings` |
| 5. Data | SQLite (per-user DB) + filesystem | One DB file, one media dir, portable backups |

## 4. Repository layout (Cargo + pnpm monorepo)

```
yoube/
  apps/
    desktop/                 # the Tauri 2 app (UI + shell)
      src/                   # React 19 app
      src-tauri/             # Tauri 2 Rust wrapper
  crates/
    yoube-core/              # the Rust services crate
    yoube-yt-dlp/            # thin wrapper around the yt-dlp CLI
    yoube-filter/            # adblock-rust integration + ABP hosts parser
    yoube-dns/               # system hosts writer (admin/root elevation)
    yoube-storage/           # sqlx + migrations + account domain
  packages/
    ui-tokens/               # design tokens shared with the renderer
    contracts/               # generated TS bindings from Rust (specta output)
  sidecars/
    yt-dlp/                  # vendored binaries per OS/arch
    ffmpeg/                  # vendored binaries per OS/arch
  docs/
    superpowers/specs/
  .research/                 # OSS surveys + future research notes
  Cargo.toml                 # workspace root
  pnpm-workspace.yaml
  justfile                   # dev / build / test / fmt / lint tasks
```

## 5. The 7 services (Rust traits)

```rust
// 1. youtube — read-only data via yt-dlp --dump-json
#[async_trait]
pub trait YoutubeService: Send + Sync {
    async fn get_video(&self, id: &str) -> Result<Video>;
    async fn search(&self, q: &str, page: u32) -> Result<Vec<Video>>;
    async fn channel(&self, id: &str) -> Result<Channel>;
    async fn channel_videos(&self, id: &str, page: u32) -> Result<Vec<Video>>;
    async fn playlist(&self, id: &str) -> Result<Playlist>;
    async fn trending(&self, region: Region) -> Result<Vec<Video>>;
    async fn related(&self, id: &str) -> Result<Vec<Video>>;
}

// 2. downloader — yt-dlp as a subprocess with progress events
#[async_trait]
pub trait DownloaderService: Send + Sync {
    async fn list_formats(&self, id: &str) -> Result<Vec<Format>>;
    fn enqueue(&self, req: DownloadRequest) -> Result<JobId>;
    fn cancel(&self, id: JobId) -> Result<()>;
    fn pause(&self, id: JobId) -> Result<()>;
    fn resume(&self, id: JobId) -> Result<()>;
    fn subscribe(&self) -> broadcast::Receiver<DownloadEvent>;
}

// 3. filter — adblock + sponsor segments
pub trait FilterService: Send + Sync {
    fn init(&self, lists: &[FilterListSource]) -> Result<()>;
    fn matches(&self, url: &str, source_url: &str, kind: RequestKind) -> bool;
    fn segments_for(&self, video_id: &str) -> Result<Vec<Segment>>;
    fn branding_for(&self, video_id: &str) -> Result<Branding>;
}

// 4. dnsblock — system-wide hosts writer
pub trait DnsBlockService: Send + Sync {
    fn install(&self, lists: &[HostsSource]) -> Result<HostsHandle>;
    fn uninstall(&self, handle: HostsHandle) -> Result<()>;
    fn status(&self) -> Result<HostsStatus>;
    fn refresh(&self) -> Result<()>;
}

// 5. account — SQLite-backed virtual user
#[async_trait]
pub trait AccountService: Send + Sync {
    async fn current_user(&self) -> Result<UserProfile>;
    async fn switch_user(&self, id: i64) -> Result<()>;
    async fn create_user(&self, name: &str) -> Result<UserProfile>;
    async fn delete_user(&self, id: i64) -> Result<()>;
    async fn subscriptions(&self) -> Result<Vec<Channel>>;
    async fn subscribe(&self, channel_id: &str) -> Result<()>;
    async fn unsubscribe(&self, channel_id: &str) -> Result<()>;
    async fn playlists(&self) -> Result<Vec<Playlist>>;
    async fn playlist_items(&self, id: i64) -> Result<Vec<Video>>;
    async fn playlist_add(&self, id: i64, video_id: &str) -> Result<()>;
    async fn playlist_remove(&self, id: i64, video_id: &str) -> Result<()>;
    async fn history(&self, page: u32) -> Result<Vec<HistoryEntry>>;
    async fn history_clear(&self) -> Result<()>;
    async fn like(&self, video_id: &str) -> Result<()>;
    async fn unlike(&self, video_id: &str) -> Result<()>;
    async fn watch_later(&self, video_id: &str) -> Result<()>;
    async fn export(&self, dest: &Path) -> Result<()>;
    async fn import(&self, src: &Path) -> Result<()>;
}

// 6. media — player config + thumbnails/captions
pub trait MediaService: Send + Sync {
    fn cache_thumbnail(&self, url: &str) -> Result<PathBuf>;
    fn download_captions(&self, video_id: &str, lang: &str) -> Result<PathBuf>;
    fn player_config(&self) -> PlayerConfig;
}

// 7. settings — versioned JSON, never SQLite
pub trait SettingsService: Send + Sync {
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>;
    fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<()>;
    fn migrate(&self, from_version: u32) -> Result<()>;
}
```

**Rule:** services do not call each other. They communicate via the `AppContext` struct held in Tauri state. This is what makes the boundary clean and testable.

## 6. Data flow (watch-page example)

1. User clicks a video in `HomeFeed.tsx`.
2. TanStack Router navigates to `/watch?v=<id>` and mounts `<WatchPage/>`.
3. `<WatchPage/>` calls `contracts.youtube.getVideo(id)` (a Tauri command).
4. `yoube-core::youtube::get_video` invokes `yt-dlp --dump-json --skip-download <url>`, parses stdout to a typed `Video` model, returns it.
5. In parallel, `yoube-core::filter::segments_for(id)` calls `https://sponsor.ajay.app/api/skipSegments/<id>?categories=[…]`, returns `Vec<Segment>`.
6. `<WatchPage/>` calls `contracts.account.markHistory(id)` to log the view.
7. `<VideoPlayer/>` (vidstack) gets the chosen stream URL (already exposed by `get_video` in the `formats` array) and starts playback.
8. During playback, a small `useSponsorSkip` hook watches `currentTime` and seeks past any `Segment` whose start ≤ t < end.

## 7. Ad-block + tracker-block (3 layers)

| Layer | What it does | Where it runs | Source of truth |
|---|---|---|---|
| **L1 — DNS / hosts** | System-wide: blocks the app + any other app from reaching Google's telemetry/ads domains | OS `hosts` file, written by us with admin/root elevation | `StevenBlack/hosts` (consolidated: hosts + ads + malware + fakenews + gambling) + optional custom list |
| **L2 — ABP filter** | In-app: every `reqwest` call is checked against ABP rules; non-conforming requests are dropped | Rust `FilterService`, every HTTP call wrapped in a `FilterMiddleware` | `brave/adblock-rust`, lists: `EasyList`, `EasyPrivacy`, `YouTube-specific` (a small custom list we curate in `crates/yoube-filter/lists/`) |
| **L3 — Player / segment** | In the player: skip SponsorBlock segments, swap to DeArrow titles/thumbs | React `<VideoPlayer/>` + `useSponsorSkip` hook | `sponsor.ajay.app` API, no auth required |

User controls in Settings → Privacy:
- L1 on/off (writes/backs up `/etc/hosts`)
- L2 on/off + which lists
- L3 on/off + which categories to skip
- L3 has a "report missing segment" button that opens the SponsorBlock web submitter

**Why this order:** L1 protects the OS, L2 protects the app, L3 fixes the player. Disable any one safely.

## 8. Download engine

**Format selection:** user picks a profile (`Best`, `1080p`, `Audio-only (mp3)`, `Custom`). The Rust side maps that to the right `yt-dlp -f` token. The `Format` type the UI shows comes from `downloader.list_formats(id)` which is `yt-dlp -F`.

**Queue & progress:**
- `Job { id, video_id, format_id, dest_dir, state, progress_bytes, progress_eta }` lives in an in-memory `DashMap<JobId, Job>`
- One `tokio` task per concurrent download, configurable cap (default 2)
- `yt-dlp --newline --progress-template …` is parsed line-by-line; we push `DownloadEvent::Progress { … }` to a `tokio::sync::broadcast` channel
- Pause = `SIGSTOP` (POSIX) / `NtSuspendProcess` (Windows via `windows` crate); resume = `SIGCONT` / `NtResumeProcess`. Cancel = kill + remove partial file
- Atomic write: yt-dlp writes to `*.part`, we rename to final name on success

**Sidecar mechanics:**
- `sidecars/yt-dlp/` and `sidecars/ffmpeg/` have binaries for each target triple (linux, darwin, windows) per arch
- Tauri's `externalBin` in `tauri.conf.json` declares them; capability `shell:allow-execute` whitelists only the sidecar path
- On first launch, we check the bundled version against `https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest` and prompt the user to update (auto-update is opt-in, not default)
- ffmpeg is bundled the same way (needed for audio extraction + format remux)

## 9. Data model (SQLite + filesystem)

**Locations:**
- `app_data_dir()/yoube.db` — one SQLite file per machine (per-user on Windows/macOS, per-user on Linux)
- `app_data_dir()/media/` — downloaded media, organized as `media/<video_id>/<safe_title>.<ext>`
- `app_data_dir()/cache/thumbs/` — LRU-cached thumbnails
- `app_data_dir()/settings.json` — versioned user settings
- `app_data_dir()/filter-lists/` — downloaded ABP / hosts lists (refreshed weekly)
- `app_data_dir()/yt-dlp/` — user-installed (overrides bundled) yt-dlp binary, if any

**Schema (sqlx migrate, one file per migration, never edited once committed):**

```sql
-- 0001_init.sql
CREATE TABLE users (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  name         TEXT NOT NULL,
  avatar_path  TEXT,
  created_at   TEXT NOT NULL DEFAULT (datetime('now')),
  last_used_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE subscriptions (
  user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  channel_id  TEXT    NOT NULL,
  title       TEXT    NOT NULL,
  thumb_url   TEXT,
  added_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  PRIMARY KEY (user_id, channel_id)
);
CREATE TABLE playlists (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  title       TEXT    NOT NULL,
  description TEXT,
  is_watch_later INTEGER NOT NULL DEFAULT 0,
  is_liked     INTEGER NOT NULL DEFAULT 0,
  created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
  updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE playlist_items (
  playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
  position    INTEGER NOT NULL,
  video_id    TEXT    NOT NULL,
  title       TEXT    NOT NULL,
  channel_id  TEXT,
  channel_title TEXT,
  duration_s  INTEGER,
  thumb_url   TEXT,
  added_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  PRIMARY KEY (playlist_id, position)
);
CREATE TABLE history (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  video_id    TEXT    NOT NULL,
  title       TEXT    NOT NULL,
  channel_id  TEXT,
  channel_title TEXT,
  duration_s  INTEGER,
  thumb_url   TEXT,
  watched_at  TEXT    NOT NULL DEFAULT (datetime('now')),
  position_s  INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_history_user_time ON history(user_id, watched_at DESC);
CREATE TABLE likes (
  user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  video_id    TEXT    NOT NULL,
  title       TEXT    NOT NULL,
  channel_id  TEXT,
  channel_title TEXT,
  duration_s  INTEGER,
  thumb_url   TEXT,
  liked_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  PRIMARY KEY (user_id, video_id)
);
CREATE VIRTUAL TABLE search_index USING fts5(
  video_id, title, channel_title, kind UNINDEXED,
  tokenize = 'unicode61 remove_diacritics'
);
```

**Conventions:**
- All timestamps are ISO-8601 UTC strings (`datetime('now')` returns that).
- All YouTube IDs are stored as `TEXT` (they're strings, sometimes 11 chars, sometimes longer).
- The two special "playlists" (watch_later, liked) live in `playlists` with `is_watch_later=1` / `is_liked=1`; the user can rename or delete them like any normal playlist.

**Export / import format:**
A single `yoube-export-YYYY-MM-DD.json` containing `{ version, users, subscriptions, playlists, playlist_items, history, likes }`. Import is idempotent (skip if PK collides). Round-trip tested.

## 10. Settings (versioned JSON)

- Single file: `app_data_dir()/settings.json`
- Shape: `{ "version": 1, "values": { "theme": "system", "default_region": "US", "download_dir": "...", "sponsor_skip": true, "...": "..." } }`
- Every read calls `migrate(from_version)` if file's `version` is older than the running code expects
- A migration is a free function `fn migrate_v0_to_v1(v: serde_json::Value) -> serde_json::Value`
- Adds a new key = migration; renames a key = migration; removes a key = migration (just drop it)
- We never store settings in SQLite. Settings is for behavior; data is for content.

## 11. UI surface

| Route | Screen | Notes |
|---|---|---|
| `/` | Home | Curated shelves: trending, subscriptions, recommended, news |
| `/results?q=` | Search | Search results grid, filter by upload date / type / duration / features |
| `/watch?v=` | Watch | Player + metadata + comments + recommended + sponsor-skip indicator |
| `/channel/<id>` | Channel | Banner, avatar, tabs: Home / Videos / Shorts / Playlists / About |
| `/playlist?list=` | Playlist | Same as Channel → Playlists |
| `/feed/subscriptions` | Subscriptions | Reverse-chronological feed of new uploads from subscribed channels |
| `/feed/history` | History | Paginated, clearable |
| `/feed/liked` | Liked videos | Backed by the `is_liked=1` playlist |
| `/feed/watch_later` | Watch later | Backed by the `is_watch_later=1` playlist |
| `/library` | Library | All playlists + downloads + settings shortcut |
| `/downloads` | Downloads | Queue, completed, settings |
| `/settings` | Settings | Tabs: General, Player, Downloads, Privacy (ad-block controls), Account, About |
| `/users` | User switcher | List, create, switch, delete, export/import |

State management: **Zustand** for global UI state (current user, theme, modals), **TanStack Query** for server-cache-style data (videos, search results) backed by Tauri command calls. No Redux.

## 12. Testing strategy

| Layer | Strategy | Tool |
|---|---|---|
| Rust services | Unit + integration, in-memory mocks for sibling services | `cargo test`, `mockall` |
| Rust ↔ yt-dlp | Recorded fixtures (yt-dlp stdout JSON) replayed by a fake `CommandRunner` | `insta` snapshots |
| Filter lists | Golden-file tests of ABP rule → match/no-match | `insta` |
| Hosts writer | Round-trip test: install → uninstall → diff against original `/etc/hosts` | shell + `tempfile` |
| DB migrations | `sqlx migrate run` against a fresh DB on every CI run | GitHub Actions matrix: Linux + macOS + Windows |
| TS domain | TS types must match Rust types byte-for-byte | `tsc --noEmit` + specta output diff |
| UI | Component tests with RTL + Playwright E2E on a stubbed Tauri shell | Vitest, Playwright |
| E2E | Spin up the built Tauri app, drive it with Playwright, stub yt-dlp responses | Playwright + Tauri WebDriver |

CI runs on every PR: lint (`cargo clippy`, `eslint`, `biome`), typecheck, test, build artifact for all 3 OSes. Coverage target: ≥70% on `crates/`, ≥60% on UI components.

## 13. Build & distribution

- `pnpm tauri build` produces platform installers (MSI/NSIS on Windows, DEB/AppImage on Linux, DMG on macOS)
- `justfile` wraps common commands: `dev`, `build`, `test`, `fmt`, `lint`, `update-sidecars` (fetches latest yt-dlp + ffmpeg), `prepare-release` (bump + tag + sign + publish draft)
- Code signing: macOS via Apple Developer ID, Windows via Azure Trusted Signing, Linux via GPG + signed AppImage
- Sidecar update flow: at startup, the app checks `app_data_dir()/yt-dlp/yt-dlp --version` (if user installed one) vs bundled; uses the newer; surface a "yt-dlp is out of date, click to update" banner
- Feature flags in `Cargo.toml`:
  - `default = ["download", "sponsorblock", "adblock", "dnsblock"]`
  - `--no-default-features` for a read-only build (no downloads, no blocking)
  - Distros that require it can ship with `--no-default-features --features adblock` etc.

## 14. Roadmap (phased delivery)

| Phase | Deliverable | What "done" means |
|---|---|---|
| **0. Skeleton** | Tauri 2 + React 19 + TS monorepo, CI green, "Hello video" on Home | App launches, can fetch one video via yt-dlp sidecar, plays in vidstack |
| **1. Browse + Watch** | Home / Search / Watch / Channel / Trending / Region | All routes wired, search + watch fully functional, history recorded |
| **2. Virtual account** | Subscriptions, Playlists, Watch later, Likes, History, Library, multi-user, export/import | All `AccountService` methods work, FTS5 search works, export round-trips |
| **3. Downloads** | Format picker, queue, progress, pause/resume/cancel, file mgmt, post-update yt-dlp prompt | All `DownloaderService` methods work, downloads survive app restart |
| **4. Ad/tracker block** | ABP lists, hosts writer, SponsorBlock skip, DeArrow thumbs, settings UI | All `FilterService` + `DnsBlockService` methods work, settings persist |
| **5. Polish** | Tray icon (third-party crate, since Tauri 2 has no official one), media keys, mini-player, picture-in-picture, captcha handling, crash reporter | Every UI affordance feels native |
| **6. Mobile** | Tauri Mobile (or Capacitor if perf issues) port — only after desktop is stable | Same UI runs on Android/iOS |

Each phase ends in a tagged release. Phases 1-4 are the "vertical slice" + first growth. Phase 5+ is feature work.

## 15. Growth & maintenance rules (the "easy to grow" promise)

1. New service = new file in `crates/yoube-core/src/services/`, one trait, one impl, one mock.
2. New Tauri command = one line in `crates/yoube-core/src/commands.rs` that resolves the trait and calls the method.
3. New TS screen = new file in `apps/desktop/src/routes/`, consumes only `packages/contracts/`.
4. New DB column = new migration, never edit old ones.
5. New OSS dependency = research in `.research/`, then add to the relevant crate.
6. The "Definition of Done" for any PR: lint + typecheck + tests + a green build artifact for all 3 OSes.

## 16. Risks & mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| YouTube changes InnerTube schema; `yt-dlp` falls behind | Medium | High | We ship `yt-dlp` as a sidecar and check for updates on launch; in-app banner prompts user to update |
| SponsorBlock API rate limits | Low | Medium | Cache results in SQLite (24h TTL); fall back to "no skip" on 429 |
| OS hosts file gets clobbered by another tool | Medium | Medium | On install, back up the original to `app_data_dir()/backups/hosts.original`; on uninstall, restore from backup |
| DMCA / legal challenge to a redistribution | Low | High | License file clearly states we don't host content; downloader is a feature flag; `LICENSE-THIRD-PARTY.md` lists yt-dlp Unlicense, ffmpeg LGPL, SponsorBlock GPL |
| Tauri 2 has no first-party tray icon | High | Low | Phase 5 uses a community crate (e.g. `tauri-plugin-tray-icon` if it ships; otherwise a 200-line shim using `tray-icon` Rust crate) |
| yt-dlp sidecar binary size (~95MB) | High | Low | Document it; consider an opt-in "download on first launch" for the `nightly` channel |

## 17. Component provenance (verified 2026-09-04, see `.research/oss-survey.md`)

| Need | Picked | Verified | License |
|---|---|---|---|
| Tauri 2 + React 19 + TS scaffold | `tauri-apps/create-tauri-app` (`react-ts`) | last commit 2026-08-20 | MIT/Apache-2.0 |
| YouTube data | `yt-dlp/yt-dlp` as sidecar | last commit 2026-08-30 | Unlicense |
| Format / progress | `tauri-plugin-shell` (sidecar mode) | last commit 2026-08-31 | MIT/Apache-2.0 |
| Player | `vidstack/player` + `@vidstack/react` | last commit 2026-08-21 | MIT |
| ABP filter engine | `brave/adblock-rust` | active 2025-2026 | MPL-2.0 |
| Hosts source | `StevenBlack/hosts` | last commit 2026-09-03 | MIT |
| SponsorBlock / DeArrow | public `sponsor.ajay.app` API | docs wiki live | API call, no crate |
| Local DB | `transact-rs/sqlx` (0.9) | last commit 2026-09-02 | MIT/Apache-2.0 |
| Tauri 2 plugins | `tauri-apps/plugins-workspace` (v2) | last commit 2026-09-03 | MIT/Apache-2.0 |
| State (UI) | Zustand | mainline React-friendly | MIT |
| Server cache (UI) | TanStack Query | mainline | MIT |
| Routing | TanStack Router | mainline | MIT |
| List virtualization | TanStack Virtual | mainline | MIT |
| UI components | shadcn/ui + Tailwind | mainline | MIT |

## 18. Open questions (to resolve in phase 0-1)

- App icon + name: the design doc uses "yoube" as a placeholder. Real branding can be decided when the UI skeleton is up.
- Captcha handling: "Sign in to confirm you're not a bot" is rare on InnerTube but possible. We surface a clear error + the failing video ID and let the user decide.
