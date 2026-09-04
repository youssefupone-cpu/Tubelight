use serde::{Deserialize, Serialize};
use sqlx::Row;
use yoube_error::{AppError, AppResult};

use crate::Storage;

/// A virtual user account (not tied to a YouTube account).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: i64,
    pub name: String,
    pub avatar_path: Option<String>,
}

/// Return the most-recently-used user, or `NotFound` if none exists.
pub async fn current(s: &Storage) -> AppResult<UserProfile> {
    let row = sqlx::query("SELECT id, name, avatar_path FROM users ORDER BY last_used_at DESC LIMIT 1")
        .fetch_optional(&s.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("no current user".into()))?;
    Ok(UserProfile {
        id: row.get("id"),
        name: row.get("name"),
        avatar_path: row.get("avatar_path"),
    })
}

/// Create a new user and return its profile.
pub async fn create(s: &Storage, name: &str) -> AppResult<UserProfile> {
    let id = sqlx::query_scalar::<_, i64>("INSERT INTO users (name) VALUES (?) RETURNING id")
        .bind(name)
        .fetch_one(&s.pool)
        .await?;
    Ok(UserProfile {
        id,
        name: name.into(),
        avatar_path: None,
    })
}

/// Mark `id` as the current user (bumps `last_used_at`).
pub async fn switch(s: &Storage, id: i64) -> AppResult<()> {
    let affected = sqlx::query("UPDATE users SET last_used_at = strftime('%Y-%m-%d %H:%M:%S.%f', 'now') WHERE id = ?")
        .bind(id)
        .execute(&s.pool)
        .await?
        .rows_affected();
    if affected == 0 {
        return Err(AppError::NotFound(format!("user {id} not found")));
    }
    Ok(())
}

/// Delete a user.  Cascades to subscriptions, playlists, history, likes.
pub async fn delete(s: &Storage, id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(&s.pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_and_current_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let s = Storage::open(&tmp.path().join("test.db")).await.unwrap();
        s.migrate().await.unwrap();

        let u = create(&s, "alice").await.unwrap();
        assert_eq!(u.name, "alice");
        assert!(u.avatar_path.is_none());

        let fetched = current(&s).await.unwrap();
        assert_eq!(fetched.id, u.id);
        assert_eq!(fetched.name, "alice");
    }

    #[tokio::test]
    async fn switch_missing_user_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let s = Storage::open(&tmp.path().join("test.db")).await.unwrap();
        s.migrate().await.unwrap();
        assert!(switch(&s, 999).await.is_err());
    }
}
