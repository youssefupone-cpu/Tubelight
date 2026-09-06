use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use yoube_yt_dlp::YtDlp;
use yoube_yt_dlp::model::{Format, VideoSummary};
use yoube_yt_dlp::parser::{parse_dump_json, parse_list_formats};

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
        let v = self.ytdlp.dump_json_full(&url).await?;
        let summary = parse_dump_json(&v)?;
        let formats = parse_list_formats(&v)?;
        Ok(Video { summary, formats })
    }
    async fn search(&self, q: &str, page: u32) -> yoube_core::AppResult<Vec<VideoSummary>> {
        // NOTE: `ytsearchN:` tells yt-dlp to fetch N results; real pagination
        // needs `--playlist-items` (Phase-1 polish TODO).
        let n = ((page + 1) * 20).to_string();
        let url = format!("ytsearch{n}:{q}");
        self.flat_list(&url).await
    }
    async fn channel(&self, id: &str) -> yoube_core::AppResult<Channel> {
        let url = format!("https://www.youtube.com/channel/{id}");
        let v = self.ytdlp.dump_json_full(&url).await?;
        Ok(Channel {
            id: id.to_string(),
            title: v
                .get("title")
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string(),
            description: v
                .get("description")
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string(),
            thumb_url: v
                .get("thumbnails")
                .and_then(|t| t.as_array())
                .and_then(|a| a.last())
                .and_then(|t| t.get("url"))
                .and_then(|u| u.as_str())
                .map(String::from),
        })
    }
    async fn channel_videos(
        &self,
        id: &str,
        _page: u32,
    ) -> yoube_core::AppResult<Vec<VideoSummary>> {
        self.flat_list(&format!("https://www.youtube.com/channel/{id}/videos"))
            .await
    }
    async fn playlist(&self, id: &str) -> yoube_core::AppResult<Playlist> {
        let url = format!("https://www.youtube.com/playlist?list={id}");
        let args = [
            "--flat-playlist",
            "--skip-download",
            "--dump-single-json",
            "--no-warnings",
            &url,
        ];
        let out = self.ytdlp.runner.output(&self.ytdlp.bin, &args).await?;
        if !out.status.success() {
            return Err(yoube_core::AppError::YtDlp(
                String::from_utf8_lossy(&out.stderr).into_owned(),
            ));
        }
        let v: serde_json::Value = serde_json::from_slice(&out.stdout)?;
        let entries = v
            .get("entries")
            .and_then(|e| e.as_array())
            .cloned()
            .unwrap_or_default();
        let items: Vec<VideoSummary> = entries
            .into_iter()
            .filter_map(|e| parse_dump_json(&e).ok())
            .collect();
        Ok(Playlist {
            id: id.to_string(),
            title: v
                .get("title")
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string(),
            channel_id: v
                .get("channel_id")
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string(),
            items,
        })
    }
    async fn trending(&self, region: Region) -> yoube_core::AppResult<Vec<VideoSummary>> {
        self.flat_list(&format!(
            "https://www.youtube.com/feed/trending?gl={}",
            region.code()
        ))
        .await
    }
    async fn related(&self, id: &str) -> yoube_core::AppResult<Vec<VideoSummary>> {
        self.flat_list(&format!("https://www.youtube.com/watch?v={id}"))
            .await
    }
}

impl YtDlpYoutubeService {
    async fn flat_list(&self, url: &str) -> yoube_core::AppResult<Vec<VideoSummary>> {
        let args = [
            "--flat-playlist",
            "--skip-download",
            "--dump-single-json",
            "--no-warnings",
            url,
        ];
        let out = self.ytdlp.runner.output(&self.ytdlp.bin, &args).await?;
        if !out.status.success() {
            return Err(yoube_core::AppError::YtDlp(
                String::from_utf8_lossy(&out.stderr).into_owned(),
            ));
        }
        let v: serde_json::Value = serde_json::from_slice(&out.stdout)?;
        let entries = v
            .get("entries")
            .and_then(|e| e.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(entries
            .into_iter()
            .filter_map(|e| parse_dump_json(&e).ok())
            .collect())
    }
}
