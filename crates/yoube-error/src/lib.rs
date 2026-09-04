//! yoube-error: shared application error type.
//!
//! `AppError`/`AppResult` live here (not in `yoube-core`) so that the
//! `yoube-core` <-> `yoube-yt-dlp` graph has no package-level cycle:
//! `yoube-yt-dlp` depends on `yoube-error`, and `yoube-core` depends on both
//! `yoube-yt-dlp` and `yoube-error`. `yoube-core` re-exports these for
//! backwards-compatible paths (`yoube_core::AppError`).

use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("db error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("yt-dlp error: {0}")]
    YtDlp(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("internal: {0}")]
    Internal(#[from] anyhow::Error),
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serializes_to_string() {
        let e = AppError::NotFound("video".into());
        let s = serde_json::to_string(&e).unwrap();
        assert_eq!(s, "\"not found: video\"");
    }
    #[test]
    fn converts_from_io() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "x");
        let _: AppError = io.into();
    }
}
