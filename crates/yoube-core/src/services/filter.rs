//! `FilterService`: L2 ABP matching + L3 SponsorBlock/DeArrow metadata.
//!
//! Thin async façade over `yoube-filter`. The ABP engine itself is sync
//! (and blocks its worker thread briefly), so the `matches` check stays
//! sync here too — Tauri commands that call it from async context wrap it
//! in `tokio::task::spawn_blocking`. `segments_for` / `branding_for` are
//! natively async (reqwest + file cache, fail-open).

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use yoube_filter::{Branding, FilterEngine, Segment, SegmentCategory};

use crate::AppResult;

/// Content-filter service (spec §5 `FilterService`, adapted).
#[async_trait]
pub trait FilterService: Send + Sync {
    /// Fetch remote ABP lists into the engine (offline-tolerant).
    /// Returns the number of rules now loaded.
    async fn init(&self) -> AppResult<usize>;
    /// Synchronous ABP verdict: `true` = block this request.
    fn matches(&self, url: &str, source_url: &str) -> bool;
    /// SponsorBlock skip segments for a video (cached, fail-open).
    async fn segments_for(&self, video_id: &str) -> AppResult<Vec<Segment>>;
    /// DeArrow crowd-sourced branding (cached, fail-open).
    async fn branding_for(&self, video_id: &str) -> AppResult<Branding>;
}

/// Production implementation backed by `yoube-filter`.
pub struct AppFilterService {
    engine: FilterEngine,
    http: reqwest::Client,
    cache_dir: PathBuf,
}

impl AppFilterService {
    /// Build with the built-in rules; call `init()` to fetch remotes.
    pub fn new(cache_dir: PathBuf) -> AppResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("yoube/0.0.1")
            .build()?;
        Ok(Self {
            engine: FilterEngine::boot(),
            http,
            cache_dir,
        })
    }

    fn segments_dir(&self) -> PathBuf {
        self.cache_dir.join("sponsorblock")
    }

    fn branding_dir(&self) -> PathBuf {
        self.cache_dir.join("dearrow")
    }

    fn lists_dir(&self) -> PathBuf {
        self.cache_dir.join("filter-lists")
    }
}

#[async_trait]
impl FilterService for AppFilterService {
    async fn init(&self) -> AppResult<usize> {
        self.engine.init(&self.http, &self.lists_dir()).await
    }

    fn matches(&self, url: &str, source_url: &str) -> bool {
        self.engine.matches(url, source_url, "other")
    }

    async fn segments_for(&self, video_id: &str) -> AppResult<Vec<Segment>> {
        yoube_filter::sponsorblock::segments_for(
            &self.http,
            &self.segments_dir(),
            video_id,
            SegmentCategory::defaults(),
        )
        .await
    }

    async fn branding_for(&self, video_id: &str) -> AppResult<Branding> {
        yoube_filter::dearrow::branding_for(&self.http, &self.branding_dir(), video_id).await
    }
}

impl yoube_yt_dlp::UrlBlocker for AppFilterService {
    fn is_blocked(&self, url: &str) -> bool {
        self.matches(url, "")
    }
}

/// Shareable handle type used when wiring the L2 gate into `YtDlp`.
pub type SharedFilterService = Arc<AppFilterService>;

#[cfg(test)]
mod tests {
    use super::*;

    fn svc() -> AppFilterService {
        let dir = std::env::temp_dir().join("yoube-filter-svc-test");
        AppFilterService::new(dir).expect("service builds offline")
    }

    #[test]
    fn builtin_rules_block_trackers() {
        let s = svc();
        // The worker loads builtins on boot; allow a beat for the channel.
        std::thread::sleep(Duration::from_millis(300));
        assert!(s.matches("https://doubleclick.net/ads/x", "https://www.youtube.com/"));
        assert!(!s.matches(
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
            "https://www.youtube.com/"
        ));
    }

    #[test]
    fn url_blocker_adapter_agrees_with_matches() {
        use yoube_yt_dlp::UrlBlocker;
        let s = svc();
        std::thread::sleep(Duration::from_millis(300));
        assert!(s.is_blocked("https://googleadservices.com/pagead/x"));
        assert!(!s.is_blocked("https://www.youtube.com/feed/trending"));
    }

    #[tokio::test]
    async fn blocked_url_short_circuits_ytdlp_without_spawning() {
        use std::sync::Arc;
        use yoube_yt_dlp::YtDlp;
        use yoube_yt_dlp::runner::CommandRunner;

        struct PanicRunner;
        #[async_trait]
        impl CommandRunner for PanicRunner {
            async fn output(
                &self,
                _bin: &std::path::Path,
                _args: &[&str],
            ) -> AppResult<std::process::Output> {
                panic!("runner must not be called for blocked URLs")
            }
        }

        let s = svc();
        std::thread::sleep(Duration::from_millis(300));
        let ytdlp = YtDlp::new(
            PathBuf::from("yt-dlp"),
            Arc::new(PanicRunner) as Arc<dyn CommandRunner>,
        )
        .with_blocker(Arc::new(s));
        let err = ytdlp
            .dump_json("https://doubleclick.net/ads/x")
            .await
            .expect_err("blocked URL must error");
        assert!(matches!(err, crate::AppError::Blocked(_)));
    }
}
