use tauri::State;
use yoube_core::AppContext;
use yoube_core::AppError;
use yoube_core::UserProfile;
use yoube_core::services::account::{ChannelRef, HistoryEntryView, Playlist as AccountPlaylist};
use yoube_core::services::youtube::{Channel, Playlist, Region, Video};
use yoube_yt_dlp::model::VideoSummary;

#[tauri::command]
// #[specta::specta]   // offline-unavailable (tauri-specta/specta contracts feature); re-enable with network.
pub async fn get_video(ctx: State<'_, AppContext>, id: String) -> Result<Video, AppError> {
    ctx.youtube.get_video(&id).await
}

#[tauri::command]
// #[specta::specta]
pub async fn search(
    ctx: State<'_, AppContext>,
    q: String,
    page: u32,
) -> Result<Vec<VideoSummary>, AppError> {
    ctx.youtube.search(&q, page).await
}

#[tauri::command]
// #[specta::specta]
pub async fn channel(ctx: State<'_, AppContext>, id: String) -> Result<Channel, AppError> {
    ctx.youtube.channel(&id).await
}

#[tauri::command]
// #[specta::specta]
pub async fn channel_videos(
    ctx: State<'_, AppContext>,
    id: String,
    page: u32,
) -> Result<Vec<VideoSummary>, AppError> {
    ctx.youtube.channel_videos(&id, page).await
}

#[tauri::command]
// #[specta::specta]
pub async fn playlist(ctx: State<'_, AppContext>, id: String) -> Result<Playlist, AppError> {
    ctx.youtube.playlist(&id).await
}

#[tauri::command]
// #[specta::specta]
pub async fn trending(
    ctx: State<'_, AppContext>,
    region: Region,
) -> Result<Vec<VideoSummary>, AppError> {
    ctx.youtube.trending(region).await
}

#[tauri::command]
// #[specta::specta]
pub async fn related(
    ctx: State<'_, AppContext>,
    id: String,
) -> Result<Vec<VideoSummary>, AppError> {
    ctx.youtube.related(&id).await
}

#[tauri::command]
pub async fn current_user(ctx: State<'_, AppContext>) -> Result<UserProfile, AppError> {
    ctx.account.current_user().await
}

#[tauri::command]
pub async fn switch_user(ctx: State<'_, AppContext>, id: i64) -> Result<(), AppError> {
    ctx.account.switch_user(id).await
}

#[tauri::command]
pub async fn create_user(
    ctx: State<'_, AppContext>,
    name: String,
) -> Result<UserProfile, AppError> {
    ctx.account.create_user(&name).await
}

#[tauri::command]
pub async fn delete_user(ctx: State<'_, AppContext>, id: i64) -> Result<(), AppError> {
    ctx.account.delete_user(id).await
}

#[tauri::command]
pub async fn subscriptions(ctx: State<'_, AppContext>) -> Result<Vec<ChannelRef>, AppError> {
    ctx.account.subscriptions().await
}

#[tauri::command]
pub async fn subscribe(
    ctx: State<'_, AppContext>,
    channel_id: String,
    title: String,
    thumb_url: Option<String>,
) -> Result<(), AppError> {
    ctx.account
        .subscribe(&channel_id, &title, thumb_url.as_deref())
        .await
}

#[tauri::command]
pub async fn unsubscribe(ctx: State<'_, AppContext>, channel_id: String) -> Result<(), AppError> {
    ctx.account.unsubscribe(&channel_id).await
}

#[tauri::command]
pub async fn playlists(ctx: State<'_, AppContext>) -> Result<Vec<AccountPlaylist>, AppError> {
    ctx.account.playlists().await
}

#[tauri::command]
pub async fn playlist_items(
    ctx: State<'_, AppContext>,
    id: i64,
) -> Result<Vec<VideoSummary>, AppError> {
    ctx.account.playlist_items(id).await
}

#[tauri::command]
pub async fn playlist_add(
    ctx: State<'_, AppContext>,
    id: i64,
    video: VideoSummary,
) -> Result<(), AppError> {
    ctx.account.playlist_add(id, &video).await
}

#[tauri::command]
pub async fn playlist_remove(
    ctx: State<'_, AppContext>,
    id: i64,
    video_id: String,
) -> Result<(), AppError> {
    ctx.account.playlist_remove(id, &video_id).await
}

#[tauri::command]
pub async fn history(
    ctx: State<'_, AppContext>,
    page: u32,
) -> Result<Vec<HistoryEntryView>, AppError> {
    ctx.account.history(page).await
}

#[tauri::command]
pub async fn history_clear(ctx: State<'_, AppContext>) -> Result<(), AppError> {
    ctx.account.history_clear().await
}

#[tauri::command]
pub async fn like(ctx: State<'_, AppContext>, video: VideoSummary) -> Result<(), AppError> {
    ctx.account.like(&video).await
}

#[tauri::command]
pub async fn unlike(ctx: State<'_, AppContext>, video_id: String) -> Result<(), AppError> {
    ctx.account.unlike(&video_id).await
}

#[tauri::command]
pub async fn watch_later(ctx: State<'_, AppContext>, video: VideoSummary) -> Result<(), AppError> {
    ctx.account.watch_later(&video).await
}

/// Expand a user-supplied file path: rejects empties and resolves a leading
/// `~/` against `$HOME` (Unix) / `%USERPROFILE%` (Windows). Std-only so no
/// new crate dependency is needed for the export/import flow.
fn expand_user_path(raw: &str) -> Result<std::path::PathBuf, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::NotFound("empty path: choose a file first".into()));
    }
    if let Some(rest) = trimmed
        .strip_prefix("~/")
        .or_else(|| trimmed.strip_prefix("~\\"))
    {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| AppError::NotFound("cannot resolve home directory".into()))?;
        Ok(std::path::PathBuf::from(home).join(rest))
    } else {
        Ok(std::path::PathBuf::from(trimmed))
    }
}

#[tauri::command]
// #[specta::specta]
pub async fn export_account(ctx: State<'_, AppContext>, dest: String) -> Result<String, AppError> {
    let path = expand_user_path(&dest)?;
    ctx.account.export(&path).await?;
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
// #[specta::specta]
pub async fn import_account(ctx: State<'_, AppContext>, src: String) -> Result<(), AppError> {
    let path = expand_user_path(&src)?;
    ctx.account.import(&path).await
}

#[tauri::command]
// #[specta::specta]
pub async fn list_formats(
    ctx: State<'_, AppContext>,
    video_id: String,
) -> Result<Vec<yoube_yt_dlp::model::Format>, AppError> {
    ctx.downloader.list_formats(&video_id).await
}

#[tauri::command]
// #[specta::specta]
pub async fn download_enqueue(
    ctx: State<'_, AppContext>,
    video_id: String,
    format_id: String,
    dest_dir: String,
) -> Result<u64, AppError> {
    ctx.downloader
        .enqueue(&video_id, &format_id, std::path::Path::new(&dest_dir))
        .await
}

#[tauri::command]
// #[specta::specta]
pub async fn download_pause(ctx: State<'_, AppContext>, id: u64) -> Result<(), AppError> {
    ctx.downloader.pause(id).await
}

#[tauri::command]
// #[specta::specta]
pub async fn download_resume(ctx: State<'_, AppContext>, id: u64) -> Result<(), AppError> {
    ctx.downloader.resume(id).await
}

#[tauri::command]
// #[specta::specta]
pub async fn download_cancel(ctx: State<'_, AppContext>, id: u64) -> Result<(), AppError> {
    ctx.downloader.cancel(id).await
}

#[tauri::command]
// #[specta::specta]
pub async fn download_list(
    ctx: State<'_, AppContext>,
) -> Result<Vec<yoube_yt_dlp::downloader::Job>, AppError> {
    ctx.downloader.list_jobs().await
}

#[tauri::command]
// #[specta::specta]
pub async fn filter_init(ctx: State<'_, AppContext>) -> Result<usize, AppError> {
    ctx.filter.init().await
}

#[tauri::command]
// #[specta::specta]
pub async fn filter_matches(
    ctx: State<'_, AppContext>,
    url: String,
    source_url: String,
) -> Result<bool, AppError> {
    // `matches` is sync and may block its worker briefly; keep it off the
    // async runtime with `spawn_blocking`.
    let filter = ctx.filter.clone();
    Ok(
        tokio::task::spawn_blocking(move || filter.matches(&url, &source_url))
            .await
            .map_err(|e| AppError::Internal(e.into()))?,
    )
}

#[tauri::command]
// #[specta::specta]
pub async fn filter_segments_for(
    ctx: State<'_, AppContext>,
    video_id: String,
) -> Result<Vec<yoube_filter::Segment>, AppError> {
    ctx.filter.segments_for(&video_id).await
}

#[tauri::command]
// #[specta::specta]
pub async fn filter_branding_for(
    ctx: State<'_, AppContext>,
    video_id: String,
) -> Result<yoube_filter::Branding, AppError> {
    ctx.filter.branding_for(&video_id).await
}

#[tauri::command]
// #[specta::specta]
pub async fn dns_install(ctx: State<'_, AppContext>) -> Result<usize, AppError> {
    ctx.dnsblock.install().await
}

#[tauri::command]
// #[specta::specta]
pub async fn dns_uninstall(ctx: State<'_, AppContext>) -> Result<(), AppError> {
    ctx.dnsblock.uninstall().await
}

#[tauri::command]
// #[specta::specta]
pub async fn dns_status(ctx: State<'_, AppContext>) -> Result<String, AppError> {
    ctx.dnsblock.status().await
}

#[tauri::command]
// #[specta::specta]
pub async fn dns_refresh(ctx: State<'_, AppContext>) -> Result<usize, AppError> {
    ctx.dnsblock.refresh().await
}

/// Update metadata surfaced to Settings → About. `available` is false when
/// the endpoint is unreachable or the app is current (fail-open: the user
/// can always open the releases page instead).
#[derive(Debug, Clone, serde::Serialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub current: String,
    pub latest: Option<String>,
    pub notes: Option<String>,
}

#[tauri::command]
// #[specta::specta]
pub async fn check_update(app: tauri::AppHandle) -> Result<UpdateInfo, AppError> {
    use tauri_plugin_updater::UpdaterExt;
    let current = app.package_info().version.to_string();
    let found = app
        .updater()
        .map_err(|e| AppError::Internal(e.into()))?
        .check()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    Ok(match found {
        Some(u) => UpdateInfo {
            available: true,
            current,
            latest: Some(u.version),
            notes: u.body,
        },
        None => UpdateInfo {
            available: false,
            current,
            latest: None,
            notes: None,
        },
    })
}
