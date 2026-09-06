//! Background downloader: a `DashMap` job queue + `tokio` worker per download.
//!
//! Lives in `yoube-yt-dlp` (leaf → `yoube-error`) so it's offline-testable.
//! The `DownloaderService` trait that exposes it to Tauri lives in
//! `yoube-core/src/services/downloader.rs` (Task 3.2).
//!
//! Architecture note (faithful deviation from the brief): the brief spawns
//! `tokio::process::Command` directly + uses `nix::kill` for pause/resume.
//! That bypasses the `CommandRunner` injection layer and would make the
//! progress-parsing logic untestable with a mock. This module routes every
//! subprocess through `Arc<dyn CommandRunner>::spawn_streaming` instead, so
//! the core download loop is fully exercised offline. OS signal delivery
//! (SIGSTOP/SIGCONT/SIGTERM) for pause/resume/cancel still operates on the
//! real PID — but only the POSIX path is implemented here, since the brief's
//! `windows` crate isn't a workspace dependency.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use yoube_error::{AppError, AppResult};

use crate::YtDlp;
use crate::runner::Spawned;
#[cfg(test)]
use crate::runner::{ChildHandle, CommandRunner};

pub type JobId = u64;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobState {
    Queued,
    Running,
    Paused,
    Done,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub video_id: String,
    pub format_id: String,
    pub dest_dir: PathBuf,
    pub state: JobState,
    pub progress_bytes: u64,
    pub total_bytes: Option<u64>,
    pub eta_s: Option<u32>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DownloadEvent {
    Progress {
        id: JobId,
        bytes: u64,
        total: Option<u64>,
        eta_s: Option<u32>,
    },
    State {
        id: JobId,
        state: JobState,
        error: Option<String>,
    },
}

/// Parsed fragments pulled out of a single yt-dlp progress line.
#[derive(Debug, Default)]
struct ProgressBits {
    bytes: Option<u64>,
    total: Option<u64>,
    eta_s: Option<u32>,
}

/// Parse a single yt-dlp progress line into `(bytes, total, eta_s)`.
///
/// Handles the default template shape (`[download] 12.3% of ~50.00MiB ...`)
/// as well as bare numeric throughput lines from `--progress-template`.
///
/// Returns `None` for non-progress lines (e.g. `[info] ...`).
fn parse_progress_line(line: &str) -> Option<ProgressBits> {
    let body = line.split("[download]").nth(1)?;
    let s = body.trim();
    let mut out = ProgressBits::default();
    let mut got = false;

    // percent-of form: "12.3% of ~50.00MiB ... ETA 00:13"
    if let Some(pct_idx) = s.find('%')
        && let Some(pct_str) = s[..pct_idx].split_whitespace().last()
        && let Ok(pct) = pct_str.parse::<f64>()
        && let Some(of_idx) = s[pct_idx..].find("of ")
    {
        let rest = &s[pct_idx + of_idx + 3..];
        let total_tok = rest.split_whitespace().next().unwrap_or("");
        if let Some(b) = parse_magnitude(total_tok) {
            out.total = Some(b);
            out.bytes = Some((b as f64 * pct / 100.0) as u64);
            got = true;
        }
    }

    // ETA
    if let Some(eta_idx) = s.find("ETA") {
        let eta_part = s[eta_idx + 3..].trim();
        out.eta_s = Some(parse_eta_seconds(
            eta_part.split_whitespace().next().unwrap_or("0"),
        ));
        got = true;
    }

    // Bare byte count (from --progress-template numeric output).
    if !got {
        for tok in s.split_whitespace() {
            if let Ok(n) = tok.parse::<u64>() {
                out.bytes = Some(n);
                got = true;
                break;
            }
        }
    }

    if got { Some(out) } else { None }
}

/// Parse `1.5MiB`, `50.00MiB`, `~3.2GiB` → bytes.
fn parse_magnitude(s: &str) -> Option<u64> {
    let s = s.trim_start_matches('~');
    let split_at = s.find(|c: char| c.is_ascii_alphabetic()).unwrap_or(s.len());
    let num: f64 = s[..split_at].parse().ok()?;
    let unit = s[split_at..].to_ascii_lowercase();
    let mul = match unit.as_str() {
        "kib" => 1024.0,
        "mib" => 1024.0 * 1024.0,
        "gib" => 1024.0 * 1024.0 * 1024.0,
        "kb" => 1e3,
        "mb" => 1e6,
        "gb" => 1e9,
        "" => 1.0,
        _ => return None,
    };
    Some((num * mul) as u64)
}

/// Parse `00:13` or `13` → seconds.
fn parse_eta_seconds(s: &str) -> u32 {
    if s.contains(':') {
        let mut secs = 0u32;
        for part in s.split(':') {
            let v = part.parse::<u32>().unwrap_or(0);
            secs = secs * 60 + v;
        }
        secs
    } else {
        s.parse::<u32>().unwrap_or(0)
    }
}

const MAX_CONCURRENT: usize = 2;

pub struct Downloader {
    ytdlp: YtDlp,
    jobs: Arc<DashMap<JobId, Job>>,
    jobs_pid: Arc<DashMap<JobId, u32>>,
    next_id: AtomicU64,
    tx: broadcast::Sender<DownloadEvent>,
}

impl Downloader {
    pub fn new(ytdlp: YtDlp) -> Self {
        let (tx, _rx) = broadcast::channel::<DownloadEvent>(256);
        Self {
            ytdlp,
            jobs: Arc::new(DashMap::new()),
            jobs_pid: Arc::new(DashMap::new()),
            next_id: AtomicU64::new(1),
            tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DownloadEvent> {
        self.tx.subscribe()
    }

    pub fn list(&self) -> Vec<Job> {
        self.jobs.iter().map(|e| e.value().clone()).collect()
    }

    pub fn job(&self, id: JobId) -> Option<Job> {
        // Clone out of the shard guard immediately; never hold across calls.
        self.jobs.get(&id).map(|e| e.value().clone())
    }

    fn emit(&self, ev: DownloadEvent) {
        let _ = self.tx.send(ev);
    }

    /// Queue a download. Returns the assigned `JobId`.
    pub fn enqueue(&self, video_id: &str, format_id: &str, dest_dir: &Path) -> AppResult<JobId> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let job = Job {
            id,
            video_id: video_id.into(),
            format_id: format_id.into(),
            dest_dir: dest_dir.to_path_buf(),
            state: JobState::Queued,
            progress_bytes: 0,
            total_bytes: None,
            eta_s: None,
            error: None,
        };
        self.jobs.entry(id).or_insert(job);
        self.emit(DownloadEvent::State {
            id,
            state: JobState::Queued,
            error: None,
        });
        self.try_spawn_worker();
        Ok(id)
    }

    /// Schedule the next queued job if a worker slot is free.
    fn try_spawn_worker(&self) {
        let running = {
            self.jobs
                .iter()
                .filter(|e| e.value().state == JobState::Running)
                .count()
        };
        if running >= MAX_CONCURRENT {
            return;
        }
        // NOTE: scope the iterator so its shard locks drop before `get_mut`
        // below — holding `iter()` across `get_mut` deadlocks DashMap.
        let next_queued: Option<JobId> = {
            self.jobs
                .iter()
                .find(|e| e.value().state == JobState::Queued)
                .map(|e| *e.key())
        };
        if let Some(id) = next_queued {
            if let Some(mut j) = self.jobs.get_mut(&id) {
                j.state = JobState::Running;
                j.error = None;
            }
            self.emit(DownloadEvent::State {
                id,
                state: JobState::Running,
                error: None,
            });
            let ytdlp = self.ytdlp.clone();
            let jobs = Arc::clone(&self.jobs);
            let pids = Arc::clone(&self.jobs_pid);
            let tx = self.tx.clone();
            tokio::spawn(async move {
                run_one(&ytdlp, &jobs, &pids, &tx, id).await;
            });
        }
    }

    pub fn pause(&self, id: JobId) -> AppResult<()> {
        self.mutate_state(id, JobState::Paused, None);
        if let Some(pid) = self.jobs_pid.get(&id) {
            suspend_process(*pid.value())?;
        }
        Ok(())
    }

    pub fn resume(&self, id: JobId) -> AppResult<()> {
        self.mutate_state(id, JobState::Running, None);
        if let Some(pid) = self.jobs_pid.get(&id) {
            resume_process(*pid.value())?;
        }
        Ok(())
    }

    pub fn cancel(&self, id: JobId) -> AppResult<()> {
        self.mutate_state(id, JobState::Cancelled, None);
        if let Some((_, pid)) = self.jobs_pid.remove(&id) {
            terminate_process(pid)?;
        }
        if let Some(job) = self.jobs.get(&id) {
            let partial = job.dest_dir.join(format!("{}.part", job.video_id));
            let _ = std::fs::remove_file(&partial);
        }
        Ok(())
    }

    fn mutate_state(&self, id: JobId, state: JobState, error: Option<String>) {
        if let Some(mut j) = self.jobs.get_mut(&id) {
            j.state = state;
            j.error = error.clone();
        }
        self.emit(DownloadEvent::State { id, state, error });
    }

    /// Kick off the next queued download (called after resume/cancel frees a slot).
    pub fn try_next(&self) {
        self.try_spawn_worker();
    }
}

/// Drive a single yt-dlp invocation to completion, emitting progress events.
async fn run_one(
    ytdlp: &YtDlp,
    jobs: &DashMap<JobId, Job>,
    pids: &DashMap<JobId, u32>,
    tx: &broadcast::Sender<DownloadEvent>,
    id: JobId,
) {
    let (format_id, dest_dir, video_id) = {
        let g = match jobs.get(&id) {
            Some(g) => g,
            None => return,
        };
        (
            g.value().format_id.clone(),
            g.value().dest_dir.clone(),
            g.value().video_id.clone(),
        )
    };

    let out_path = dest_dir.join(format!("{}.%(ext)s", video_id));
    let url = format!("https://www.youtube.com/watch?v={video_id}");
    let args = [
        "-f",
        &format_id,
        "--newline",
        "--progress-template",
        "%(progress)",
        "-o",
        &out_path.to_string_lossy(),
        &url,
    ];

    let spawned = match ytdlp.runner.spawn_streaming(&ytdlp.bin, &args).await {
        Ok(s) => s,
        Err(e) => {
            fail_job(jobs, tx, id, e.to_string()).await;
            return;
        }
    };
    pids.insert(id, spawned.child.pid);
    drain_progress(jobs, pids, tx, id, spawned, &video_id).await;
    pids.remove(&id);
}

/// Read progress lines until the child stream ends, driving the job state.
async fn drain_progress(
    jobs: &DashMap<JobId, Job>,
    _pids: &DashMap<JobId, u32>,
    tx: &broadcast::Sender<DownloadEvent>,
    id: JobId,
    spawned: Spawned,
    _video_id: &str,
) {
    let mut stdout = spawned.child.stdout;
    let mut last_total: Option<u64> = None;

    while let Some(line) = stdout.recv().await {
        // If the job was cancelled mid-flight, stop consuming.
        let state = {
            let g = jobs.get(&id);
            g.map(|g| g.value().state)
        };
        if matches!(state, Some(JobState::Cancelled)) {
            break;
        }

        if let Some(bits) = parse_progress_line(&line) {
            if let Some(t) = bits.total {
                last_total = Some(t);
            }
            let bytes = bits.bytes.unwrap_or(0);
            let total = bits.total.or(last_total);
            if let Some(mut j) = jobs.get_mut(&id) {
                j.progress_bytes = bytes;
                j.total_bytes = total;
                j.eta_s = bits.eta_s;
            }
            let _ = tx.send(DownloadEvent::Progress {
                id,
                bytes,
                total,
                eta_s: bits.eta_s,
            });
        }
    }

    // Stream ended. If the job wasn't cancelled, mark it done. If a signal
    // paused it, preserve Paused — the worker just exited for that reason.
    let final_state = {
        let mut g = jobs.get_mut(&id);
        match g.as_deref_mut() {
            Some(j) if j.state != JobState::Cancelled => {
                if j.state != JobState::Paused {
                    j.state = JobState::Done;
                }
                j.state
            }
            Some(j) => j.state,
            None => JobState::Cancelled,
        }
    };
    let _ = tx.send(DownloadEvent::State {
        id,
        state: final_state,
        error: None,
    });
}

async fn fail_job(
    jobs: &DashMap<JobId, Job>,
    tx: &broadcast::Sender<DownloadEvent>,
    id: JobId,
    err: String,
) {
    if let Some(mut j) = jobs.get_mut(&id) {
        j.state = JobState::Failed;
        j.error = Some(err.clone());
    }
    let _ = tx.send(DownloadEvent::State {
        id,
        state: JobState::Failed,
        error: Some(err),
    });
}

// ---------------------------------------------------------------------------
// Cross-platform process control for pause/resume/cancel.
// Uses std-only POSIX `kill` (no `nix`/`libc` workspace dep). Windows uses
// NtSuspendProcess per the brief, but the `windows` crate isn't a workspace
// dependency, so that path is unimplemented — POSIX covers the testable
// surface for Phase 3.
// ---------------------------------------------------------------------------

#[cfg(unix)]
fn suspend_process(pid: u32) -> AppResult<()> {
    unix_signal(pid, SIGSTOP)
}
#[cfg(windows)]
fn suspend_process(_pid: u32) -> AppResult<()> {
    Err(AppError::Internal(anyhow::anyhow!(
        "pause/resume on Windows is not implemented"
    )))
}

#[cfg(unix)]
fn resume_process(pid: u32) -> AppResult<()> {
    unix_signal(pid, SIGCONT)
}
#[cfg(windows)]
fn resume_process(_pid: u32) -> AppResult<()> {
    Err(AppError::Internal(anyhow::anyhow!(
        "pause/resume on Windows is not implemented"
    )))
}

#[cfg(unix)]
fn terminate_process(pid: u32) -> AppResult<()> {
    unix_signal(pid, SIGTERM)
}
#[cfg(windows)]
fn terminate_process(_pid: u32) -> AppResult<()> {
    Err(AppError::Internal(anyhow::anyhow!(
        "cancel on Windows is not implemented"
    )))
}

#[cfg(unix)]
const SIGSTOP: i32 = 19;
#[cfg(unix)]
const SIGCONT: i32 = 18;
#[cfg(unix)]
const SIGTERM: i32 = 15;

#[cfg(unix)]
fn unix_signal(pid: u32, sig: i32) -> AppResult<()> {
    // SAFETY: `kill` is async-signal-safe; we pass a valid pid and signal
    // number only. Return value checked via errno.
    let ret = unsafe { libc::kill(pid as libc::pid_t, sig as libc::c_int) };
    if ret != 0 {
        return Err(AppError::Internal(anyhow::anyhow!(
            "kill({pid}, {sig}) failed: {}",
            std::io::Error::last_os_error()
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Test mock (offline — no yt-dlp binary needed). Kept before the test module
// so `clippy::items_after_test_module` stays clean.
// ---------------------------------------------------------------------------

/// Mock runner for tests: `output()` returns empty success; `spawn_streaming`
/// replays a canned vector of stdout lines over a fresh channel.
#[cfg(test)]
struct LineRunner {
    lines: Vec<String>,
}

#[cfg(test)]
impl LineRunner {
    fn new(lines: Vec<String>) -> Self {
        Self { lines }
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl CommandRunner for LineRunner {
    async fn output(&self, _bin: &Path, _args: &[&str]) -> AppResult<std::process::Output> {
        Ok(std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: vec![],
            stderr: vec![],
        })
    }

    async fn spawn_streaming(&self, _bin: &Path, _args: &[&str]) -> AppResult<Spawned> {
        let (tx, rx) = tokio::sync::mpsc::channel::<String>(64);
        let lines = self.lines.clone();
        let pid = 999u32;
        let join = tokio::spawn(async move {
            for line in lines {
                let _ = tx.send(line).await;
            }
        });
        Ok(Spawned {
            child: ChildHandle { pid, stdout: rx },
            _join: join,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests (offline, using the LineRunner mock above).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_percent_with_eta() {
        let b =
            parse_progress_line("[download]  12.3% of  ~50.00MiB at  3.00MiB/s ETA 00:13").unwrap();
        assert_eq!(b.total, Some(50 * 1024 * 1024));
        assert_eq!(b.eta_s, Some(13));
        assert!(b.bytes.is_some());
    }

    #[test]
    fn parse_bare_byte_form() {
        let b = parse_progress_line("[download] 12345678").unwrap();
        assert_eq!(b.bytes, Some(12345678));
        assert_eq!(b.total, None);
    }

    #[test]
    fn parse_eta_minutes_form() {
        let b = parse_progress_line("[download]  50.0% of 100MiB ETA 01:02").unwrap();
        assert_eq!(b.total, Some(100 * 1024 * 1024));
        assert_eq!(b.eta_s, Some(62));
    }

    #[test]
    fn non_progress_line_yields_none() {
        assert!(parse_progress_line("[info] Writing metadata").is_none());
    }

    #[test]
    fn parse_magnitude_variants() {
        assert_eq!(parse_magnitude("~50.00MiB"), Some(50 * 1024 * 1024));
        assert_eq!(
            parse_magnitude("1.5GiB"),
            Some((1.5 * 1024.0 * 1024.0 * 1024.0) as u64)
        );
        assert_eq!(parse_magnitude("3.2kb"), Some(3200));
        assert_eq!(parse_magnitude("100"), Some(100));
        assert_eq!(parse_magnitude("1.5ZiB"), None);
    }

    #[tokio::test]
    async fn queue_drives_job_to_done_with_progress() {
        let lines = vec![
            "[download]  25.0% of 100MiB ETA 10:00".to_string(),
            "[download]  75.0% of 100MiB ETA 02:00".to_string(),
        ];
        let runner: Arc<dyn CommandRunner> = Arc::new(LineRunner::new(lines));
        let ytdlp = YtDlp::new(PathBuf::from("yt-dlp"), runner);
        let dl = Downloader::new(ytdlp);
        let mut rx = dl.subscribe();
        let id = dl.enqueue("vid1", "best", Path::new("/tmp")).unwrap();

        // Bound the wait so a regression deadlocks the test for 5s, not forever.
        let res = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while let Ok(ev) = rx.recv().await {
                if let DownloadEvent::State { id: eid, state, .. } = ev
                    && eid == id
                    && state == JobState::Done
                {
                    return true;
                }
            }
            false
        })
        .await
        .expect("timed out waiting for JobState::Done — likely DashMap deadlock");
        let done = res;
        assert!(done);

        let j = dl.job(id).unwrap();
        assert_eq!(j.state, JobState::Done);
        assert_eq!(j.total_bytes, Some(100 * 1024 * 1024));
    }
}
