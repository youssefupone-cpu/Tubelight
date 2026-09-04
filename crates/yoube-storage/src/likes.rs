use serde::{Deserialize, Serialize};
use sqlx::Row;
use yoube_error::{AppError, AppResult};

use crate::Storage;

/// A liked-video summary stored in the `likes` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LikeEntry {
    pub video_id: String,
    pub title: String,
    pub channel_id: Option<String>,
    pub channel_title: Option<String>,
    pub duration_s: Option<i64>,
    pub thumb_url: Option<String>,
}

/// List all liked videos for a user, newest first.
pub async fn list(s: &Storage, user_id: i64) -> AppResult<Vec<LikeEntry>> {
    let rows = sqlx::query(
        "SELECT video_id, title, channel_id, channel_title, duration_s, thumb_url \
         FROM likes WHERE user_id = ? ORDER BY liked_at DESC",
    )
    .bind(user_id)
    .fetch_all(&s.pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| LikeEntry {
            video_id: r.get("video_id"),
            title: r.get("title"),
            channel_id: r.get("channel_id"),
            channel_title: r.get("channel_title"),
            duration_s: r.get("duration_s"),
            thumb_url: r.get("thumb_url"),
        })
        .collect())
}

/// Like (upsert) a video.
#[allow(clippy::too_many_arguments)]
pub async fn add(
    s: &Storage,
    user_id: i64,
    video_id: &str,
    title: &str,
    channel_id: Option<&str>,
    channel_title: Option<&str>,
    duration_s: Option<i64>,
    thumb_url: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        "INSERT OR REPLACE INTO likes \
         (user_id, video_id, title, channel_id, channel_title, duration_s, thumb_url) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(video_id)
    .bind(title)
    .bind(channel_id)
    .bind(channel_title)
    .bind(duration_s)
    .bind(thumb_url)
    .execute(&s.pool)
    .await?;
    Ok(())
}

/// Unlike (remove) a video.
pub async fn remove(s: &Storage, user_id: i64, video_id: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM likes WHERE user_id = ? AND video_id = ?")
        .bind(user_id)
        .bind(video_id)
        .execute(&s.pool)
        .await?;
    Ok(())
}

/// Check whether a video is in the user's likes.
pub async fn contains(s: &Storage, user_id: i64, video_id: &str) -> AppResult<bool> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM likes WHERE user_id = ? AND video_id = ?)",
    )
    .bind(user_id)
    .bind(video_id)
    .fetch_one(&s.pool)
    .await?;
    Ok(exists)
}

/// Add a video to the user's "watch later" smart playlist.
/// Add a video to the user's "watch later" smart playlist.
///
/// Looks up the playlist flagged `is_watch_later = 1`; if it does not exist yet
/// it is created automatically (matching the spec's invariant that every user
/// has these two smart playlists on demand).
#[allow(clippy::too_many_arguments)]
pub async fn watch_later(
    s: &Storage,
    user_id: i64,
    video_id: &str,
    title: &str,
    channel_id: Option<&str>,
    channel_title: Option<&str>,
    duration_s: Option<i64>,
    thumb_url: Option<&str>,
) -> AppResult<()> {
    let playlist_id: i64 = match crate::playlists::get_watch_later(s, user_id).await {
        Ok(id) => id,
        Err(AppError::NotFound(_)) => {
            crate::playlists::create_special(s, user_id, "Watch later", true, false).await?
        }
        Err(e) => return Err(e),
    };
    crate::playlists::add_item(
        s,
        playlist_id,
        video_id,
        title,
        channel_id,
        channel_title,
        duration_s,
        thumb_url,
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> (Storage, i64) {
        let tmp = Box::leak(Box::new(tempfile::tempdir().unwrap()));
        let s = Storage::open(&tmp.path().join("test.db")).await.unwrap();
        s.migrate().await.unwrap();
        let u = crate::users::create(&s, "grace").await.unwrap();
        (s, u.id)
    }

    #[tokio::test]
    async fn like_and_unlike() {
        let (s, uid) = setup().await;

        add(&s, uid, "vid1", "Title", None, None, Some(200), Some("https://t.png"))
            .await
            .unwrap();
        assert!(contains(&s, uid, "vid1").await.unwrap());

        remove(&s, uid, "vid1").await.unwrap();
        assert!(!contains(&s, uid, "vid1").await.unwrap());
    }

    #[tokio::test]
    async fn like_uses_upsert() {
        let (s, uid) = setup().await;
        add(&s, uid, "vid1", "Old Title", None, None, Some(200), None)
            .await
            .unwrap();
        add(&s, uid, "vid1", "New Title", None, None, Some(210), None)
            .await
            .unwrap();

        let likes = list(&s, uid).await.unwrap();
        assert_eq!(likes.len(), 1);
        assert_eq!(likes[0].title, "New Title");
    }

    #[tokio::test]
    async fn watch_later_creates_playlist_if_needed() {
        let (s, uid) = setup().await;

        watch_later(&s, uid, "vid1", "Title", None, None, None, None)
            .await
            .unwrap();

        let pid = crate::playlists::get_watch_later(&s, uid).await.unwrap();
        let items = crate::playlists::items(&s, pid).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].video_id, "vid1");
    }
}
