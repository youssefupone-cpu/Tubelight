//! StevenBlack hosts source: fetch + parse the consolidated blocklist.
//!
//! The list format is classic hosts lines (`0.0.0.0 <domain>` plus comments).
//! Only bare domains are extracted; the caller decides how to render them.

use std::time::Duration;
use yoube_error::{AppError, AppResult};

/// Consolidated StevenBlack hosts (hosts + ads + malware + fakenews +
/// gambling), per spec §7 L1.
pub const STEVENBLACK_URL: &str =
    "https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts";

/// Fetch the raw list body with a bounded timeout.
pub async fn fetch_hosts(http: &reqwest::Client) -> AppResult<String> {
    let resp = http
        .get(STEVENBLACK_URL)
        .timeout(Duration::from_secs(30))
        .send()
        .await?;
    let status = resp.status();
    if !status.is_success() {
        return Err(AppError::Internal(anyhow::anyhow!(
            "hosts list fetch failed: {STEVENBLACK_URL} -> {status}"
        )));
    }
    Ok(resp.text().await?)
}

/// Extract bare domains from hosts-file text.
///
/// Skips comments, blank lines, `localhost`/`broadcasthost` loopback aliases,
/// and anything that is not a plausible DNS name.
pub fn parse_domains(text: &str) -> Vec<String> {
    const LOCAL: &[&str] = &[
        "localhost",
        "localhost.localdomain",
        "local",
        "broadcasthost",
        "ip6-localhost",
        "ip6-loopback",
    ];
    let mut out = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Strip inline comments: `0.0.0.0 example.com # tracker`.
        let body = line.split('#').next().unwrap_or("").trim();
        let mut parts = body.split_whitespace();
        let ip = parts.next().unwrap_or("");
        if ip != "0.0.0.0" && ip != "127.0.0.1" && ip != "::1" {
            continue;
        }
        for name in parts {
            let name = name.trim().trim_end_matches('.').to_lowercase();
            if name.is_empty()
                || name.contains(' ')
                || name.contains('/')
                || LOCAL.contains(&name.as_str())
            {
                continue;
            }
            // Plausible DNS name: letters/digits/hyphen/dots, at least one dot.
            if !name.contains('.')
                || !name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
            {
                continue;
            }
            out.push(name);
        }
    }
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sample_hosts() {
        let text = "# comment\n127.0.0.1 localhost\n0.0.0.0 ads.example.com # tracker\n0.0.0.0 multi.a.com multi.b.com\n1.2.3.4 not-a-block entry\n";
        assert_eq!(
            parse_domains(text),
            vec!["ads.example.com", "multi.a.com", "multi.b.com"]
        );
    }

    #[test]
    fn drops_non_dns_tokens() {
        assert!(parse_domains("0.0.0.0 not_a_domain\n").is_empty());
        assert!(parse_domains("0.0.0.0 localhost\n").is_empty());
    }
}
