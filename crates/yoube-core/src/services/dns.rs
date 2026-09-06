//! `DnsBlockService`: L1 system-hosts blocking façade.
//!
//! Wraps `yoube-dns::DnsBlock` (path-injected, tempdir-testable) with the
//! shared HTTP client. Install/refresh need admin/root for the real hosts
//! file — the leaf crate surfaces that as a actionable `AppError`.

use async_trait::async_trait;
use std::path::PathBuf;
use std::time::Duration;
use yoube_dns::{DnsBlock, HostsStatus};

use crate::AppResult;

/// System-hosts blocking service (spec §5 `DnsBlockService`, adapted).
#[async_trait]
pub trait DnsBlockService: Send + Sync {
    /// Install the managed block (fetch → backup → write). Returns entries.
    async fn install(&self) -> AppResult<usize>;
    /// Remove the managed block / restore the backup.
    async fn uninstall(&self) -> AppResult<()>;
    /// Current installation state.
    async fn status(&self) -> AppResult<String>;
    /// Re-fetch and replace (only when installed). Returns entries.
    async fn refresh(&self) -> AppResult<usize>;
}

/// Production implementation.
pub struct AppDnsBlockService {
    inner: DnsBlock,
    http: reqwest::Client,
}

impl AppDnsBlockService {
    /// Build against explicit paths (tests) or production paths.
    pub fn new(hosts_path: PathBuf, backup_path: PathBuf) -> AppResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("yoube/0.0.1")
            .build()?;
        Ok(Self {
            inner: DnsBlock::new(hosts_path, backup_path),
            http,
        })
    }

    /// Production wiring: real hosts path + backup under `backup_dir`.
    pub fn production(backup_dir: PathBuf) -> AppResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("yoube/0.0.1")
            .build()?;
        Ok(Self {
            inner: DnsBlock::production(&backup_dir),
            http,
        })
    }
}

#[async_trait]
impl DnsBlockService for AppDnsBlockService {
    async fn install(&self) -> AppResult<usize> {
        self.inner.install(&self.http).await
    }

    async fn uninstall(&self) -> AppResult<()> {
        self.inner.uninstall()
    }

    async fn status(&self) -> AppResult<String> {
        Ok(match self.inner.status()? {
            HostsStatus::NotInstalled => "not_installed".to_string(),
            HostsStatus::Installed { entries } => format!("installed:{entries}"),
            HostsStatus::Unknown(reason) => format!("unknown:{reason}"),
        })
    }

    async fn refresh(&self) -> AppResult<usize> {
        self.inner.refresh(&self.http).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn svc() -> (tempfile::TempDir, AppDnsBlockService) {
        let dir = tempfile::tempdir().unwrap();
        let hosts = dir.path().join("hosts");
        std::fs::write(&hosts, "127.0.0.1 localhost\n").unwrap();
        let s = AppDnsBlockService::new(hosts, dir.path().join("backup").join("hosts.original"))
            .unwrap();
        (dir, s)
    }

    #[tokio::test]
    async fn status_starts_not_installed() {
        let (_dir, s) = svc();
        assert_eq!(s.status().await.unwrap(), "not_installed");
    }

    #[tokio::test]
    async fn uninstall_without_install_keeps_clean_file() {
        let (_dir, s) = svc();
        s.uninstall().await.unwrap();
        assert_eq!(s.status().await.unwrap(), "not_installed");
    }
}
