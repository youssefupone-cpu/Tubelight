use async_trait::async_trait;
use std::path::Path;
use yoube_error::AppResult;

/// A handle to a process spawned for streaming use.
///
/// `pid` is exposed so the caller can send OS signals (SIGSTOP/SIGCONT/SIGTERM)
/// for pause / resume / cancel. The child's stdout is exposed as an async
/// line channel; the reader task runs until the child exits.
pub struct ChildHandle {
    pub pid: u32,
    pub stdout: tokio::sync::mpsc::Receiver<String>,
}

/// The result of spawning a streaming child: a handle that owns the live
/// stdout line channel plus the join handle for the reader task.
pub struct Spawned {
    pub child: ChildHandle,
    pub _join: tokio::task::JoinHandle<()>,
}

#[async_trait]
pub trait CommandRunner: Send + Sync {
    /// Run `bin` and collect the entire stdout/stderr (used by metadata methods).
    async fn output(&self, bin: &Path, args: &[&str]) -> AppResult<std::process::Output>;

    /// Spawn `bin` and stream its stdout lines as they arrive.
    ///
    /// Default impl returns an error so the offline `StaticRunner` mock used
    /// by the youtube tests keeps working without implementing streaming.
    /// Callers that need streaming (the downloader) use a mock override.
    async fn spawn_streaming(&self, _bin: &Path, _args: &[&str]) -> AppResult<Spawned> {
        Err(yoube_error::AppError::Internal(anyhow::anyhow!(
            "this runner does not support streaming"
        )))
    }
}

pub struct TokioCommandRunner;

#[async_trait]
impl CommandRunner for TokioCommandRunner {
    async fn output(&self, bin: &Path, args: &[&str]) -> AppResult<std::process::Output> {
        use tokio::process::Command;
        Ok(Command::new(bin).args(args).output().await?)
    }

    async fn spawn_streaming(&self, bin: &Path, args: &[&str]) -> AppResult<Spawned> {
        use tokio::io::AsyncBufReadExt;
        use tokio::process::Command;

        let mut child = Command::new(bin)
            .args(args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
        let pid = child
            .id()
            .ok_or_else(|| yoube_error::AppError::Internal(anyhow::anyhow!("child had no pid")))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| yoube_error::AppError::Internal(anyhow::anyhow!("no stdout")))?;
        let mut reader = tokio::io::BufReader::new(stdout).lines();
        let (tx, rx) = tokio::sync::mpsc::channel::<String>(64);
        let join = tokio::spawn(async move {
            while let Ok(Some(line)) = reader.next_line().await {
                if tx.send(line).await.is_err() {
                    break;
                }
            }
            let _ = child.wait().await;
        });

        Ok(Spawned {
            child: ChildHandle { pid, stdout: rx },
            _join: join,
        })
    }
}
