use std::sync::Arc;

use yoube_core::AppContext;
use yoube_core::services::account::StorageAccountService;
use yoube_core::services::dns::AppDnsBlockService;
use yoube_core::services::downloader::YtDlpDownloaderService;
use yoube_core::services::filter::AppFilterService;
use yoube_core::services::youtube::YtDlpYoutubeService;
use yoube_storage::Storage;
use yoube_yt_dlp::YtDlp;
use yoube_yt_dlp::runner::TokioCommandRunner;

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
            // L3/L2 filter service first: its cache lives under the same
            // data dir as the DB, and the L2 gate wraps the shared yt-dlp
            // handle below so every metadata/download call is screened.
            let data_dir = app.path().data_dir().join("yoube");
            std::fs::create_dir_all(&data_dir)?;
            let filter: std::sync::Arc<AppFilterService> =
                std::sync::Arc::new(AppFilterService::new(data_dir.join("cache"))?);

            let bin = crate::sidecar::yt_dlp_path(app.handle());
            let ytdlp = YtDlp::new(bin, Arc::new(TokioCommandRunner))
                .with_blocker(filter.clone() as std::sync::Arc<dyn yoube_yt_dlp::UrlBlocker>);
            let youtube = Arc::new(YtDlpYoutubeService {
                ytdlp: ytdlp.clone(),
            });
            let downloader = Arc::new(YtDlpDownloaderService::new(ytdlp));

            // Resolve the per-user data directory then open (or create) the DB.
            // `setup` is synchronous, so drive the async storage init with the
            // Tauri async runtime (`block_on`); `Storage::open` itself is fast
            // (pool creation, no I/O beyond file creation).
            let db_path = data_dir.join("yoube.db");
            let storage = tauri::async_runtime::block_on(async {
                let s = Storage::open(&db_path).await?;
                s.migrate().await?;
                Ok::<_, yoube_core::AppError>(s)
            })?;
            let account = Arc::new(StorageAccountService::new(storage));
            let dnsblock = Arc::new(AppDnsBlockService::production(data_dir.join("backups"))?);

            let ctx = AppContext::new(youtube, account, downloader, filter, dnsblock);
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
            commands::watch_later,
            commands::list_formats,
            commands::download_enqueue,
            commands::download_pause,
            commands::download_resume,
            commands::download_cancel,
            commands::download_list,
            commands::filter_init,
            commands::filter_matches,
            commands::filter_segments_for,
            commands::filter_branding_for,
            commands::dns_install,
            commands::dns_uninstall,
            commands::dns_status,
            commands::dns_refresh
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
