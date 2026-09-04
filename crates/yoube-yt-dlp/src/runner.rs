use async_trait::async_trait;
use std::path::Path;
use yoube_core::AppResult;

#[async_trait]
pub trait CommandRunner: Send + Sync {
    async fn output(&self, bin: &Path, args: &[&str]) -> AppResult<std::process::Output>;
}

pub struct TokioCommandRunner;

#[async_trait]
impl CommandRunner for TokioCommandRunner {
    async fn output(&self, bin: &Path, args: &[&str]) -> AppResult<std::process::Output> {
        use tokio::process::Command;
        Ok(Command::new(bin).args(args).output().await?)
    }
}
