//! yoube-core: the service layer that powers the desktop app.

extern crate self as yoube_core;

pub mod context;
pub mod error;
pub mod services;
pub use context::AppContext;
pub use error::{AppError, AppResult};
// Re-export the boundary types the Tauri shell returns from account commands,
// so the shell depends only on `yoube-core` (not directly on `yoube-storage`).
pub use services::account::{ChannelRef, HistoryEntryView, Playlist as AccountPlaylist};
pub use yoube_storage::users::UserProfile;
