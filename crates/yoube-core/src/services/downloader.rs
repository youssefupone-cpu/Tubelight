//! `DownloaderService`: the Tauri-facing façade over `yoube-yt-dlp::Downloader`.
//!
//! Lives in `yoube-core` (no `tauri` dependency — the `#[tauri::command]`
//! shims are in `apps/desktop/src-tauri/src/commands.rs`).
//!
//! Design note: progress streaming to the UI is done by polling
//! `list_jobs()` from React Query (`refetchInterval: 1000`). The underlying
//! `Downloader` also exposes a `broadcast::Receiver<DownloadEvent>` via
//! `subscribe()` for future push-based UI (Tauri `emit`) without changing
//! this trait.

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use yoube_yt_dlp::YtDlp;
use yoube_yt_dlp::downloader::{Downloader, Job, JobId};
use yoube_yt_dlp::model::Format;

use crate::AppResult;

/// Download queue service (spec §5 `DownloaderService`, adapted).
#[async_trait]
pub trait DownloaderService: Send + Sync {
    /// List available formats for a video (`yt-dlp --dump-single-json`).
    async fn list_formats(&self, video_id: &str) -> AppResult<Vec<Format>>;
    /// Queue a download; returns the assigned job id.
    async fn enqueue(&self, video_id: &str, format_id: &str, dest_dir: &Path) -> AppResult<JobId>;
    /// Pause a running job (SIGSTOP on POSIX).
    async fn pause(&self, id: JobId) -> AppResult<()>;
    /// Resume a paused job (SIGCONT on POSIX).
    async fn resume(&self, id: JobId) -> AppResult<()>;
    /// Cancel a job; removes its `.part` file.
    async fn cancel(&self, id: JobId) -> AppResult<()>;
    /// Snapshot of all known jobs (queued → done), for UI polling.
    async fn list_jobs(&self) -> AppResult<Vec<Job>>;
}

/// `YtDlp`-backed implementation.
pub struct YtDlpDownloaderService {
    ytdlp: YtDlp,
    downloader: Arc<Downloader>,
}

impl YtDlpDownloaderService {
    /// Build from an existing `YtDlp` handle.
    pub fn new(ytdlp: YtDlp) -> Self {
        let downloader = Arc::new(Downloader::new(ytdlp.clone()));
        Self { ytdlp, downloader }
    }

    /// Borrow the inner queue (for future Tauri `emit` wiring / tests).
    pub fn inner(&self) -> &Arc<Downloader> {
        &self.downloader
    }
}

#[async_trait]
impl DownloaderService for YtDlpDownloaderService {
    async fn list_formats(&self, video_id: &str) -> AppResult<Vec<Format>> {
        let url = format!("https://www.youtube.com/watch?v={video_id}");
        self.ytdlp.list_formats(&url).await
    }

    async fn enqueue(&self, video_id: &str, format_id: &str, dest_dir: &Path) -> AppResult<JobId> {
        // Ensure the destination exists before yt-dlp writes `*.part` there.
        // `create_dir_all` on an existing dir is a no-op.
        if !dest_dir.as_os_str().is_empty() {
            let _ = std::fs::create_dir_all(dest_dir);
        }
        let dest = if dest_dir.as_os_str().is_empty() {
            PathBuf::from(".")
        } else {
            dest_dir.to_path_buf()
        };
        self.downloader.enqueue(video_id, format_id, &dest)
    }

    async fn pause(&self, id: JobId) -> AppResult<()> {
        self.downloader.pause(id)
    }

    async fn resume(&self, id: JobId) -> AppResult<()> {
        self.downloader.resume(id)?;
        self.downloader.try_next();
        Ok(())
    }

    async fn cancel(&self, id: JobId) -> AppResult<()> {
        self.downloader.cancel(id)?;
        self.downloader.try_next();
        Ok(())
    }

    async fn list_jobs(&self) -> AppResult<Vec<Job>> {
        Ok(self.downloader.list())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yoube_yt_dlp::runner::{CommandRunner, Spawned};

    struct FailingRunner;
    #[async_trait]
    impl CommandRunner for FailingRunner {
        async fn output(&self, _bin: &Path, _args: &[&str]) -> AppResult<std::process::Output> {
            Ok(std::process::Output {
                status: std::process::ExitStatus::default(),
                stdout: b"{}".to_vec(),
                stderr: vec![],
            })
        }

        async fn spawn_streaming(&self, _bin: &Path, _args: &[&str]) -> AppResult<Spawned> {
            Err(crate::AppError::Internal(anyhow::anyhow!(
                "no binary in test"
            )))
        }
    }

    #[tokio::test]
    async fn enqueue_with_failing_runner_marks_job_failed() {
        let runner: Arc<dyn CommandRunner> = Arc::new(FailingRunner);
        let ytdlp = YtDlp::new(PathBuf::from("yt-dlp"), runner);
        let svc = YtDlpDownloaderService::new(ytdlp);
        let tmp = std::env::temp_dir();
        let id = svc.enqueue("vid1", "best", &tmp).await.unwrap();
        // The worker fails fast (spawn_streaming errors); poll until Failed.
        let mut failed = false;
        for _ in 0..50 {
            let jobs = svc.list_jobs().await.unwrap();
            if jobs
                .iter()
                .any(|j| j.id == id && j.state == yoube_yt_dlp::downloader::JobState::Failed)
            {
                failed = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        assert!(failed, "job should reach Failed when the runner errors");
    }

    #[tokio::test]
    async fn pause_resume_cancel_unknown_job_is_noop_ok() {
        let runner: Arc<dyn CommandRunner> = Arc::new(FailingRunner);
        let ytdlp = YtDlp::new(PathBuf::from("yt-dlp"), runner);
        let svc = YtDlpDownloaderService::new(ytdlp);
        // Unknown ids touch no shard entries but must not error (no pid).
        svc.pause(999_999).await.unwrap();
        svc.resume(999_999).await.unwrap();
        svc.cancel(999_999).await.unwrap();
    }
}
