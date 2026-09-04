use crate::model::{Format, VideoSummary};
use serde_json::Value;
use yoube_error::AppResult;

pub fn parse_dump_json(v: &Value) -> AppResult<VideoSummary> {
    Ok(VideoSummary {
        id: v["id"].as_str().ok_or_else(|| missing("id"))?.to_string(),
        title: v["title"]
            .as_str()
            .ok_or_else(|| missing("title"))?
            .to_string(),
        channel_id: v
            .get("channel_id")
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .to_string(),
        channel_title: v
            .get("channel")
            .or_else(|| v.get("uploader"))
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .to_string(),
        duration_s: v.get("duration").and_then(|x| x.as_f64()).map(|d| d as u32),
        view_count: v.get("view_count").and_then(|x| x.as_u64()),
        upload_date: v
            .get("upload_date")
            .and_then(|x| x.as_str())
            .map(String::from),
        thumbnail_url: v
            .get("thumbnail")
            .and_then(|x| x.as_str())
            .map(String::from),
    })
}

pub fn parse_list_formats(v: &Value) -> AppResult<Vec<Format>> {
    let arr = v["formats"].as_array().ok_or_else(|| missing("formats"))?;
    let mut out = Vec::with_capacity(arr.len());
    for f in arr {
        out.push(Format {
            format_id: f["format_id"].as_str().unwrap_or_default().to_string(),
            ext: f["ext"].as_str().unwrap_or_default().to_string(),
            url: f.get("url").and_then(|x| x.as_str()).map(String::from),
            resolution: f
                .get("resolution")
                .and_then(|x| x.as_str())
                .map(String::from),
            fps: f.get("fps").and_then(|x| x.as_f64()).map(|n| n as f32),
            vcodec: f.get("vcodec").and_then(|x| x.as_str()).map(String::from),
            acodec: f.get("acodec").and_then(|x| x.as_str()).map(String::from),
            filesize: f.get("filesize").and_then(|x| x.as_u64()),
            tbr: f.get("tbr").and_then(|x| x.as_f64()).map(|n| n as f32),
            note: f
                .get("format_note")
                .and_then(|x| x.as_str())
                .map(String::from),
        });
    }
    Ok(out)
}

fn missing(field: &'static str) -> yoube_error::AppError {
    yoube_error::AppError::Internal(anyhow::anyhow!("yt-dlp dump-json missing field: {field}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    const FIXTURE: &str = include_str!("../fixtures/dump_json.json");
    #[test]
    fn parses_summary() {
        let v: Value = serde_json::from_str(FIXTURE).unwrap();
        let s = parse_dump_json(&v).unwrap();
        assert_eq!(s.id, "dQw4w9WgXcQ");
        assert_eq!(s.title, "Never Gonna Give You Up");
    }
    #[test]
    fn parses_formats() {
        let v: Value = serde_json::from_str(FIXTURE).unwrap();
        let f = parse_list_formats(&v).unwrap();
        assert!(!f.is_empty());
        assert!(f.iter().any(|f| f.format_id == "22"));
    }
}
