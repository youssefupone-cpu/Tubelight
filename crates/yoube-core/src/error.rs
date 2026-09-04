//! `AppError`/`AppResult` are defined in the dedicated `yoube-error` crate to
//! avoid a `yoube-core` <-> `yoube-yt-dlp` package cycle (see `yoube-error`'s
//! lib docs). They are re-exported here so the historical
//! `yoube_core::{AppError, AppResult}` paths keep working.

pub use yoube_error::{AppError, AppResult};
