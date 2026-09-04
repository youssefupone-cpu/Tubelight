//! yoube-core: the service layer that powers the desktop app.

pub mod context;
pub mod error;
pub use context::AppContext;
pub use error::{AppError, AppResult};
