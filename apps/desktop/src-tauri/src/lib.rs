use std::sync::Arc;

use yoube_core::AppContext;
use yoube_core::services::account::StorageAccountService;
use yoube_core::services::youtube::YtDlpYoutubeService;
use yoube_storage::Storage;
use yoube_yt_dlp::runner::TokioCommandRunner;
use yoube_yt_dlp::YtDlp;

mod commands;
mod sidecar;

#[tauri::command]
fn ping(ctx: tauri::State<'_, AppContext>) -> Result<String, yoube_core::AppError> {
    Ok(ctx.ping()?.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let bin = crate::sidecar::yt_dlp_path(app.handle());
            let ytdlp = YtDlp::new(bin, Arc::new(TokioCommandRunner));
            let youtube = Arc::new(YtDlpYoutubeService { ytdlp });

            // Resolve the per-user data directory then open (or create) the DB.
            // `app.path().data_dir()` returns `$HOME/.local/share/yoube` on
            // Linux / `%APPDATA%\yoube` on Windows — exactly the XDG path the
            // storage layer expects. The directory is created lazily by
            // `Storage::open` via `create_if_missing`.
            let data_dir = app.path().data_dir().join("yoube");
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("yoube.db");
            let storage = Storage::open(&db_path).await?;
            storage.migrate().await?;
            let account = Arc::new(StorageAccountService::new(storage));

            let ctx = AppContext::new(youtube, account);
            app.manage(ctx);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ping,
            commands::get_video,
            commands::search,
            commands::channel,
            commands::channel_videos,
            commands::playlist,
            commands::trending,
            commands::related,
            commands::current_user,
            commands::switch_user,
            commands::create_user,
            commands::delete_user,
            commands::subscriptions,
            commands::subscribe,
            commands::unsubscribe,
            commands::playlists,
            commands::playlist_items,
            commands::playlist_add,
            commands::playlist_remove,
            commands::history,
            commands::history_clear,
            commands::like,
            commands::unlike,
            commands::watch_later
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
