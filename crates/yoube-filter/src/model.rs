//! Shared filter types: SponsorBlock segments + DeArrow branding.
//!
//! These cross the Tauri FFI boundary, so they stay `Serialize`/`Deserialize`
//! with an offline-gated `specta::Type` derive (same pattern as
//! `yoube-yt-dlp::model`).

use serde::{Deserialize, Serialize};

/// Segment categories the player can auto-skip (subset of the SponsorBlock
/// taxonomy the UI exposes as checkboxes).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum SegmentCategory {
    Sponsor,
    Intro,
    Outro,
    SelfPromo,
    Preview,
    MusicOfftopic,
    Filler,
    Interaction,
    PoiHighlight,
}

impl SegmentCategory {
    /// SponsorBlock API slug for this category.
    pub fn slug(self) -> &'static str {
        match self {
            SegmentCategory::Sponsor => "sponsor",
            SegmentCategory::Intro => "intro",
            SegmentCategory::Outro => "outro",
            SegmentCategory::SelfPromo => "selfpromo",
            SegmentCategory::Preview => "preview",
            SegmentCategory::MusicOfftopic => "music_offtopic",
            SegmentCategory::Filler => "filler",
            SegmentCategory::Interaction => "interaction_reminder",
            SegmentCategory::PoiHighlight => "poi_highlight",
        }
    }

    /// All skippable categories enabled by default.
    pub fn defaults() -> &'static [SegmentCategory] {
        &[
            SegmentCategory::Sponsor,
            SegmentCategory::Intro,
            SegmentCategory::Outro,
            SegmentCategory::SelfPromo,
            SegmentCategory::Preview,
            SegmentCategory::MusicOfftopic,
            SegmentCategory::Filler,
        ]
    }
}

/// A single skippable time range inside a video.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Segment {
    pub category: SegmentCategory,
    pub start_s: f32,
    pub end_s: f32,
    pub uuid: String,
}

/// Crowd-sourced title/thumbnail replacement (DeArrow).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Branding {
    pub title: Option<String>,
    pub thumbnail_url: Option<String>,
}
