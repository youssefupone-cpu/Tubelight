//! yoube-yt-dlp: thin wrapper around the yt-dlp binary.

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
}

impl YtDlp {
    pub fn new(bin: PathBuf, runner: Arc<dyn CommandRunner>) -> Self {
        Self { bin, runner }
    }
    pub async fn dump_json_full(&self, url: &str) -> AppResult<Value> {
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
