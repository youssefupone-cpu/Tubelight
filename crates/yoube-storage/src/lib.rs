//! yoube-storage: persistent storage layer backed by SQLite.
//!
//! Leaf crate that depends only on `yoube-error` (for `AppResult`/`AppError`),
//! following the DAG discipline documented in `yoube-core/Cargo.toml`.

pub mod history;
pub mod likes;
pub mod playlists;
pub mod subscriptions;
pub mod users;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::sqlite::SqliteJournalMode;
use std::path::Path;
use std::str::FromStr;
use yoube_error::{AppError, AppResult};

/// Shared SQLite-backed store for all per-user data.
#[derive(Clone)]
pub struct Storage {
    pub pool: sqlx::SqlitePool,
}

impl Storage {
    /// Open (or create) the database at `path`.  The caller is responsible for
    /// resolving the path (e.g. via Tauri's `app_data_dir`); this crate stays
    /// free of a `directories` dependency for offline build compatibility.
    pub async fn open(path: &Path) -> AppResult<Self> {
        let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(opts)
            .await?;
        Ok(Self { pool })
    }

    /// Run embedded migrations against the pool.
    ///
    /// Uses `sqlx::migrate!` which folds the SQL files in `migrations/` into the
    /// binary at compile time — no `sqlx-data.json` or live DB required.  This
    /// is safe in fully-offline builds because `migrate!` only calls
    /// `include_str!` internally; compile-time SQL *verification* (needed for
    /// `query!`/`query_as!` macros) is intentionally avoided everywhere in this
    /// crate.
    pub async fn migrate(&self) -> AppResult<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        Ok(())
    }
}
