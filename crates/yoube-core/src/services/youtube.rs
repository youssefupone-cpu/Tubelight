use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use yoube_yt_dlp::YtDlp;
use yoube_yt_dlp::model::{Format, VideoSummary};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Video {
    pub summary: VideoSummary,
    pub formats: Vec<Format>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Channel {
    pub id: String,
    pub title: String,
    pub description: String,
    pub thumb_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Playlist {
    pub id: String,
    pub title: String,
    pub channel_id: String,
    pub items: Vec<VideoSummary>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Region {
    US,
    GB,
    EG,
    SA,
    DE,
    FR,
    JP,
    BR,
    IN,
    AU,
    CA,
    MX,
    ES,
    IT,
    RU,
    TR,
    ZA,
    NG,
    KR,
    AR,
}

impl Region {
    pub fn code(self) -> &'static str {
        match self {
            Region::US => "US",
            Region::GB => "GB",
            Region::EG => "EG",
            Region::SA => "SA",
            Region::DE => "DE",
            Region::FR => "FR",
            Region::JP => "JP",
            Region::BR => "BR",
            Region::IN => "IN",
            Region::AU => "AU",
            Region::CA => "CA",
            Region::MX => "MX",
            Region::ES => "ES",
            Region::IT => "IT",
            Region::RU => "RU",
            Region::TR => "TR",
            Region::ZA => "ZA",
            Region::NG => "NG",
            Region::KR => "KR",
            Region::AR => "AR",
        }
    }
}

#[async_trait]
pub trait YoutubeService: Send + Sync {
    async fn get_video(&self, id: &str) -> yoube_core::AppResult<Video>;
    async fn search(&self, q: &str, page: u32) -> yoube_core::AppResult<Vec<VideoSummary>>;
    async fn channel(&self, id: &str) -> yoube_core::AppResult<Channel>;
    async fn channel_videos(&self, id: &str, page: u32)
    -> yoube_core::AppResult<Vec<VideoSummary>>;
    async fn playlist(&self, id: &str) -> yoube_core::AppResult<Playlist>;
    async fn trending(&self, region: Region) -> yoube_core::AppResult<Vec<VideoSummary>>;
    async fn related(&self, id: &str) -> yoube_core::AppResult<Vec<VideoSummary>>;
}

pub struct YtDlpYoutubeService {
    pub ytdlp: YtDlp,
}

#[async_trait]
impl YoutubeService for YtDlpYoutubeService {
    async fn get_video(&self, id: &str) -> yoube_core::AppResult<Video> {
        let url = format!("https://www.youtube.com/watch?v={id}");
        let summary = self.ytdlp.dump_json(&url).await?;
        let formats = self.ytdlp.list_formats(&url).await?;
        Ok(Video { summary, formats })
    }
    async fn search(&self, q: &str, _page: u32) -> yoube_core::AppResult<Vec<VideoSummary>> {
        let _url = format!("ytsearch:{q}");
        Err(yoube_core::AppError::Internal(anyhow::anyhow!(
            "search not yet implemented in phase 1.3"
        )))
    }
    async fn channel(&self, _id: &str) -> yoube_core::AppResult<Channel> {
        Err(yoube_core::AppError::Internal(anyhow::anyhow!(
            "channel not yet implemented"
        )))
    }
    async fn channel_videos(
        &self,
        _id: &str,
        _page: u32,
    ) -> yoube_core::AppResult<Vec<VideoSummary>> {
        Err(yoube_core::AppError::Internal(anyhow::anyhow!(
            "channel_videos not yet implemented"
        )))
    }
    async fn playlist(&self, _id: &str) -> yoube_core::AppResult<Playlist> {
        Err(yoube_core::AppError::Internal(anyhow::anyhow!(
            "playlist not yet implemented"
        )))
    }
    async fn trending(&self, _region: Region) -> yoube_core::AppResult<Vec<VideoSummary>> {
        Err(yoube_core::AppError::Internal(anyhow::anyhow!(
            "trending not yet implemented"
        )))
    }
    async fn related(&self, _id: &str) -> yoube_core::AppResult<Vec<VideoSummary>> {
        Err(yoube_core::AppError::Internal(anyhow::anyhow!(
            "related not yet implemented"
        )))
    }
}
