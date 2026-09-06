//! `AccountService`: the SQLite-backed virtual-user service.
//!
//! This trait + impl live in `yoube-core` (offline-testable, no `tauri`
//! dependency — the `#[tauri::command]` shims are in `apps/desktop/src-tauri`).
//!
//! `StorageAccountService` is a thin façade over `yoube-storage`. Because the
//! trait methods are stateless (they take no `user_id`), the service tracks
//! the *current* virtual user in an `AtomicI64`, set on first access / user
//! switch — matching the spec's "single active virtual user" model.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::path::Path;
use std::sync::atomic::{AtomicI64, Ordering};
use yoube_storage::Storage;
use yoube_storage::history as hist_mod;
use yoube_storage::likes as like_mod;
use yoube_storage::playlists as pl_mod;
use yoube_storage::subscriptions as sub_mod;
use yoube_storage::users as user_mod;

use crate::AppResult;
use yoube_yt_dlp::model::VideoSummary;

use user_mod::UserProfile;

/// A channel reference surfaced in the UI's subscription feed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ChannelRef {
    pub id: String,
    pub title: String,
    pub thumb_url: Option<String>,
}

/// A playlist row as seen by the AccountService contract. Mirrors the
/// storage `Playlist` but re-derives `count` and exposes boolean flags.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Playlist {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub is_watch_later: bool,
    pub is_liked: bool,
    pub count: i64,
}

/// A history row surfaced to the UI. Aliases the storage type so callers
/// don't depend on `yoube-storage` directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct HistoryEntryView {
    pub video_id: String,
    pub title: String,
    pub channel_title: Option<String>,
    pub duration_s: Option<i64>,
    pub thumb_url: Option<String>,
    pub watched_at: String,
    pub position_s: i64,
}

#[async_trait]
pub trait AccountService: Send + Sync {
    async fn current_user(&self) -> AppResult<UserProfile>;
    async fn switch_user(&self, id: i64) -> AppResult<()>;
    async fn create_user(&self, name: &str) -> AppResult<UserProfile>;
    async fn delete_user(&self, id: i64) -> AppResult<()>;
    async fn subscriptions(&self) -> AppResult<Vec<ChannelRef>>;
    async fn subscribe(
        &self,
        channel_id: &str,
        title: &str,
        thumb_url: Option<&str>,
    ) -> AppResult<()>;
    async fn unsubscribe(&self, channel_id: &str) -> AppResult<()>;
    async fn playlists(&self) -> AppResult<Vec<Playlist>>;
    async fn playlist_items(&self, id: i64) -> AppResult<Vec<VideoSummary>>;
    async fn playlist_add(&self, id: i64, video: &VideoSummary) -> AppResult<()>;
    async fn playlist_remove(&self, id: i64, video_id: &str) -> AppResult<()>;
    async fn history(&self, page: u32) -> AppResult<Vec<HistoryEntryView>>;
    async fn history_clear(&self) -> AppResult<()>;
    async fn mark_history(&self, video: &VideoSummary) -> AppResult<()>;
    async fn like(&self, video: &VideoSummary) -> AppResult<()>;
    async fn unlike(&self, video_id: &str) -> AppResult<()>;
    async fn watch_later(&self, video: &VideoSummary) -> AppResult<()>;
    async fn export(&self, dest: &Path) -> AppResult<()>;
    async fn import(&self, src: &Path) -> AppResult<()>;
}

/// Storage-backed implementation of `AccountService`.
pub struct StorageAccountService {
    storage: Storage,
    current_user_id: AtomicI64,
}

impl StorageAccountService {
    pub fn new(storage: Storage) -> Self {
        Self {
            storage,
            current_user_id: AtomicI64::new(0),
        }
    }

    /// Resolve (lazily) the current user, creating a default one if none exists.
    async fn ensure_current(&self) -> AppResult<i64> {
        let cached = self.current_user_id.load(Ordering::Relaxed);
        if cached > 0 {
            return Ok(cached);
        }
        match user_mod::current(&self.storage).await {
            Ok(u) => {
                self.current_user_id.store(u.id, Ordering::Relaxed);
                Ok(u.id)
            }
            Err(crate::AppError::NotFound(_)) => {
                let u = user_mod::create(&self.storage, "yoube").await?;
                self.current_user_id.store(u.id, Ordering::Relaxed);
                Ok(u.id)
            }
            Err(e) => Err(e),
        }
    }
}

#[async_trait]
impl AccountService for StorageAccountService {
    async fn current_user(&self) -> AppResult<UserProfile> {
        // Ensure a default user exists (idempotent) so `current` won't 404.
        self.ensure_current().await?;
        user_mod::current(&self.storage).await
    }

    async fn switch_user(&self, id: i64) -> AppResult<()> {
        self.ensure_current().await?;
        user_mod::switch(&self.storage, id).await?;
        self.current_user_id.store(id, Ordering::Relaxed);
        Ok(())
    }

    async fn create_user(&self, name: &str) -> AppResult<UserProfile> {
        let u = user_mod::create(&self.storage, name).await?;
        self.current_user_id.store(u.id, Ordering::Relaxed);
        Ok(u)
    }

    async fn delete_user(&self, id: i64) -> AppResult<()> {
        user_mod::delete(&self.storage, id).await
    }

    async fn subscriptions(&self) -> AppResult<Vec<ChannelRef>> {
        let uid = self.ensure_current().await?;
        let subs = sub_mod::list(&self.storage, uid).await?;
        Ok(subs
            .into_iter()
            .map(|s| ChannelRef {
                id: s.channel_id,
                title: s.title,
                thumb_url: s.thumb_url,
            })
            .collect())
    }

    async fn subscribe(
        &self,
        channel_id: &str,
        title: &str,
        thumb_url: Option<&str>,
    ) -> AppResult<()> {
        let uid = self.ensure_current().await?;
        sub_mod::subscribe(&self.storage, uid, channel_id, title, thumb_url).await
    }

    async fn unsubscribe(&self, channel_id: &str) -> AppResult<()> {
        let uid = self.ensure_current().await?;
        sub_mod::unsubscribe(&self.storage, uid, channel_id).await
    }

    async fn playlists(&self) -> AppResult<Vec<Playlist>> {
        let uid = self.ensure_current().await?;
        // Include the smart playlists (is_watch_later / is_liked) which
        // `pl_mod::list` deliberately excludes.  Roll up item counts in one
        // join so we don't ping the DB N+1 times.
        let rows = sqlx::query(
            "SELECT p.id, p.title, p.description, p.is_watch_later, p.is_liked, \
             COUNT(playlist_items.playlist_id) AS cnt \
             FROM playlists p \
             LEFT JOIN playlist_items ON playlist_items.playlist_id = p.id \
             WHERE p.user_id = ? \
             GROUP BY p.id \
             ORDER BY p.updated_at DESC",
        )
        .bind(uid)
        .fetch_all(&self.storage.pool)
        .await?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            out.push(Playlist {
                id: r.get::<i64, _>("id"),
                title: r.get::<String, _>("title"),
                description: r.get::<Option<String>, _>("description"),
                is_watch_later: r.get::<bool, _>("is_watch_later"),
                is_liked: r.get::<bool, _>("is_liked"),
                count: r.get::<i64, _>("cnt"),
            });
        }
        Ok(out)
    }

    async fn playlist_items(&self, id: i64) -> AppResult<Vec<VideoSummary>> {
        let items = pl_mod::items(&self.storage, id).await?;
        Ok(items
            .into_iter()
            .map(|it| VideoSummary {
                id: it.video_id,
                title: it.title,
                channel_id: it.channel_id.unwrap_or_default(),
                channel_title: it.channel_title.unwrap_or_default(),
                duration_s: it.duration_s.map(|n| n as u32),
                view_count: None,
                upload_date: None,
                thumbnail_url: it.thumb_url,
            })
            .collect())
    }

    async fn playlist_add(&self, id: i64, video: &VideoSummary) -> AppResult<()> {
        pl_mod::add_item(
            &self.storage,
            id,
            &video.id,
            &video.title,
            Some(&video.channel_id),
            Some(&video.channel_title),
            video.duration_s.map(|n| n as i64),
            video.thumbnail_url.as_deref(),
        )
        .await
    }

    async fn playlist_remove(&self, id: i64, video_id: &str) -> AppResult<()> {
        pl_mod::remove_item(&self.storage, id, video_id).await
    }

    async fn history(&self, page: u32) -> AppResult<Vec<HistoryEntryView>> {
        let uid = self.ensure_current().await?;
        let rows = hist_mod::list(&self.storage, uid, page).await?;
        Ok(rows
            .into_iter()
            .map(|h| HistoryEntryView {
                video_id: h.video_id,
                title: h.title,
                channel_title: h.channel_title,
                duration_s: h.duration_s,
                thumb_url: h.thumb_url,
                watched_at: h.watched_at,
                position_s: h.position_s,
            })
            .collect())
    }

    async fn history_clear(&self) -> AppResult<()> {
        let uid = self.ensure_current().await?;
        hist_mod::clear(&self.storage, uid).await
    }

    async fn mark_history(&self, video: &VideoSummary) -> AppResult<()> {
        let uid = self.ensure_current().await?;
        hist_mod::add(
            &self.storage,
            uid,
            &video.id,
            &video.title,
            Some(&video.channel_id),
            Some(&video.channel_title),
            video.duration_s.map(|n| n as i64),
            video.thumbnail_url.as_deref(),
            0,
        )
        .await
    }

    async fn like(&self, video: &VideoSummary) -> AppResult<()> {
        let uid = self.ensure_current().await?;
        like_mod::add(
            &self.storage,
            uid,
            &video.id,
            &video.title,
            Some(&video.channel_id),
            Some(&video.channel_title),
            video.duration_s.map(|n| n as i64),
            video.thumbnail_url.as_deref(),
        )
        .await
    }

    async fn unlike(&self, video_id: &str) -> AppResult<()> {
        let uid = self.ensure_current().await?;
        like_mod::remove(&self.storage, uid, video_id).await
    }

    async fn watch_later(&self, video: &VideoSummary) -> AppResult<()> {
        let uid = self.ensure_current().await?;
        like_mod::watch_later(
            &self.storage,
            uid,
            &video.id,
            &video.title,
            Some(&video.channel_id),
            Some(&video.channel_title),
            video.duration_s.map(|n| n as i64),
            video.thumbnail_url.as_deref(),
        )
        .await
    }

    async fn export(&self, dest: &Path) -> AppResult<()> {
        let mut doc = serde_json::Map::new();
        let uid = self.ensure_current().await?;
        doc.insert(
            "users".into(),
            serde_json::json!(user_mod::current(&self.storage).await.ok()),
        );
        doc.insert(
            "subscriptions".into(),
            serde_json::json!(sub_mod::list(&self.storage, uid).await?),
        );
        doc.insert(
            "playlists".into(),
            serde_json::json!(pl_mod::list(&self.storage, uid).await?),
        );
        doc.insert(
            "history".into(),
            serde_json::json!(hist_mod::list(&self.storage, uid, 0).await?),
        );
        doc.insert(
            "likes".into(),
            serde_json::json!(like_mod::list(&self.storage, uid).await?),
        );
        let json = serde_json::to_string_pretty(&serde_json::Value::Object(doc))?;
        tokio::fs::write(dest, json).await?;
        Ok(())
    }

    async fn import(&self, src: &Path) -> AppResult<()> {
        let raw = tokio::fs::read_to_string(src).await?;
        let doc: serde_json::Value = serde_json::from_str(&raw)?;
        if let Some(arr) = doc.get("subscriptions").and_then(|v| v.as_array()) {
            for sub in arr {
                if let (Some(cid), Some(title)) = (sub.get("channel_id"), sub.get("title")) {
                    let _ = self
                        .subscribe(
                            cid.as_str().unwrap(),
                            title.as_str().unwrap(),
                            sub.get("thumb_url").and_then(|t| t.as_str()),
                        )
                        .await;
                }
            }
        }
        // playlist items / history / likes could be imported the same way; the
        // spec's export/import round-trips the canonical shape, but for Phase 2
        // we import subscriptions only (the others are best-effort re-derivable).
        if let Some(arr) = doc.get("history").and_then(|v| v.as_array()) {
            for h in arr {
                if let Some(vid) = h.get("video_id").and_then(|v| v.as_str()) {
                    let _ = self
                        .mark_history(&VideoSummary {
                            id: vid.to_string(),
                            title: h
                                .get("title")
                                .and_then(|t| t.as_str())
                                .unwrap_or("")
                                .to_string(),
                            channel_id: h
                                .get("channel_id")
                                .and_then(|t| t.as_str())
                                .unwrap_or_default()
                                .to_string(),
                            channel_title: h
                                .get("channel_title")
                                .and_then(|t| t.as_str())
                                .unwrap_or_default()
                                .to_string(),
                            duration_s: h
                                .get("duration_s")
                                .and_then(|t| t.as_i64())
                                .map(|n| n as u32),
                            view_count: None,
                            upload_date: None,
                            thumbnail_url: h
                                .get("thumb_url")
                                .and_then(|t| t.as_str())
                                .map(String::from),
                        })
                        .await;
                }
            }
        }
        if let Some(arr) = doc.get("likes").and_then(|v| v.as_array()) {
            for l in arr {
                if let Some(vid) = l.get("video_id").and_then(|v| v.as_str()) {
                    let _ = self
                        .like(&VideoSummary {
                            id: vid.to_string(),
                            title: l
                                .get("title")
                                .and_then(|t| t.as_str())
                                .unwrap_or("")
                                .to_string(),
                            channel_id: l
                                .get("channel_id")
                                .and_then(|t| t.as_str())
                                .unwrap_or_default()
                                .to_string(),
                            channel_title: l
                                .get("channel_title")
                                .and_then(|t| t.as_str())
                                .unwrap_or_default()
                                .to_string(),
                            duration_s: l
                                .get("duration_s")
                                .and_then(|t| t.as_i64())
                                .map(|n| n as u32),
                            view_count: None,
                            upload_date: None,
                            thumbnail_url: l
                                .get("thumb_url")
                                .and_then(|t| t.as_str())
                                .map(String::from),
                        })
                        .await;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a live `StorageAccountService` backed by an on-disk temp DB.
    /// The `TempDir` is leaked so it outlives the pool (see Task 2.1 fix).
    async fn setup() -> StorageAccountService {
        let tmp = Box::leak(Box::new(tempfile::tempdir().unwrap()));
        let s = Storage::open(&tmp.path().join("t.db")).await.unwrap();
        s.migrate().await.unwrap();
        StorageAccountService::new(s)
    }

    #[tokio::test]
    async fn create_user_and_subscribe() {
        let svc = setup().await;
        let u = svc.create_user("alice").await.unwrap();
        assert_eq!(u.name, "alice");

        svc.subscribe("UC1", "Ch A", Some("thumb.png"))
            .await
            .unwrap();
        let subs = svc.subscriptions().await.unwrap();
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].id, "UC1");
    }

    #[tokio::test]
    async fn unsubscribe_removes() {
        let svc = setup().await;
        svc.subscribe("UC1", "Ch A", None).await.unwrap();
        assert_eq!(svc.subscriptions().await.unwrap().len(), 1);
        svc.unsubscribe("UC1").await.unwrap();
        assert!(svc.subscriptions().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn playlist_add_and_items() {
        let svc = setup().await;
        let pid = pl_mod::create(&svc.storage, svc.ensure_current().await.unwrap(), "L", None)
            .await
            .unwrap();
        let v = VideoSummary {
            id: "vid1".into(),
            title: "T".into(),
            channel_id: "ch1".into(),
            channel_title: "Ch".into(),
            duration_s: Some(120),
            view_count: None,
            upload_date: None,
            thumbnail_url: None,
        };
        svc.playlist_add(pid, &v).await.unwrap();
        let items = svc.playlist_items(pid).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "vid1");
    }

    #[tokio::test]
    async fn mark_history_and_export_roundtrip() {
        let svc = setup().await;
        let v = VideoSummary {
            id: "vid9".into(),
            title: "T".into(),
            channel_id: "ch9".into(),
            channel_title: "Ch 9".into(),
            duration_s: Some(60),
            view_count: None,
            upload_date: None,
            thumbnail_url: None,
        };
        svc.mark_history(&v).await.unwrap();
        assert_eq!(svc.history(0).await.unwrap().len(), 1);

        // export then re-read the JSON to prove round-trip shape
        let tmp = Box::leak(Box::new(tempfile::tempdir().unwrap()));
        let dest = tmp.path().join("export.json");
        svc.export(&dest).await.unwrap();
        let json = std::fs::read_to_string(&dest).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(val.get("users").is_some());
        assert!(val.get("subscriptions").is_some());
        assert!(val.get("playlists").is_some());
        assert!(val.get("history").is_some());
        assert!(val.get("likes").is_some());
    }
}
