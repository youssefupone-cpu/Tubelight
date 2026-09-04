use serde::{Deserialize, Serialize};
use sqlx::Row;
use yoube_error::AppResult;

use crate::Storage;

/// A YouTube channel the user is subscribed to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub channel_id: String,
    pub title: String,
    pub thumb_url: Option<String>,
}

/// List all subscriptions for `user_id`, newest first.
pub async fn list(s: &Storage, user_id: i64) -> AppResult<Vec<Subscription>> {
    let rows = sqlx::query(
        "SELECT channel_id, title, thumb_url FROM subscriptions WHERE user_id = ? ORDER BY added_at DESC",
    )
    .bind(user_id)
    .fetch_all(&s.pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| Subscription {
            channel_id: r.get("channel_id"),
            title: r.get("title"),
            thumb_url: r.get("thumb_url"),
        })
        .collect())
}

/// Subscribe to a channel (upsert so re-subscribing just refreshes the title).
pub async fn subscribe(
    s: &Storage,
    user_id: i64,
    channel_id: &str,
    title: &str,
    thumb_url: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        "INSERT OR REPLACE INTO subscriptions (user_id, channel_id, title, thumb_url) VALUES (?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(channel_id)
    .bind(title)
    .bind(thumb_url)
    .execute(&s.pool)
    .await?;
    Ok(())
}

/// Unsubscribe from a channel.
pub async fn unsubscribe(s: &Storage, user_id: i64, channel_id: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM subscriptions WHERE user_id = ? AND channel_id = ?")
        .bind(user_id)
        .bind(channel_id)
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
        let u = crate::users::create(&s, "bob").await.unwrap();
        (s, u.id)
    }

    #[tokio::test]
    async fn subscribe_unsubscribe_lifecycle() {
        let (s, uid) = setup().await;

        subscribe(&s, uid, "UC123", "Test Channel", Some("https://img/t.png"))
            .await
            .unwrap();

        let subs = list(&s, uid).await.unwrap();
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].channel_id, "UC123");
        assert_eq!(subs[0].title, "Test Channel");

        unsubscribe(&s, uid, "UC123").await.unwrap();
        assert!(list(&s, uid).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn subscribe_isolated_per_user() {
        let (s, uid) = setup().await;
        let u2 = crate::users::create(&s, "carol").await.unwrap();

        subscribe(&s, uid, "UC1", "Ch A", None).await.unwrap();
        subscribe(&s, u2.id, "UC2", "Ch B", None).await.unwrap();

        assert_eq!(list(&s, uid).await.unwrap().len(), 1);
        assert_eq!(list(&s, u2.id).await.unwrap().len(), 1);
    }
}
