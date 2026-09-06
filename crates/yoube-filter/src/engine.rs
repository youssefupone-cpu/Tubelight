//! ABP matching engine (`adblock` crate) behind a Send+Sync actor.
//!
//! Why an actor: `adblock::Engine` was `!Send + !Sync` in the 0.9 line the
//! plan pinned (internal regex manager uses thread-local state), and its
//! 0.9 dependency tree no longer compiles against fresh transitive deps
//! (`rmp-serde` 0.15 vs `rmp` 0.8 drift). The workspace therefore tracks
//! `adblock` 0.13, whose `BlockerResult::should_block()` is the single
//! blocking verdict. The actor is kept regardless: it pins all matching to
//! one thread and keeps `matches()` a plain sync call usable from any
//! context (Tauri commands wrap it in `spawn_blocking`).
//!
//! Fail-open everywhere: unknown URLs, oversized inputs, and worker timeouts
//! return `false` (allow) rather than breaking playback.

use std::path::{Path, PathBuf};
use std::time::Duration;

use yoube_error::{AppError, AppResult};

use crate::lists::{DEFAULT_LIST_URLS, builtin_youtube_rules, rule_lines};

const WORKER_REPLY_TIMEOUT: Duration = Duration::from_millis(500);
const MAX_RULE_BYTES: usize = 32 * 1024 * 1024;

enum EngineCmd {
    LoadRules(Vec<String>),
    Check {
        url: String,
        source_url: String,
        request_type: String,
        reply: std::sync::mpsc::Sender<bool>,
    },
}

/// Send+Sync handle to the ABP worker thread.
pub struct FilterEngine {
    tx: std::sync::mpsc::Sender<EngineCmd>,
}

impl FilterEngine {
    /// Boot the worker with only the built-in rules (offline-safe).
    pub fn boot() -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<EngineCmd>();
        std::thread::Builder::new()
            .name("yoube-filter".into())
            .spawn(move || worker_loop(rx))
            .expect("filter worker thread spawns");
        let engine = Self { tx };
        engine.load_rules(rule_lines(builtin_youtube_rules()));
        engine
    }

    /// Build from an explicit rule set (tests / `init` after fetch).
    pub fn from_rules(rules: Vec<String>) -> Self {
        let engine = Self::boot();
        engine.load_rules(rules);
        engine
    }

    fn load_rules(&self, rules: Vec<String>) {
        let _ = self.tx.send(EngineCmd::LoadRules(rules));
    }

    /// Replace the rule set (called by `init` after fetching lists).
    pub fn replace_rules(&self, rules: Vec<String>) {
        self.load_rules(rules);
    }

    /// Check whether a request should be blocked.
    ///
    /// `request_type` is an adblock request type (`"script"`, `"image"`,
    /// `"xmlhttprequest"`, …); anything unknown falls back to `"other"`.
    /// Never panics, never blocks longer than `WORKER_REPLY_TIMEOUT`.
    pub fn matches(&self, url: &str, source_url: &str, request_type: &str) -> bool {
        if url.len() > 8192 || source_url.len() > 8192 {
            return false;
        }
        let (reply_tx, reply_rx) = std::sync::mpsc::channel::<bool>();
        let cmd = EngineCmd::Check {
            url: url.to_string(),
            source_url: source_url.to_string(),
            request_type: normalize_request_type(request_type).to_string(),
            reply: reply_tx,
        };
        if self.tx.send(cmd).is_err() {
            return false;
        }
        // Sync API: block the calling thread (with timeout) for the worker's
        // verdict. Callers on an async runtime must wrap `matches` in
        // `spawn_blocking`; the worker itself never touches the runtime.
        reply_rx.recv_timeout(WORKER_REPLY_TIMEOUT).unwrap_or(false)
    }

    /// Fetch the default ABP lists, cache them under `cache_dir`, and load
    /// them into the worker. Offline-tolerant: on any network error the
    /// built-in rules stay active and `Ok(0)` is returned.
    ///
    /// Returns the number of rules loaded.
    pub async fn init(&self, http: &reqwest::Client, cache_dir: &Path) -> AppResult<usize> {
        let mut combined = String::from(builtin_youtube_rules());
        let mut fetched_any = false;
        for url in DEFAULT_LIST_URLS {
            match fetch_list(http, url).await {
                Ok(text) => {
                    fetched_any = true;
                    combined.push('\n');
                    combined.push_str(&text);
                    let _ = write_cache(cache_dir, url, &text).await;
                }
                Err(_) => {
                    // Try the on-disk cache before giving up on this list.
                    if let Some(cached) = read_cache(cache_dir, url).await {
                        fetched_any = true;
                        combined.push('\n');
                        combined.push_str(&cached);
                    }
                }
            }
            if combined.len() > MAX_RULE_BYTES {
                break;
            }
        }
        let rules = rule_lines(&combined);
        let n = rules.len();
        self.replace_rules(rules);
        if fetched_any {
            Ok(n)
        } else {
            Ok(rule_lines(builtin_youtube_rules()).len())
        }
    }
}

fn normalize_request_type(t: &str) -> &str {
    match t {
        "script" | "image" | "stylesheet" | "object" | "xmlhttprequest" | "subdocument"
        | "document" | "websocket" | "media" | "font" | "other" | "ping" | "csp_report" => t,
        "xhr" => "xmlhttprequest",
        "css" => "stylesheet",
        "img" => "image",
        "js" => "script",
        _ => "other",
    }
}

fn worker_loop(rx: std::sync::mpsc::Receiver<EngineCmd>) {
    let mut engine = adblock::Engine::new_with_list_text("||doubleclick.net^");
    for cmd in rx {
        match cmd {
            EngineCmd::LoadRules(rules) => {
                engine = adblock::Engine::new_with_list_text(rules.join("\n"));
            }
            EngineCmd::Check {
                url,
                source_url,
                request_type,
                reply,
            } => {
                let blocked = check_one(&engine, &url, &source_url, &request_type);
                let _ = reply.send(blocked);
            }
        }
    }
}

fn check_one(engine: &adblock::Engine, url: &str, source_url: &str, request_type: &str) -> bool {
    let req = match adblock::request::Request::new(url, source_url, request_type, "get") {
        Ok(r) => r,
        Err(_) => return false,
    };
    engine.check_network_request(&req).should_block()
}

async fn fetch_list(http: &reqwest::Client, url: &str) -> AppResult<String> {
    let resp = http.get(url).send().await?;
    let status = resp.status();
    if !status.is_success() {
        return Err(AppError::Internal(anyhow::anyhow!(
            "filter list fetch failed: {url} -> {status}"
        )));
    }
    Ok(resp.text().await?)
}

fn cache_file_for(cache_dir: &Path, url: &str) -> PathBuf {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in url.bytes() {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    cache_dir.join(format!("list-{hash:016x}.txt"))
}

async fn write_cache(cache_dir: &Path, url: &str, text: &str) {
    let _ = tokio::fs::create_dir_all(cache_dir).await;
    let path = cache_file_for(cache_dir, url);
    // Best-effort: cache failures must not fail `init`.
    let _ = tokio::fs::write(&path, text).await;
}

async fn read_cache(cache_dir: &Path, url: &str) -> Option<String> {
    let path = cache_file_for(cache_dir, url);
    tokio::fs::read_to_string(&path).await.ok()
}

/// Request kinds the UI / middleware can pass to `matches`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestKind {
    Script,
    Image,
    Media,
    Xhr,
    Document,
    Other,
}

impl RequestKind {
    pub fn as_str(self) -> &'static str {
        match self {
            RequestKind::Script => "script",
            RequestKind::Image => "image",
            RequestKind::Media => "media",
            RequestKind::Xhr => "xmlhttprequest",
            RequestKind::Document => "document",
            RequestKind::Other => "other",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_blocks_doubleclick() {
        let engine = FilterEngine::from_rules(rule_lines(builtin_youtube_rules()));
        // Give the worker a moment to load (channel send is async).
        std::thread::sleep(Duration::from_millis(200));
        assert!(engine.matches(
            "https://doubleclick.net/ads/x",
            "https://www.youtube.com/watch?v=abc",
            "script"
        ));
    }

    #[test]
    fn exception_allows_playback_stats() {
        let engine = FilterEngine::from_rules(rule_lines(builtin_youtube_rules()));
        std::thread::sleep(Duration::from_millis(200));
        assert!(!engine.matches(
            "https://www.youtube.com/api/stats/playback?x=1",
            "https://www.youtube.com/watch?v=abc",
            "xmlhttprequest"
        ));
    }

    #[test]
    fn benign_url_is_allowed() {
        let engine = FilterEngine::from_rules(rule_lines(builtin_youtube_rules()));
        std::thread::sleep(Duration::from_millis(200));
        assert!(!engine.matches(
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
            "https://www.youtube.com/",
            "document"
        ));
    }

    #[test]
    fn garbage_input_fails_open() {
        let engine = FilterEngine::boot();
        assert!(!engine.matches("::::", "::::", "script"));
        assert!(!engine.matches("", "", "other"));
    }
}
