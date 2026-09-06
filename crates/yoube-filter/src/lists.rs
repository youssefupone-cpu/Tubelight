//! Filter-list sources: EasyList, EasyPrivacy, plus a small curated
//! YouTube-specific list shipped in-repo.
//!
//! Network fetching + disk caching live in `engine.rs`; this module only
//! defines *what* to fetch and the built-in fallback rules used when the
//! network is unavailable (offline-first: the engine always boots, even with
//! zero fetched bytes).

/// Default remote ABP lists (spec §7 L2).
pub const DEFAULT_LIST_URLS: &[&str] = &[
    "https://easylist.to/easylist/easylist.txt",
    "https://easylist.to/easylist/easyprivacy.txt",
];

/// Minimal curated YouTube/tracker rules baked into the binary.
///
/// These are intentionally tiny (a few dozen bytes) — the full lists come
/// from `DEFAULT_LIST_URLS` at `init` time. The built-ins guarantee the
/// golden-file tests and offline boots have *something* to match against.
pub fn builtin_youtube_rules() -> &'static str {
    concat!(
        "||doubleclick.net^\n",
        "||googleadservices.com^\n",
        "||googlesyndication.com^\n",
        "||google-analytics.com^\n",
        "||youtube.com/api/stats/ads^\n",
        "@@||youtube.com/api/stats/playback^\n",
    )
}

/// Split raw list text into individual rule lines, dropping comments,
/// element-hiding rules (`##`), and blank lines (network matching only).
pub fn rule_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('!') && !l.starts_with('#'))
        .filter(|l| !l.contains("##") && !l.contains("#@#"))
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_rules_have_expected_entries() {
        let rules = builtin_youtube_rules();
        assert!(rules.contains("||doubleclick.net^"));
        assert!(rules.contains("@@||youtube.com/api/stats/playback^"));
    }

    #[test]
    fn rule_lines_drops_comments_and_cosmetic() {
        let out = rule_lines("! comment\n||ads.example^\n##.ad\n\n@@||ok.example^\n");
        assert_eq!(out, vec!["||ads.example^", "@@||ok.example^"]);
    }
}
