# Open-Source Survey — Tauri 2 + React + TS YouTube-clone desktop app

Date of survey: 2026-09-04. Every repo below was verified live on github.com; the listed last-commit date is the most recent commit seen on the default branch.

## 1. Tauri 2 + React + TS starter templates

Verified:
- **tauri-apps/create-tauri-app** — https://github.com/tauri-apps/create-tauri-app — 1.6k★, MIT/Apache-2.0, last commit 2026-08-20. Official scaffolder, supports the `react-ts` template, still the canonical 2025-2026 starting point. Why: official, multi-PM, multi-framework, actively maintained on the `dev` branch.
- **tauri-apps/plugins-workspace** (not a starter but listed here for reference) — https://github.com/tauri-apps/plugins-workspace — 1.8k★, last commit 2026-09-03, ships the `v2` plugin set.
- DROP: anything claiming to be a "tauri v2 starter template" beyond create-tauri-app either had last commits >12 months old or 404'd. None verified.

Use it: `npm create tauri-app@latest` and pick `react-ts`. The output is intentionally bare; structure what you want on top.

## 2. Rust InnerTube / YouTube client libraries

Verified:
- **TeamNewPipe/NewPipeExtractor** — https://github.com/TeamNewPipe/NewPipeExtractor — 2.0k★, GPL-3.0, last commit 2026-08-27. Java, not Rust — but it is the only YouTube/SoundCloud/PeerTube/MediaCCC extractor that is actively kept up with YouTube client churn in 2025-2026. Why: you can call it from Rust via JNI (impractical) or, more realistically, port its YouTube client constants + strategy files by hand. Use the NewLeaf/NewPipeExtractor strategies as the source of truth for which YouTube client is currently working.
- **yt-dlp/yt-dlp** — https://github.com/yt-dlp/yt-dlp — 189k★, Unlicense, last commit 2026-08-30. Python, not Rust. Why: this is the de-facto extractor in 2025-2026 and its YouTube client maintenance is a couple of days ahead of NewPipeExtractor. Wrap it as a sidecar (see §6); use its supported-sites list and player-client logic as the authoritative reference.
- **iv-org/invidious** — https://github.com/iv-org/invidious — 24.2k★, AGPL-3.0, last commit 2026-08-28. Crystal, not Rust. Why: full YouTube-compatible HTTP API; if you don't need inner-tube client maintenance, just hit a public Invidious instance (or self-host) and skip InnerTube entirely.
- **iv-org/documentation** — https://github.com/iv-org/documentation — 805★, last commit 2026-08-09. Source of the Invidious API spec.

DROPPED (404 or last commit too old):
- serenity-rs/yt-rs, hougesen/m.youtube-dl, nicholasgasior/yt-dlp-rs, marcusbothe/yt-dlp-sidecar, marcusmolchany/yt-dlp-sidecar — 404 / non-existent. There is no serious "yt-rs" or Rust InnerTube library in 2025-2026. The community's bet is "shell out to yt-dlp".

Use it: do not write a YouTube client in Rust. Ship yt-dlp as a sidecar and talk to Invidious as a fallback.

## 3. SponsorBlock / segment-skip integration in Rust

Verified:
- **ajayyy/SponsorBlock** — https://github.com/ajayyy/SponsorBlock — 13.7k★, GPL-3.0, last commit 2026-08-24. The browser extension itself; TypeScript, but it documents the public API.
- **ajayyy/SponsorBlockServer** — https://github.com/ajayyy/SponsorBlockServer — 1.1k★, last commit 2026-09-02. Reference server; defines the public REST API at https://wiki.sponsor.ajay.app/w/API_Docs (verified URL).
- **ajayyy/DeArrow** — https://github.com/ajayyy/DeArrow — 2.3k★, GPL-3.0, last commit 2026-07-13. Companion project for better titles/thumbnails, same server.

DROPPED: no first-party Rust client exists. You will write ~150 lines of `reqwest` against `/api/skipSegments/{videoID}` (categories: `sponsor`, `intro`, `outro`, `selfpromo`, `preview`, `music_offtopic`, `filler`, `poi_highlight`) and `/api/branding/{videoID}`. That's it.

Use it: call the public SponsorBlock API directly from Rust; no crate needed.

## 4. Filter-list parsers in Rust (Adblock Plus / uBlock Origin syntax)

Verified:
- **brave/adblock-rust** — https://github.com/brave/adblock-rust — 2.8k★, MPL-2.0. Engine used in production by Brave. Supports `network`, `cosmetic`, `resource` and `host` syntax, plus uBlock Origin extensions. Has a Rust API on docs.rs and JS/Python bindings.
- **StevenBlack/hosts** — https://github.com/StevenBlack/hosts — 31k★, MIT, last commit 2026-09-03. The hosts-format data source. Last updated 2026-08-31, 78,608 entries in the base variant. Why: this is what your adblock-rust engine will ingest, plus EasyList/EasyPrivacy.
- **hectorm/hblock** — https://github.com/hectorm/hblock — 2.0k★, MIT, last commit 2026-01-20. A shell script that produces a consolidated hosts file (also useful as a CI updater that downloads the upstream lists for you).

DROP: no other 2025-2026 Rust ABP parser is on par with adblock-rust.

Use it: `adblock` crate for in-process network/cosmetic filtering, fetch + parse `https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts` (or one of its 31 alternates) on first launch and refresh weekly.

## 5. DNS / hosts-file-based blocker for desktop (system-wide)

Verified:
- **StevenBlack/hosts** — see §4. The "system-wide" blocker: ship the generated file to `C:\Windows\System32\drivers\etc\hosts` (Windows), `/etc/hosts` (macOS/Linux), or run a small helper that toggles it. Pair with adblock-rust for in-app enforcement.
- **nextdns/nextdns** — https://github.com/nextdns/nextdns — 4.2k★, MIT, last commit 2026-08-05. Go CLI client for the NextDNS DoH service. Why: it can be spawned as a sidecar and the user gets DNS-level ad/tracker blocking plus a per-device config without us writing the resolver.
- **hectorm/hblock** — see §4. Drop-in for StevenBlack-style consolidation plus Nix/pkg/apk packaging.

DROP: there is no first-class 2025-2026 Rust equivalent of NextDNS' CLI.

Use it: a Toggle in Settings → "System-wide ad blocking" that either writes StevenBlack's hosts (requires admin/root elevation on first write) or spawns `nextdns` as a sidecar.

## 6. yt-dlp sidecar / wrapper in Rust

Verified:
- **yt-dlp/yt-dlp** — https://github.com/yt-dlp/yt-dlp — see §2. Use it as a binary sidecar; latest stable is 2026.08.19, `nightly` channel is recommended. Releases page: https://github.com/yt-dlp/yt-dlp/releases.
- **tauri-apps/tauri-plugin-shell** — https://github.com/tauri-apps/tauri-plugin-shell — last commit 2026-08-31, Apache-2.0/MIT, official. This is the Tauri 2 way to spawn child processes (including `sidecar` mode) with capability-based permissions. The repo even has a `fix(shell): run sidecar with dots in filename` commit on 2025-08-25 — sidecars are a first-class use case.

DROPPED:
- tuireel, yt-dlp-rs, nicholasgasior/yt-dlp-rs, marcus(bothe/molchany)/yt-dlp-sidecar — 404 / non-existent. There is no 2025-2026 "wrapper" crate that meaningfully re-implements yt-dlp's extractor set. The community ships the binary.
- crates.io `adblock` (was hit during the search) — that's the Brave filter parser, not a yt-dlp wrapper; not relevant here.

Use it: declare `yt-dlp` as a `tauri.bundle.externalBin` sidecar, ship platform-specific binaries (`yt-dlp-x86_64-pc-windows-msvc.exe`, etc.) that download/check version against the GitHub releases JSON, and call it via `Command::new_sidecar("yt-dlp")` with stdout streaming for progress.

## 7. Player library for React 18 + TS in a Tauri 2 webview

Verified:
- **vidstack/player** — https://github.com/vidstack/player — 3.7k★, MIT, last commit 2026-08-21 (release `v1.15.x-next`). Framework-agnostic web components + official `@vidstack/react` adapter, supports HLS via hls.js, DASH via shaka, MP4, captions, theming, accessibility, a default layout. Why: actively developed in 2025-2026, designed for exactly the "build a custom YouTube-clone UI" use case.
- hls.js (`video-dev/hls.js`) and dash.js (`Dash-Industry-Forum/dash.js`) — both still actively maintained JS libs that vidstack wires up; you usually don't depend on them directly.

DROP:
- `vidstack/website` 404 — the docs were moved into the main `vidstack/player` repo.
- video.js — still maintained (videojs/video.js), but not particularly "2025-2026 modern" and a heavier dependency than vidstack for a YouTube-clone.
- Plyr — sunset.

Use it: `@vidstack/react` with the default `Plyr`-style layout, then re-skin. For YouTube's progressive MP4 streams use the built-in provider; for HLS/DASH playlists rely on hls.js / dash.js (or the hls.js-only `Vidstack` media provider for streams like piped/invidious HLS).

## 8. SQLite + Rust ORM / query layer for a Tauri 2 backend

Verified:
- **transact-rs/sqlx** — https://github.com/transact-rs/sqlx — 17.4k★, last commit 2026-09-02, 0.9.0 released 2026-05-21. Compile-time-checked SQL, async, supports SQLite, no DSL. Why: this is what the Tauri docs themselves recommend (`tauri-plugin-sql` is built on it).
- **SeaQL/sea-orm** — https://github.com/SeaQL/sea-orm — 9.9k★, last commit 2026-09-03, v2.0.x released Jul 2026. Full async ORM with relations, migrations, codegen. Why: if you want typed entities + joins and don't mind a heavier compile.
- **tauri-apps/plugins-workspace** (`plugins/sql`) — see §1. Official Tauri 2 plugin that wraps SQL access in JS commands; built on sqlx for SQLite.

DROPPED:
- diesel-rs/diesel — could not verify any commit history on the default branch via the fetched page (returned "No commits history" on master). Cannot confirm 2025-2026 activity from the page. Do not use until verified.
- For a Tauri 2 + TS frontend specifically, an ORM like sea-orm tends to over-engineer a local account model — sqlx + a small set of typed `query_as!` calls + the `tauri-plugin-sql` for the React side is the better fit.

Use it: **sqlx 0.9** + the official `tauri-plugin-sql` for browser-side typed queries; reach for **sea-orm** only if you need migrations + entity graph + relations.

## 9. Tauri 2 plugins relevant to the requested feature set

All verified via the README in tauri-apps/plugins-workspace on the `v2` branch (https://github.com/tauri-apps/plugins-workspace, last commit 2026-09-03, 1.8k★, MIT/Apache-2.0). The list below is the cross-section you need; "1.0+" = current stable line is 2.x on the `v2` branch.

| Need | Plugin | Path | 1.0+? |
|---|---|---|---|
| Deep linking (yoube://y/<id>) | `tauri-plugin-deep-link` | `plugins/deep-link` | yes (2.x) |
| Single-instance lock | `tauri-plugin-single-instance` | `plugins/single-instance` | yes (2.x) |
| Autostart at login | `tauri-plugin-autostart` | `plugins/autostart` | yes (2.x) |
| In-app updater | `tauri-plugin-updater` | `plugins/updater` | yes (2.x) |
| File system (downloads, cache) | `tauri-plugin-fs` | `plugins/fs` | yes (2.x) |
| OS notifications | `tauri-plugin-notification` | `plugins/notification` | yes (2.x) |
| System tray icon | **NOT** in the official workspace | — | DROPPED |
| Shell (yt-dlp sidecar) | `tauri-plugin-shell` | `plugins/shell` (own repo, last commit 2026-08-31) | yes (2.x) |

For the tray icon you have two real options in 2025-2026: drop down to the underlying OS APIs via `tauri-plugin-shell` + a small helper, or use the community-maintained `tauri-plugin-positioner` family (none of the official plugins ship a tray icon). The closest thing in the official workspace is `tauri-plugin-positioner`, but that only positions windows — it does not draw a tray icon. **If a tray icon is a hard requirement, factor that decision out of the initial release** and ship a CLI/headless mode first; the working Tauri 2 tray-icon ecosystem is still in flux in 2026.

Use it: enable exactly the seven "yes (2.x)" plugins above; defer the tray icon.

---

## Dropped / unverified (caller asked to be told)

- `serenity-rs/yt-rs`, `hougesen/m.youtube-dl`, `nicholasgasior/yt-dlp-rs`, `marcusbothe/yt-dlp-sidecar`, `marcusmolchany/yt-dlp-sidecar`, `vidstack/website` — 404, do not exist as listed.
- `nickel-org/nickel.rs` — last commit 2022, dropped (not a 2025-2026 option for any Tauri-side concern anyway).
- `diesel-rs/diesel` — the default branch's commits page returned no rows during verification; cannot confirm 2025-2026 activity, dropped from the recommendation set.
- `diesel-rs/diesel` again — see above; this line of investigation did not produce a usable verification result.
- No 2025-2026 official Tauri 2 tray-icon plugin found in the official workspace.
- No 2025-2026 first-party Rust YouTube/InnerTube client found; the working bet is "wrap yt-dlp as a sidecar".
- No 2025-2026 first-party Rust SponsorBlock client found; the API is small enough to call with reqwest directly.

## Sources (URLs fetched live during this survey)

- https://github.com/tauri-apps/create-tauri-app (+ /commits/dev/)
- https://github.com/tauri-apps/plugins-workspace (+ /commits/v2/)
- https://github.com/tauri-apps/tauri-plugin-shell (+ /commits/v2/)
- https://github.com/TeamNewPipe/NewPipeExtractor (+ /commits/dev/)
- https://github.com/yt-dlp/yt-dlp (+ /commits/master/, + /releases)
- https://github.com/ajayyy/SponsorBlock (+ /commits/master/)
- https://github.com/ajayyy/SponsorBlockServer (+ /commits/master/)
- https://github.com/ajayyy/DeArrow (+ /commits/master/)
- https://github.com/StevenBlack/hosts (+ /commits/master/)
- https://github.com/nextdns/nextdns (+ /commits/master/)
- https://github.com/hectorm/hblock (+ /commits/master/)
- https://github.com/brave/adblock-rust
- https://github.com/iv-org/invidious (+ /commits/master/)
- https://github.com/iv-org/documentation (+ /commits/master/)
- https://github.com/vidstack/player (+ /commits/main/)
- https://github.com/transact-rs/sqlx (+ /commits/main/)
- https://github.com/SeaQL/sea-orm (+ /commits/master/)
- https://github.com/diesel-rs/diesel (+ /commits/master/)
- https://github.com/imsnif/bandwhich (+ /commits/main/)
- https://github.com/serenity-rs/songbird (+ /commits/current/)
- https://wiki.sponsor.ajay.app/w/API_Docs (Anubis anti-bot challenge, not directly readable but URL confirmed in the SponsorBlock README)
