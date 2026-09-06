//! `yoube-filter`: three-layer content filtering support (spec §7).
//!
//! - L2 (in-app ABP): [`engine::FilterEngine`] — the `adblock` crate behind a
//!   `Send + Sync` worker thread (the engine itself is `!Send` in 0.9).
//! - L3 (player): [`sponsorblock`] skip segments + [`dearrow`] branding.
//!
//! L1 (system hosts) lives in `yoube-dns`. The `FilterService` trait that
//! exposes all of this to Tauri lives in `yoube-core`.

pub mod dearrow;
pub mod engine;
pub mod lists;
pub mod model;
pub mod sponsorblock;

pub use engine::{FilterEngine, RequestKind};
pub use model::{Branding, Segment, SegmentCategory};
