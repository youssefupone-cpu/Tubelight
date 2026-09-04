use std::sync::Arc;

use crate::error::AppResult;
use crate::services::youtube::YoutubeService;

pub struct AppContext {
    pub youtube: Arc<dyn YoutubeService>,
}

impl AppContext {
    pub fn new(youtube: Arc<dyn YoutubeService>) -> Self {
        Self { youtube }
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

    #[test]
    fn new_context_pings() {
        let ctx = AppContext::new(Arc::new(NoopYoutubeService));
        assert_eq!(ctx.ping().unwrap(), "pong");
    }
}
