//! System hosts writer (L1): install / uninstall / status / refresh.
//!
//! All paths are injected (`hosts_path`, `backup_path`) so tests run against
//! a temp dir. Production uses [`DnsBlock::production`]:
//! `/etc/hosts` on Unix, the System32 drivers path on Windows.
//!
//! Privilege model (spec §7, plan Task 4.3): writing the real hosts file
//! needs admin/root. This crate does NOT self-elevate — a permission error
//! surfaces as an `AppError` telling the user to re-run elevated
//! (`sudo`/run-as-admin). The Windows `runas` relaunch is a documented
//! follow-up; the write path itself is platform-correct (`atomic_write`
/// uses write-temp-plus-rename, which is atomic on NTFS too).
use std::path::{Path, PathBuf};
use yoube_error::{AppError, AppResult};

use crate::source::{fetch_hosts, parse_domains};

/// Begin marker for the block this crate manages.
pub const BEGIN_MARKER: &str = "# >>> yoube-managed >>>";
/// End marker for the block this crate manages.
pub const END_MARKER: &str = "# <<< yoube-managed <<<";

/// Installation state of the managed block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostsStatus {
    /// No managed block present.
    NotInstalled,
    /// Managed block present with `entries` blocked domains.
    Installed { entries: usize },
    /// Hosts file unreadable (with reason).
    Unknown(String),
}

/// Hosts-file manager.
#[derive(Debug, Clone)]
pub struct DnsBlock {
    hosts_path: PathBuf,
    backup_path: PathBuf,
}

impl DnsBlock {
    /// Build with explicit paths (tests + custom installs).
    pub fn new(hosts_path: PathBuf, backup_path: PathBuf) -> Self {
        Self {
            hosts_path,
            backup_path,
        }
    }

    /// Production paths for the current OS.
    pub fn production(backup_dir: &Path) -> Self {
        Self::new(default_hosts_path(), backup_dir.join("hosts.original"))
    }

    /// Read the current status without touching anything.
    pub fn status(&self) -> AppResult<HostsStatus> {
        let text = match std::fs::read_to_string(&self.hosts_path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(HostsStatus::NotInstalled);
            }
            Err(e) => return Ok(HostsStatus::Unknown(e.to_string())),
        };
        match extract_block(&text) {
            Some(block) => Ok(HostsStatus::Installed {
                entries: count_entries(&block),
            }),
            None => Ok(HostsStatus::NotInstalled),
        }
    }

    /// Install the blocklist: fetch → back up original → atomic write.
    ///
    /// Returns the number of blocked domains installed.
    pub async fn install(&self, http: &reqwest::Client) -> AppResult<usize> {
        let body = fetch_hosts(http).await?;
        let domains = parse_domains(&body);
        self.install_domains(&domains)
    }

    /// Install from an explicit domain list (tests + offline path).
    pub fn install_domains(&self, domains: &[String]) -> AppResult<usize> {
        // Back up the pristine file once — never overwrite an existing backup.
        if !self.backup_path.exists() {
            if let Some(parent) = self.backup_path.parent() {
                std::fs::create_dir_all(parent).map_err(elevate_hint)?;
            }
            if self.hosts_path.exists() {
                std::fs::copy(&self.hosts_path, &self.backup_path).map_err(elevate_hint)?;
            } else {
                std::fs::write(&self.backup_path, "").map_err(elevate_hint)?;
            }
        }
        let current = std::fs::read_to_string(&self.hosts_path).unwrap_or_default();
        let stripped = strip_block(&current);
        let mut next = stripped;
        if !next.is_empty() && !next.ends_with('\n') {
            next.push('\n');
        }
        next.push_str(&render_block(domains));
        atomic_write(&self.hosts_path, &next).map_err(elevate_hint)?;
        Ok(domains.len())
    }

    /// Remove the managed block, restoring the file to its pre-install bytes.
    ///
    /// Prefers the backup when it matches "current minus our block" (the
    /// normal case); otherwise just strips the block in place.
    pub fn uninstall(&self) -> AppResult<()> {
        let current = std::fs::read_to_string(&self.hosts_path).map_err(elevate_hint)?;
        let stripped = strip_block(&current);
        if self.backup_path.exists()
            && let Ok(backup) = std::fs::read_to_string(&self.backup_path)
            // The backup is authoritative only if it equals the file
            // without our block — otherwise an external tool edited hosts
            // meanwhile, and clobbering it would lose their changes.
            && normalize(&backup) == normalize(&stripped)
        {
            atomic_write(&self.hosts_path, &backup).map_err(elevate_hint)?;
            return Ok(());
        }
        // `stripped` may be missing its trailing newline; restore exactly one.
        let mut out = stripped;
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        atomic_write(&self.hosts_path, &out).map_err(elevate_hint)?;
        Ok(())
    }

    /// Re-fetch and replace the block. Only valid when installed.
    pub async fn refresh(&self, http: &reqwest::Client) -> AppResult<usize> {
        match self.status()? {
            HostsStatus::Installed { .. } => self.install(http).await,
            HostsStatus::NotInstalled => Err(AppError::Internal(anyhow::anyhow!(
                "cannot refresh: hosts block is not installed"
            ))),
            HostsStatus::Unknown(reason) => Err(AppError::Internal(anyhow::anyhow!(
                "cannot refresh: hosts file unreadable ({reason})"
            ))),
        }
    }
}

/// Default hosts path for the current OS.
pub fn default_hosts_path() -> PathBuf {
    #[cfg(windows)]
    {
        let root = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
        PathBuf::from(root).join(r"System32\drivers\etc\hosts")
    }
    #[cfg(not(windows))]
    {
        PathBuf::from("/etc/hosts")
    }
}

fn elevate_hint(e: std::io::Error) -> AppError {
    if e.kind() == std::io::ErrorKind::PermissionDenied {
        AppError::Internal(anyhow::anyhow!(
            "{e}; writing the system hosts file needs admin/root — re-run elevated (sudo / run as administrator)"
        ))
    } else {
        AppError::Internal(e.into())
    }
}

/// Extract the managed block (without markers), if present.
fn extract_block(text: &str) -> Option<String> {
    let start = text.find(BEGIN_MARKER)?;
    let end = text[start..].find(END_MARKER)?;
    Some(text[start + BEGIN_MARKER.len()..start + end].to_string())
}

fn count_entries(block: &str) -> usize {
    block
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .count()
}

fn strip_block(text: &str) -> String {
    let Some(start) = text.find(BEGIN_MARKER) else {
        return text.to_string();
    };
    // Also drop the full line containing the begin marker (and its newline).
    let line_start = text[..start].rfind('\n').map_or(0, |i| i + 1);
    let after_begin = &text[start..];
    let Some(end_rel) = after_begin.find(END_MARKER) else {
        return text[..line_start].to_string();
    };
    let end_abs = start + end_rel + END_MARKER.len();
    // Drop through the end of the end-marker line.
    let rest = text[end_abs..]
        .strip_prefix('\n')
        .unwrap_or(&text[end_abs..]);
    format!("{}{}", &text[..line_start], rest)
}

fn render_block(domains: &[String]) -> String {
    let mut s = String::from(BEGIN_MARKER);
    s.push('\n');
    s.push_str(
        "# yoube-managed ad/tracker block (StevenBlack). Do not edit between the markers.\n",
    );
    for d in domains {
        s.push_str("0.0.0.0 ");
        s.push_str(d);
        s.push('\n');
    }
    s.push_str(END_MARKER);
    s.push('\n');
    s
}

fn atomic_write(path: &Path, contents: &str) -> std::io::Result<()> {
    let tmp = path.with_extension("yoube-tmp");
    std::fs::write(&tmp, contents)?;
    std::fs::rename(&tmp, path)
}

fn normalize(s: &str) -> String {
    s.replace("\r\n", "\n").trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_hosts() -> String {
        "127.0.0.1 localhost\n::1 localhost\n".to_string()
    }

    fn setup() -> (tempfile::TempDir, DnsBlock) {
        let dir = tempfile::tempdir().unwrap();
        let hosts = dir.path().join("hosts");
        std::fs::write(&hosts, fixture_hosts()).unwrap();
        let dns = DnsBlock::new(hosts, dir.path().join("backup").join("hosts.original"));
        (dir, dns)
    }

    #[test]
    fn install_uninstall_round_trips_bytes() {
        let (_dir, dns) = setup();
        assert_eq!(dns.status().unwrap(), HostsStatus::NotInstalled);
        let domains = vec![
            "ads.example.com".to_string(),
            "tracker.example.org".to_string(),
        ];
        assert_eq!(dns.install_domains(&domains).unwrap(), 2);
        assert_eq!(dns.status().unwrap(), HostsStatus::Installed { entries: 2 });
        dns.uninstall().unwrap();
        assert_eq!(dns.status().unwrap(), HostsStatus::NotInstalled);
        let after = std::fs::read_to_string(&dns.hosts_path).unwrap();
        assert_eq!(after, fixture_hosts());
    }

    #[test]
    fn reinstall_does_not_clobber_backup() {
        let (_dir, dns) = setup();
        dns.install_domains(&["a.example.com".to_string()]).unwrap();
        let backup_once = std::fs::read_to_string(&dns.backup_path).unwrap();
        dns.install_domains(&["b.example.com".to_string()]).unwrap();
        let backup_twice = std::fs::read_to_string(&dns.backup_path).unwrap();
        assert_eq!(backup_once, backup_twice);
        assert_eq!(dns.status().unwrap(), HostsStatus::Installed { entries: 1 });
    }

    #[test]
    fn uninstall_without_install_is_noop_on_clean_file() {
        let (_dir, dns) = setup();
        dns.uninstall().unwrap();
        assert_eq!(
            std::fs::read_to_string(&dns.hosts_path).unwrap(),
            fixture_hosts()
        );
    }

    #[tokio::test]
    async fn refresh_without_install_errors() {
        let (_dir, dns) = setup();
        let http = reqwest::Client::new();
        assert!(dns.refresh(&http).await.is_err());
    }
}
