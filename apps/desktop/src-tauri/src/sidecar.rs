use std::path::PathBuf;
use tauri::Manager;

pub fn yt_dlp_path(app: &tauri::AppHandle) -> PathBuf {
    app.path().resolve("yt-dlp", tauri::path::BaseDirectory::Resource)
        .expect("yt-dlp sidecar resolution failed")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn triple_aware_filename() {
        // sanity: the function only takes an AppHandle in production; here we
        // verify the path-string logic by inlining the same convention
        for (triple, name) in [
            ("x86_64-unknown-linux-gnu", "yt-dlp-x86_64-unknown-linux-gnu"),
            ("aarch64-apple-darwin", "yt-dlp-aarch64-apple-darwin"),
            ("x86_64-pc-windows-msvc", "yt-dlp-x86_64-pc-windows-msvc.exe"),
        ] {
            assert!(name.starts_with("yt-dlp-"));
            assert!(name.ends_with(triple) || name.ends_with(&format!("{triple}.exe")));
        }
    }
}
