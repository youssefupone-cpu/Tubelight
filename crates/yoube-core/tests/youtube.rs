use std::path::{Path, PathBuf};
use std::sync::Arc;

use yoube_core::services::youtube::{YoutubeService, YtDlpYoutubeService};
use yoube_yt_dlp::YtDlp;
use yoube_yt_dlp::runner::CommandRunner;

struct StaticRunner {
    fixture: &'static str,
    _exit: i32,
}

#[async_trait::async_trait]
impl CommandRunner for StaticRunner {
    async fn output(
        &self,
        _bin: &Path,
        _args: &[&str],
    ) -> yoube_core::AppResult<std::process::Output> {
        use std::process::Output;
        Ok(Output {
            status: std::process::ExitStatus::default(),
            stdout: self.fixture.as_bytes().to_vec(),
            stderr: vec![],
        })
    }
}

#[tokio::test]
async fn get_video_uses_dump_json_fixture() {
    let fixture = include_str!("../../yoube-yt-dlp/fixtures/dump_json.json");
    let runner: Arc<dyn CommandRunner> = Arc::new(StaticRunner { fixture, _exit: 0 });
    let ytdlp = YtDlp::new(PathBuf::from("yt-dlp"), runner);
    let svc = YtDlpYoutubeService { ytdlp };
    let v = svc.get_video("dQw4w9WgXcQ").await.unwrap();
    assert_eq!(v.summary.id, "dQw4w9WgXcQ");
    assert!(!v.formats.is_empty());
}

#[tokio::test]
async fn search_uses_flat_playlist_fixture() {
    let fixture = include_str!("fixtures/search.json");
    let runner: Arc<dyn CommandRunner> = Arc::new(StaticRunner { fixture, _exit: 0 });
    let ytdlp = YtDlp::new(PathBuf::from("yt-dlp"), runner);
    let svc = YtDlpYoutubeService { ytdlp };
    let results = svc.search("Rust programming", 0).await.unwrap();
    assert!(!results.is_empty());
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].id, "9fYVNmBjRUs");
    assert_eq!(
        results[0].title,
        "Rust Programming Tutorial #1 - What is Rust?"
    );
    assert_eq!(results[0].channel_id, "UCWs0E4Ig_ZtCi5mfu5oyYUzA");
    assert_eq!(results[0].channel_title, "freeCodeCamp.org");
    assert_eq!(results[0].view_count, Some(3200000));
    assert_eq!(
        results[0].thumbnail_url.as_deref(),
        Some("https://i.ytimg.com/vi/9fYVNmBjRUs/hqdefault.jpg")
    );
}
