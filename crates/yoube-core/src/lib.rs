//! yoube-core: the service layer that powers the desktop app.

extern crate self as yoube_core;

pub mod context;
pub mod error;
pub mod services;
pub use context::AppContext;
pub use error::{AppError, AppResult};
