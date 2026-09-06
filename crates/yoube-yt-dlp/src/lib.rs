//! yoube-yt-dlp: thin wrapper around the yt-dlp binary.

pub mod downloader;
pub mod model;
pub mod parser;
pub mod runner;

use crate::model::{Format, VideoSummary};
use crate::parser::{parse_dump_json, parse_list_formats};
use crate::runner::CommandRunner;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use yoube_error::{AppError, AppResult};

#[derive(Clone)]
pub struct YtDlp {
    pub bin: PathBuf,
    pub runner: Arc<dyn CommandRunner>,
    blocker: Option<Arc<dyn UrlBlocker>>,
}

/// L2 content gate (spec §7): implemented by `yoube-core::FilterService` via
/// an adapter. Lives here (not in `yoube-core`) so the crate graph stays a
/// DAG — `yoube-yt-dlp` never depends on `yoube-core`.
pub trait UrlBlocker: Send + Sync {
    /// Return `true` if `url` must not be fetched.
    fn is_blocked(&self, url: &str) -> bool;
}

impl YtDlp {
    pub fn new(bin: PathBuf, runner: Arc<dyn CommandRunner>) -> Self {
        Self {
            bin,
            runner,
            blocker: None,
        }
    }

    /// Attach an L2 blocker; chaining-friendly (consumes + returns `Self`).
    pub fn with_blocker(mut self, blocker: Arc<dyn UrlBlocker>) -> Self {
        self.blocker = Some(blocker);
        self
    }

    pub(crate) fn check_blocked(&self, url: &str) -> AppResult<()> {
        if let Some(b) = &self.blocker
            && b.is_blocked(url)
        {
            return Err(AppError::Blocked(url.to_string()));
        }
        Ok(())
    }
    pub async fn dump_json_full(&self, url: &str) -> AppResult<Value> {
        self.check_blocked(url)?;
        let args = [
            "--skip-download",
            "--dump-single-json",
            "--no-warnings",
            url,
        ];
        let out = self.runner.output(&self.bin, &args).await?;
        if !out.status.success() {
            return Err(AppError::YtDlp(
                String::from_utf8_lossy(&out.stderr).into_owned(),
            ));
        }
        let v: Value = serde_json::from_slice(&out.stdout)?;
        Ok(v)
    }
    pub async fn dump_json(&self, url: &str) -> AppResult<VideoSummary> {
        let v = self.dump_json_full(url).await?;
        parse_dump_json(&v)
    }
    pub async fn list_formats(&self, url: &str) -> AppResult<Vec<Format>> {
        let v = self.dump_json_full(url).await?;
        parse_list_formats(&v)
    }
}
