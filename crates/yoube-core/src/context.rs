use std::sync::Arc;

use crate::error::AppResult;
use crate::services::account::AccountService;
use crate::services::downloader::DownloaderService;
use crate::services::youtube::YoutubeService;

pub struct AppContext {
    pub youtube: Arc<dyn YoutubeService>,
    pub account: Arc<dyn AccountService>,
    pub downloader: Arc<dyn DownloaderService>,
}

impl AppContext {
    pub fn new(
        youtube: Arc<dyn YoutubeService>,
        account: Arc<dyn AccountService>,
        downloader: Arc<dyn DownloaderService>,
    ) -> Self {
        Self {
            youtube,
            account,
            downloader,
        }
    }
    pub fn ping(&self) -> AppResult<&'static str> {
        Ok("pong")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::youtube::{Channel, Playlist, Region, Video};

    struct NoopYoutubeService;
    #[async_trait::async_trait]
    impl YoutubeService for NoopYoutubeService {
        async fn get_video(&self, _id: &str) -> AppResult<Video> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn search(
            &self,
            _q: &str,
            _page: u32,
        ) -> AppResult<Vec<yoube_yt_dlp::model::VideoSummary>> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn channel(&self, _id: &str) -> AppResult<Channel> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn channel_videos(
            &self,
            _id: &str,
            _page: u32,
        ) -> AppResult<Vec<yoube_yt_dlp::model::VideoSummary>> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn playlist(&self, _id: &str) -> AppResult<Playlist> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn trending(
            &self,
            _region: Region,
        ) -> AppResult<Vec<yoube_yt_dlp::model::VideoSummary>> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn related(&self, _id: &str) -> AppResult<Vec<yoube_yt_dlp::model::VideoSummary>> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
    }

    struct NoopAccountService;
    #[async_trait::async_trait]
    impl AccountService for NoopAccountService {
        async fn current_user(&self) -> AppResult<crate::UserProfile> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn switch_user(&self, _id: i64) -> AppResult<()> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn create_user(&self, _name: &str) -> AppResult<crate::UserProfile> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn delete_user(&self, _id: i64) -> AppResult<()> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn subscriptions(&self) -> AppResult<Vec<crate::services::account::ChannelRef>> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn subscribe(&self, _: &str, _: &str, _: Option<&str>) -> AppResult<()> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn unsubscribe(&self, _: &str) -> AppResult<()> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn playlists(&self) -> AppResult<Vec<crate::AccountPlaylist>> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn playlist_items(
            &self,
            _: i64,
        ) -> AppResult<Vec<yoube_yt_dlp::model::VideoSummary>> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn playlist_add(
            &self,
            _: i64,
            _: &yoube_yt_dlp::model::VideoSummary,
        ) -> AppResult<()> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn playlist_remove(&self, _: i64, _: &str) -> AppResult<()> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn history(
            &self,
            _: u32,
        ) -> AppResult<Vec<crate::services::account::HistoryEntryView>> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn history_clear(&self) -> AppResult<()> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn mark_history(&self, _: &yoube_yt_dlp::model::VideoSummary) -> AppResult<()> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn like(&self, _: &yoube_yt_dlp::model::VideoSummary) -> AppResult<()> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn unlike(&self, _: &str) -> AppResult<()> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn watch_later(&self, _: &yoube_yt_dlp::model::VideoSummary) -> AppResult<()> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn export(&self, _: &std::path::Path) -> AppResult<()> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn import(&self, _: &std::path::Path) -> AppResult<()> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
    }

    struct NoopDownloaderService;
    #[async_trait::async_trait]
    impl crate::services::downloader::DownloaderService for NoopDownloaderService {
        async fn list_formats(&self, _: &str) -> AppResult<Vec<yoube_yt_dlp::model::Format>> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn enqueue(&self, _: &str, _: &str, _: &std::path::Path) -> AppResult<u64> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn pause(&self, _: u64) -> AppResult<()> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn resume(&self, _: u64) -> AppResult<()> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn cancel(&self, _: u64) -> AppResult<()> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
        async fn list_jobs(&self) -> AppResult<Vec<yoube_yt_dlp::downloader::Job>> {
            Err(crate::error::AppError::Internal(anyhow::anyhow!("noop")))
        }
    }

    #[test]
    fn new_context_pings() {
        let ctx = AppContext::new(
            Arc::new(NoopYoutubeService),
            Arc::new(NoopAccountService),
            Arc::new(NoopDownloaderService),
        );
        assert_eq!(ctx.ping().unwrap(), "pong");
    }
}
