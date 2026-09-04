use tauri::State;
use yoube_core::AppContext;
use yoube_core::AppError;
use yoube_core::UserProfile;
use yoube_core::services::youtube::{Channel, Playlist, Region, Video};
use yoube_core::services::account::{ChannelRef, HistoryEntryView, Playlist as AccountPlaylist};
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
pub async fn create_user(ctx: State<'_, AppContext>, name: String) -> Result<UserProfile, AppError> {
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
pub async fn subscribe(ctx: State<'_, AppContext>, channel_id: String, title: String, thumb_url: Option<String>) -> Result<(), AppError> {
    ctx.account.subscribe(&channel_id, &title, thumb_url.as_deref()).await
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
pub async fn playlist_items(ctx: State<'_, AppContext>, id: i64) -> Result<Vec<VideoSummary>, AppError> {
    ctx.account.playlist_items(id).await
}

#[tauri::command]
pub async fn playlist_add(ctx: State<'_, AppContext>, id: i64, video: VideoSummary) -> Result<(), AppError> {
    ctx.account.playlist_add(id, &video).await
}

#[tauri::command]
pub async fn playlist_remove(ctx: State<'_, AppContext>, id: i64, video_id: String) -> Result<(), AppError> {
    ctx.account.playlist_remove(id, &video_id).await
}

#[tauri::command]
pub async fn history(ctx: State<'_, AppContext>, page: u32) -> Result<Vec<HistoryEntryView>, AppError> {
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
