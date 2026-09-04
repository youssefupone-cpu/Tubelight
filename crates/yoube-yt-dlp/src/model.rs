use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Format {
    pub format_id: String,
    pub ext: String,
    pub url: Option<String>,
    pub resolution: Option<String>,
    pub fps: Option<f32>,
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
    pub filesize: Option<u64>,
    pub tbr: Option<f32>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct VideoSummary {
    pub id: String,
    pub title: String,
    pub channel_id: String,
    pub channel_title: String,
    pub duration_s: Option<u32>,
    pub view_count: Option<u64>,
    pub upload_date: Option<String>, // YYYYMMDD
    pub thumbnail_url: Option<String>,
}
