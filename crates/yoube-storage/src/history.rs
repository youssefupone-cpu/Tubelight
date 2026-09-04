use serde::{Deserialize, Serialize};
use sqlx::Row;
use yoube_error::AppResult;

use crate::Storage;

/// One row in the watch history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub video_id: String,
    pub title: String,
    pub channel_id: Option<String>,
    pub channel_title: Option<String>,
    pub duration_s: Option<i64>,
    pub thumb_url: Option<String>,
    pub watched_at: String,
    pub position_s: i64,
}

const HISTORY_PAGE_SIZE: i64 = 50;

/// Paginated history, newest first.
pub async fn list(s: &Storage, user_id: i64, page: u32) -> AppResult<Vec<HistoryEntry>> {
    let offset = (page as i64) * HISTORY_PAGE_SIZE;
    let rows = sqlx::query(
        "SELECT id, video_id, title, channel_id, channel_title, duration_s, thumb_url, \
         watched_at, position_s FROM history WHERE user_id = ? \
         ORDER BY watched_at DESC, id DESC LIMIT ? OFFSET ?",
    )
    .bind(user_id)
    .bind(HISTORY_PAGE_SIZE)
    .bind(offset)
    .fetch_all(&s.pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| HistoryEntry {
            id: r.get("id"),
            video_id: r.get("video_id"),
            title: r.get("title"),
            channel_id: r.get("channel_id"),
            channel_title: r.get("channel_title"),
            duration_s: r.get("duration_s"),
            thumb_url: r.get("thumb_url"),
            watched_at: r.get("watched_at"),
            position_s: r.get("position_s"),
        })
        .collect())
}

/// Insert or refresh a history entry for a video.
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
    position_s: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO history \
         (user_id, video_id, title, channel_id, channel_title, duration_s, thumb_url, position_s) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(video_id)
    .bind(title)
    .bind(channel_id)
    .bind(channel_title)
    .bind(duration_s)
    .bind(thumb_url)
    .bind(position_s)
    .execute(&s.pool)
    .await?;
    Ok(())
}

/// Wipe the history for a single user.
pub async fn clear(s: &Storage, user_id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM history WHERE user_id = ?")
        .bind(user_id)
        .execute(&s.pool)
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
        let u = crate::users::create(&s, "eve").await.unwrap();
        (s, u.id)
    }

    #[tokio::test]
    async fn add_list_clear_history() {
        let (s, uid) = setup().await;

        add(&s, uid, "vid1", "Title1", Some("ch1"), Some("Channel 1"), Some(120), None, 0)
            .await
            .unwrap();
        add(&s, uid, "vid2", "Title2", Some("ch2"), Some("Channel 2"), Some(60), None, 30)
            .await
            .unwrap();

        let entries = list(&s, uid, 0).await.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].video_id, "vid2"); // newest first
        assert_eq!(entries[1].video_id, "vid1");

        clear(&s, uid).await.unwrap();
        assert!(list(&s, uid, 0).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn history_isolated_per_user() {
        let (s, uid) = setup().await;
        let u2 = crate::users::create(&s, "frank").await.unwrap();

        add(&s, uid, "vid1", "T1", None, None, None, None, 0).await.unwrap();
        add(&s, u2.id, "vid2", "T2", None, None, None, None, 0).await.unwrap();

        assert_eq!(list(&s, uid, 0).await.unwrap().len(), 1);
        assert_eq!(list(&s, u2.id, 0).await.unwrap().len(), 1);
    }
}
