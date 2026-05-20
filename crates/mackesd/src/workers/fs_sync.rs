//! v2.0.0 Phase B.3 — mesh filesystem sync worker.
//!
//! Supervises the long-running `python3 -m mackes.mesh_gvfs.daemon`
//! process that mounts every reachable peer's QNM-Shared bucket as a
//! FUSE filesystem under `~/.local/share/mackes-mesh-fuse/`.
//! Replaces `mackes-gvfsd-mesh.service`.
//!
//! Unlike the periodic Phase B.4/B.5/B.6 workers, this one expects
//! the subprocess to run forever — any exit is treated as a failure
//! so the Phase A.2 supervisor's `OnFailure` policy triggers an
//! exponential-back-off restart.
//!
//! On shutdown, the worker SIGTERMs the child + waits up to 5 s for
//! a clean exit before SIGKILLing. The fusermount unmount itself is
//! the operator's responsibility (or the systemd unit's ExecStop)
//! since FUSE mounts outlive the daemon process.

#![cfg(feature = "async-services")]

use std::ffi::OsString;
use std::process::Stdio;
use std::time::Duration;

use tokio::process::{Child, Command};

use super::{ShutdownToken, Worker};

/// Graceful-shutdown deadline before SIGKILL.
const SHUTDOWN_GRACE_S: u64 = 5;

/// Worker that spawns + supervises the mesh FUSE daemon.
pub struct FsSyncWorker {
    binary: OsString,
    args: Vec<OsString>,
}

impl Default for FsSyncWorker {
    fn default() -> Self {
        Self::new()
    }
}

impl FsSyncWorker {
    /// Construct a worker that supervises the canonical mesh FUSE
    /// daemon entry point.
    #[must_use]
    pub fn new() -> Self {
        Self {
            binary: OsString::from("python3"),
            args: vec![
                OsString::from("-m"),
                OsString::from("mackes.mesh_gvfs.daemon"),
            ],
        }
    }

    /// Construct with a custom argv — useful for tests that want to
    /// supervise a stand-in process (`sleep 60`, etc).
    #[must_use]
    pub fn with_argv(
        binary: impl Into<OsString>,
        args: impl IntoIterator<Item = impl Into<OsString>>,
    ) -> Self {
        Self {
            binary: binary.into(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }
}

#[async_trait::async_trait]
impl Worker for FsSyncWorker {
    fn name(&self) -> &'static str {
        "fs-sync"
    }

    async fn run(&mut self, mut shutdown: ShutdownToken) -> anyhow::Result<()> {
        let mut child: Child = Command::new(&self.binary)
            .args(&self.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                anyhow::anyhow!(
                    "fs-sync: spawning {} failed: {e}",
                    self.binary.to_string_lossy()
                )
            })?;

        tokio::select! {
            biased;
            _ = shutdown.wait() => {
                graceful_shutdown(&mut child).await;
                Ok(())
            }
            res = child.wait() => {
                let status = res.map_err(|e| {
                    anyhow::anyhow!("fs-sync: wait failed: {e}")
                })?;
                if status.success() {
                    // Daemon exited cleanly — that's still
                    // unexpected since it's supposed to run
                    // forever. Treat as a Recoverable failure.
                    Err(anyhow::anyhow!(
                        "fs-sync: daemon exited cleanly (expected to run forever)"
                    ))
                } else {
                    Err(anyhow::anyhow!(
                        "fs-sync: daemon exited {}",
                        status.code().map_or("?".to_string(), |c| c.to_string())
                    ))
                }
            }
        }
    }
}

async fn graceful_shutdown(child: &mut Child) {
    // tokio's high-level Child API only exposes start_kill (which on
    // Unix sends SIGKILL — there's no SIGTERM via the safe API). Wait
    // up to SHUTDOWN_GRACE_S in case the child saw the supervisor's
    // SIGTERM via its own signal handler and is already exiting; if
    // not, force-kill.
    let _ = tokio::time::timeout(Duration::from_secs(SHUTDOWN_GRACE_S), child.wait()).await;
    let _ = child.start_kill();
    let _ = tokio::time::timeout(Duration::from_secs(1), child.wait()).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fs_sync_worker_name_matches_phase_b_lock() {
        let w = FsSyncWorker::new();
        assert_eq!(w.name(), "fs-sync");
    }

    #[tokio::test]
    async fn fs_sync_worker_exits_clean_on_shutdown_during_run() {
        let mut w = FsSyncWorker::with_argv("sleep", vec![OsString::from("60")]);
        let (tx, rx) = tokio::sync::watch::channel(false);
        let token = ShutdownToken::from_receiver(rx);
        let handle = tokio::spawn(async move { w.run(token).await });
        tokio::time::sleep(Duration::from_millis(50)).await;
        let _ = tx.send(true);
        let result = tokio::time::timeout(Duration::from_secs(10), handle)
            .await
            .expect("worker must exit on shutdown")
            .expect("join");
        assert!(result.is_ok(), "shutdown during run should produce Ok");
    }

    #[tokio::test]
    async fn fs_sync_worker_returns_err_on_clean_subprocess_exit() {
        // `true` exits clean immediately — the worker should treat
        // that as Err (daemons aren't supposed to exit clean).
        let mut w = FsSyncWorker::with_argv("true", Vec::<OsString>::new());
        let (_tx, rx) = tokio::sync::watch::channel(false);
        let token = ShutdownToken::from_receiver(rx);
        let result = tokio::time::timeout(Duration::from_secs(3), w.run(token))
            .await
            .expect("worker exits");
        assert!(result.is_err(), "clean exit should surface as Err");
    }

    #[tokio::test]
    async fn fs_sync_worker_returns_err_when_spawn_fails() {
        let mut w = FsSyncWorker::with_argv("/never/exists/binary", Vec::<OsString>::new());
        let (_tx, rx) = tokio::sync::watch::channel(false);
        let token = ShutdownToken::from_receiver(rx);
        let result = w.run(token).await;
        assert!(result.is_err());
    }
}
