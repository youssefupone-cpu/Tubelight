use serde::{Deserialize, Serialize};
use sqlx::Row;
use yoube_error::{AppError, AppResult};

use crate::Storage;

/// A user-created or special playlist.
///
/// `is_watch_later` and `is_liked` flag the two built-in smart playlists; the
/// rest are user-managed.  Both booleans can be false (a normal playlist).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub is_watch_later: bool,
    pub is_liked: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// One video inside a playlist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistItem {
    pub position: i64,
    pub video_id: String,
    pub title: String,
    pub channel_id: Option<String>,
    pub channel_title: Option<String>,
    pub duration_s: Option<i64>,
    pub thumb_url: Option<String>,
}

/// List all playlists for a user, excluding the built-in watch-later / liked
/// smart playlists (use `get_watch_later` / `get_liked` for those).
pub async fn list(s: &Storage, user_id: i64) -> AppResult<Vec<Playlist>> {
    let rows = sqlx::query(
        "SELECT id, user_id, title, description, is_watch_later, is_liked, created_at, updated_at \
         FROM playlists WHERE user_id = ? AND is_watch_later = 0 AND is_liked = 0 \
         ORDER BY updated_at DESC",
    )
    .bind(user_id)
    .fetch_all(&s.pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| Playlist {
            id: r.get("id"),
            user_id: r.get("user_id"),
            title: r.get("title"),
            description: r.get("description"),
            is_watch_later: r.get::<bool, _>("is_watch_later"),
            is_liked: r.get::<bool, _>("is_liked"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
        .collect())
}

/// Create a new (non-special) playlist and return its id.
pub async fn create(
    s: &Storage,
    user_id: i64,
    title: &str,
    description: Option<&str>,
) -> AppResult<i64> {
    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO playlists (user_id, title, description) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(user_id)
    .bind(title)
    .bind(description)
    .fetch_one(&s.pool)
    .await?;
    Ok(id)
}

/// Update title/description of a playlist owned by `user_id`.
pub async fn update(
    s: &Storage,
    user_id: i64,
    id: i64,
    title: &str,
    description: Option<&str>,
) -> AppResult<()> {
    let affected = sqlx::query(
        "UPDATE playlists SET title = ?, description = ?, updated_at = datetime('now') \
         WHERE id = ? AND user_id = ?",
    )
    .bind(title)
    .bind(description)
    .bind(id)
    .bind(user_id)
    .execute(&s.pool)
    .await?
    .rows_affected();
    if affected == 0 {
        return Err(AppError::NotFound(format!("playlist {id} not found")));
    }
    Ok(())
}

/// Delete a playlist (items cascade via `ON DELETE`).
pub async fn delete(s: &Storage, playlist_id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM playlists WHERE id = ?")
        .bind(playlist_id)
        .execute(&s.pool)
        .await?;
    Ok(())
}

/// Return all items in `playlist_id`, ordered by position.
pub async fn items(s: &Storage, playlist_id: i64) -> AppResult<Vec<PlaylistItem>> {
    let rows = sqlx::query(
        "SELECT position, video_id, title, channel_id, channel_title, duration_s, thumb_url \
         FROM playlist_items WHERE playlist_id = ? ORDER BY position",
    )
    .bind(playlist_id)
    .fetch_all(&s.pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| PlaylistItem {
            position: r.get("position"),
            video_id: r.get("video_id"),
            title: r.get("title"),
            channel_id: r.get("channel_id"),
            channel_title: r.get("channel_title"),
            duration_s: r.get("duration_s"),
            thumb_url: r.get("thumb_url"),
        })
        .collect())
}

/// Append a video to a playlist at the next available position.
#[allow(clippy::too_many_arguments)]
pub async fn add_item(
    s: &Storage,
    playlist_id: i64,
    video_id: &str,
    title: &str,
    channel_id: Option<&str>,
    channel_title: Option<&str>,
    duration_s: Option<i64>,
    thumb_url: Option<&str>,
) -> AppResult<()> {
    let next_pos: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(position) + 1, 0) FROM playlist_items WHERE playlist_id = ?",
    )
    .bind(playlist_id)
    .fetch_one(&s.pool)
    .await?;

    sqlx::query(
        "INSERT INTO playlist_items \
         (playlist_id, position, video_id, title, channel_id, channel_title, duration_s, thumb_url) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(playlist_id)
    .bind(next_pos)
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

/// Remove a video from a playlist by position or by video_id.
pub async fn remove_item(s: &Storage, playlist_id: i64, video_id: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM playlist_items WHERE playlist_id = ? AND video_id = ?")
        .bind(playlist_id)
        .bind(video_id)
        .execute(&s.pool)
        .await?;
    Ok(())
}

/// Look up the special "watch later" playlist for a user.
pub async fn get_watch_later(s: &Storage, user_id: i64) -> AppResult<i64> {
    sqlx::query_scalar("SELECT id FROM playlists WHERE user_id = ? AND is_watch_later = 1")
        .bind(user_id)
        .fetch_optional(&s.pool)
        .await?
        .ok_or(AppError::NotFound("watch-later playlist not found".into()))
}

/// Look up the special "liked" playlist for a user.
pub async fn get_liked(s: &Storage, user_id: i64) -> AppResult<i64> {
    sqlx::query_scalar("SELECT id FROM playlists WHERE user_id = ? AND is_liked = 1")
        .bind(user_id)
        .fetch_optional(&s.pool)
        .await?
        .ok_or(AppError::NotFound("liked playlist not found".into()))
}

/// Create the "watch later" or "liked" smart playlist for a user.
/// The `is_watch_later` / `is_liked` flags are mutually exclusive.
pub async fn create_special(
    s: &Storage,
    user_id: i64,
    title: &str,
    is_watch_later: bool,
    is_liked: bool,
) -> AppResult<i64> {
    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO playlists (user_id, title, description, is_watch_later, is_liked) \
         VALUES (?, ?, NULL, ?, ?) RETURNING id",
    )
    .bind(user_id)
    .bind(title)
    .bind(is_watch_later)
    .bind(is_liked)
    .fetch_one(&s.pool)
    .await?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> (Storage, i64) {
        let tmp = Box::leak(Box::new(tempfile::tempdir().unwrap()));
        let s = Storage::open(&tmp.path().join("test.db")).await.unwrap();
        s.migrate().await.unwrap();
        let u = crate::users::create(&s, "dave").await.unwrap();
        (s, u.id)
    }

    #[tokio::test]
    async fn create_list_delete_playlist() {
        let (s, uid) = setup().await;
        let pid = create(&s, uid, "My List", Some("desc")).await.unwrap();
        let p = list(&s, uid).await.unwrap();
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].id, pid);
        assert_eq!(p[0].title, "My List");

        delete(&s, pid).await.unwrap();
        assert!(list(&s, uid).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn add_and_remove_items() {
        let (s, uid) = setup().await;
        let pid = create(&s, uid, "WL", None).await.unwrap();
        add_item(&s, pid, "vid1", "First", None, None, None, None).await.unwrap();
        add_item(&s, pid, "vid2", "Second", None, None, None, None).await.unwrap();
        let playlist_items = items(&s, pid).await.unwrap();
        assert_eq!(playlist_items.len(), 2);
        assert_eq!(playlist_items[0].position, 0);
        assert_eq!(playlist_items[1].position, 1);

        remove_item(&s, pid, "vid1").await.unwrap();
        assert_eq!(items(&s, pid).await.unwrap().len(), 1);
    }
}
