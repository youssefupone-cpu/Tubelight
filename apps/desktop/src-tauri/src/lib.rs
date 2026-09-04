use std::sync::Arc;

use yoube_core::AppContext;
use yoube_core::services::youtube::YtDlpYoutubeService;
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
            let ctx = AppContext::new(Arc::new(YtDlpYoutubeService { ytdlp }));
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
            commands::related
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
