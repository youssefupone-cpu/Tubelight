use tauri::State;
use yoube_core::AppContext;
use yoube_core::AppError;
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
