//! DeArrow branding client (`sponsor.ajay.app/api/branding`, no auth).
//!
//! Same contract as `sponsorblock.rs`: file-cached with 24h TTL, fail-open
//! (`Ok(Branding::default())` on any error) so title/thumb replacement never
//! breaks playback.

use std::path::Path;
use std::time::{Duration, SystemTime};

use serde::Deserialize;
use yoube_error::AppResult;

use crate::model::Branding;

const API_BASE: &str = "https://sponsor.ajay.app";
const CACHE_TTL: Duration = Duration::from_secs(24 * 3600);

#[derive(Debug, Deserialize)]
struct TitleCandidate {
    title: String,
    votes: i32,
}

#[derive(Debug, Deserialize)]
struct ThumbCandidate {
    #[serde(rename = "thumbnail")]
    thumbnail: String,
    votes: i32,
}

#[derive(Debug, Deserialize)]
struct BrandingResponse {
    #[serde(default)]
    titles: Vec<TitleCandidate>,
    #[serde(default)]
    thumbnails: Vec<ThumbCandidate>,
}

/// Pick the highest-voted title/thumb from a branding response body.
pub fn parse_branding(body: &str) -> Branding {
    let resp: BrandingResponse = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => return Branding::default(),
    };
    let title = resp
        .titles
        .iter()
        .max_by_key(|t| t.votes)
        .map(|t| t.title.clone());
    let thumbnail_url = resp
        .thumbnails
        .iter()
        .max_by_key(|t| t.votes)
        .map(|t| t.thumbnail.clone());
    Branding {
        title,
        thumbnail_url,
    }
}

/// Fetch crowd-sourced branding for a video (cached, fail-open).
pub async fn branding_for(
    http: &reqwest::Client,
    cache_dir: &Path,
    video_id: &str,
) -> AppResult<Branding> {
    if video_id.is_empty() || video_id.len() > 64 || video_id.contains(['/', '\\', '.']) {
        return Ok(Branding::default());
    }
    let path = cache_dir.join(format!("{video_id}-branding.json"));
    if let Ok(meta) = std::fs::metadata(&path)
        && let Ok(modified) = meta.modified()
        && SystemTime::now()
            .duration_since(modified)
            .map(|age| age < CACHE_TTL)
            .unwrap_or(false)
        && let Ok(body) = tokio::fs::read_to_string(&path).await
    {
        return Ok(parse_branding(&body));
    }

    let url = format!("{API_BASE}/api/branding/{video_id}");
    let body = match http.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => match resp.text().await {
            Ok(t) => t,
            Err(_) => return Ok(Branding::default()),
        },
        _ => return Ok(Branding::default()),
    };
    if let Some(parent) = path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
        let _ = tokio::fs::write(&path, &body).await;
    }
    Ok(parse_branding(&body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_highest_voted() {
        let body = r#"{
            "titles": [
                {"title": "bad", "votes": -2},
                {"title": "good", "votes": 10}
            ],
            "thumbnails": [
                {"thumbnail": "https://i.ytimg.com/a.jpg", "votes": 3}
            ]
        }"#;
        let b = parse_branding(body);
        assert_eq!(b.title.as_deref(), Some("good"));
        assert_eq!(
            b.thumbnail_url.as_deref(),
            Some("https://i.ytimg.com/a.jpg")
        );
    }

    #[test]
    fn garbage_body_yields_default() {
        assert_eq!(parse_branding("nope"), Branding::default());
    }
}
