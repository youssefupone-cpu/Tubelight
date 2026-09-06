//! SponsorBlock segment client (`sponsor.ajay.app`, no auth).
//!
//! Results are cached under `cache_dir/<video_id>.json` with a 24h TTL and
//! the client fails open: any network/parse error yields `Ok(vec![])` so
//! playback never breaks because a metadata call failed.

use std::path::Path;
use std::time::{Duration, SystemTime};

use serde::Deserialize;
use yoube_error::AppResult;

use crate::model::{Segment, SegmentCategory};

const API_BASE: &str = "https://sponsor.ajay.app";
const CACHE_TTL: Duration = Duration::from_secs(24 * 3600);
const MAX_VIDEO_ID_LEN: usize = 64;

#[derive(Debug, Deserialize)]
struct ApiSegment {
    #[serde(rename = "category")]
    category: String,
    #[serde(rename = "segment")]
    segment: [f32; 2],
    #[serde(rename = "UUID")]
    uuid: String,
}

fn api_category(s: &str) -> Option<SegmentCategory> {
    match s {
        "sponsor" => Some(SegmentCategory::Sponsor),
        "intro" => Some(SegmentCategory::Intro),
        "outro" => Some(SegmentCategory::Outro),
        "selfpromo" => Some(SegmentCategory::SelfPromo),
        "preview" => Some(SegmentCategory::Preview),
        "music_offtopic" => Some(SegmentCategory::MusicOfftopic),
        "filler" => Some(SegmentCategory::Filler),
        "interaction_reminder" => Some(SegmentCategory::Interaction),
        "poi_highlight" => Some(SegmentCategory::PoiHighlight),
        _ => None,
    }
}

/// Parse the raw SponsorBlock JSON body into `Segment`s, dropping unknown
/// categories and inverted ranges.
pub fn parse_segments(body: &str) -> Vec<Segment> {
    let items: Vec<ApiSegment> = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    items
        .into_iter()
        .filter_map(|it| {
            let category = api_category(&it.category)?;
            let [start_s, end_s] = it.segment;
            if end_s <= start_s || !start_s.is_finite() || !end_s.is_finite() {
                return None;
            }
            Some(Segment {
                category,
                start_s,
                end_s,
                uuid: it.uuid,
            })
        })
        .collect()
}

fn cache_path(cache_dir: &Path, video_id: &str) -> Option<std::path::PathBuf> {
    if video_id.is_empty()
        || video_id.len() > MAX_VIDEO_ID_LEN
        || video_id.contains(['/', '\\', '.'])
    {
        return None;
    }
    Some(cache_dir.join(format!("{video_id}.json")))
}

fn cache_fresh(path: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        return false;
    };
    SystemTime::now()
        .duration_since(modified)
        .map(|age| age < CACHE_TTL)
        .unwrap_or(false)
}

/// Fetch skip segments for a video (cached, fail-open).
pub async fn segments_for(
    http: &reqwest::Client,
    cache_dir: &Path,
    video_id: &str,
    categories: &[SegmentCategory],
) -> AppResult<Vec<Segment>> {
    if let Some(path) = cache_path(cache_dir, video_id)
        && cache_fresh(&path)
        && let Ok(body) = tokio::fs::read_to_string(&path).await
    {
        return Ok(parse_segments(&body));
    }

    let cats: Vec<&str> = categories.iter().map(|c| c.slug()).collect();
    let cats_json = serde_json::to_string(&cats).unwrap_or_else(|_| "[]".into());
    let url = format!("{API_BASE}/api/skipSegments/{video_id}?categories={cats_json}");

    let body = match http.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => match resp.text().await {
            Ok(t) => t,
            Err(_) => return Ok(vec![]),
        },
        // 404 = no segments submitted for this video; 429 = rate-limited.
        // Both are normal: play without skipping.
        _ => return Ok(vec![]),
    };

    if let Some(path) = cache_path(cache_dir, video_id)
        && let Some(parent) = path.parent()
    {
        let _ = tokio::fs::create_dir_all(parent).await;
        let _ = tokio::fs::write(&path, &body).await;
    }
    Ok(parse_segments(&body))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"[
        {"category":"sponsor","segment":[10.0,20.5],"UUID":"abc-123"},
        {"category":"intro","segment":[0.0,5.0],"UUID":"def-456"},
        {"category":"unknown_future","segment":[1.0,2.0],"UUID":"zzz"},
        {"category":"sponsor","segment":[30.0,25.0],"UUID":"inverted"}
    ]"#;

    #[test]
    fn parses_and_filters_sample() {
        let segs = parse_segments(SAMPLE);
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].category, SegmentCategory::Sponsor);
        assert_eq!(segs[0].start_s, 10.0);
        assert_eq!(segs[0].end_s, 20.5);
        assert_eq!(segs[0].uuid, "abc-123");
        assert_eq!(segs[1].category, SegmentCategory::Intro);
    }

    #[test]
    fn garbage_body_yields_empty() {
        assert!(parse_segments("not json").is_empty());
        assert!(parse_segments("{}").is_empty());
    }

    #[test]
    fn unsafe_video_ids_have_no_cache_path() {
        let dir = Path::new("/tmp");
        assert!(cache_path(dir, "../evil").is_none());
        assert!(cache_path(dir, "").is_none());
        assert!(cache_path(dir, "dQw4w9WgXcQ").is_some());
    }
}
