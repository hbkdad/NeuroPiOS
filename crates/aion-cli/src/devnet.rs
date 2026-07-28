//! Real multi-process local devnet coordination: `aion devnet up` spawns
//! one real OS child process per node (each running this same binary in
//! hidden `worker` mode), coordinated entirely through small plain-text
//! record files in a shared directory -- no IPC socket, no shared memory,
//! just the filesystem, which is enough for a local dev tool and keeps
//! `up`'s own process from needing to stay alive babysitting children (on
//! Unix, orphaned children are reparented to init and keep running fine
//! after `up` exits, matching how `docker-compose up -d` behaves).
//!
//! Node 0 is always the bootstrap peer: every other node waits for node
//! 0's record to appear (it carries node 0's real listen multiaddr) and
//! dials it. Each worker updates its own record's `connected_peers` line
//! every time a real `ConnectionEstablished` event fires, so `status`
//! reports genuine P2P connectivity, not just "the process is alive."

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DevnetError {
    #[error("devnet directory {0:?} has a live node already -- run `aion devnet down` first")]
    AlreadyRunning(PathBuf),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error(
        "malformed node record at {0:?}: expected at least 3 lines (pid, peer_id, listen_addr)"
    )]
    MalformedRecord(PathBuf),
    #[error("could not determine this binary's own path to spawn workers: {0}")]
    CurrentExe(std::io::Error),
    #[error(
        "timed out after {0:?} waiting for node 0's record to appear -- did it fail to start?"
    )]
    BootstrapTimeout(Duration),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeRecord {
    pub index: usize,
    pub pid: u32,
    pub peer_id: String,
    pub listen_addr: String,
    pub connected_peers: Vec<String>,
}

fn record_path(dir: &Path, index: usize) -> PathBuf {
    dir.join(format!("node-{index}.info"))
}

pub fn write_record(
    dir: &Path,
    index: usize,
    pid: u32,
    peer_id: &str,
    listen_addr: &str,
    connected_peers: &[String],
) -> Result<(), DevnetError> {
    let contents = format!(
        "{pid}\n{peer_id}\n{listen_addr}\n{}\n",
        connected_peers.join(",")
    );
    fs::write(record_path(dir, index), contents)?;
    Ok(())
}

pub fn read_record(dir: &Path, index: usize) -> Result<Option<NodeRecord>, DevnetError> {
    let path = record_path(dir, index);
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(&path)?;
    let mut lines = contents.lines();
    let malformed = || DevnetError::MalformedRecord(path.clone());
    let pid: u32 = lines
        .next()
        .and_then(|l| l.parse().ok())
        .ok_or_else(malformed)?;
    let peer_id = lines.next().ok_or_else(malformed)?.to_string();
    let listen_addr = lines.next().ok_or_else(malformed)?.to_string();
    let connected_peers = lines
        .next()
        .map(|l| {
            l.split(',')
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    Ok(Some(NodeRecord {
        index,
        pid,
        peer_id,
        listen_addr,
        connected_peers,
    }))
}

/// Every node record currently in `dir`, in index order, stopping at the
/// first missing index -- devnet node indices are always a contiguous
/// `0..nodes` range, so a gap means "no more nodes," not "a hole in the
/// middle" (a devnet is always spawned with contiguous indices by `up`).
pub fn list_records(dir: &Path) -> Result<Vec<NodeRecord>, DevnetError> {
    let mut records = Vec::new();
    let mut index = 0;
    while let Some(record) = read_record(dir, index)? {
        records.push(record);
        index += 1;
    }
    Ok(records)
}

/// Whether a PID is still a live process, checked via `kill -0` (the
/// standard POSIX way to probe liveness without actually sending a
/// signal) rather than guessed from the record file's mere existence --
/// a crashed worker still leaves its last-written record behind.
pub fn is_process_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn clear_dir(dir: &Path) -> Result<(), DevnetError> {
    for entry in fs::read_dir(dir)? {
        fs::remove_file(entry?.path())?;
    }
    Ok(())
}

/// Spawns `nodes` real child processes (this same binary, invoked in
/// hidden `worker` mode), one per index `0..nodes`. Returns as soon as
/// they're spawned -- it does not wait for them to finish starting up
/// (use `status` to check on that afterward). Refuses to run again over
/// an already-live devnet in the same directory so a caller can't
/// accidentally leak orphaned processes by forgetting to `down` first.
pub fn up(nodes: usize, dir: &Path) -> Result<(), DevnetError> {
    if dir.exists() {
        let existing = list_records(dir)?;
        if existing.iter().any(|r| is_process_alive(r.pid)) {
            return Err(DevnetError::AlreadyRunning(dir.to_path_buf()));
        }
        clear_dir(dir)?; // stale records from a previous crashed/finished run
    } else {
        fs::create_dir_all(dir)?;
    }

    let current_exe = std::env::current_exe().map_err(DevnetError::CurrentExe)?;
    for index in 0..nodes {
        // Each worker's stdout/stderr is redirected to its own log file,
        // NOT inherited from this process. Workers run indefinitely
        // (until `down` kills them), so inheriting would leave them
        // holding this process's stdio file descriptors open forever --
        // harmless for a real terminal, but it means anything that pipes
        // `up`'s own output (a test harness's `Command::output()`, a
        // shell capturing `$(aion devnet up ...)`, etc.) would block
        // waiting for EOF that never comes while a worker is still
        // running. Per-node log files are also simply more useful for a
        // real devnet tool than an inherited, shared terminal stream.
        let log_path = dir.join(format!("node-{index}.log"));
        let log_file = fs::File::create(&log_path)?;
        Command::new(&current_exe)
            .arg("worker")
            .arg("--index")
            .arg(index.to_string())
            .arg("--dir")
            .arg(dir)
            .stdout(log_file.try_clone()?)
            .stderr(log_file)
            .spawn()?;
    }
    Ok(())
}

/// Each record paired with a freshly-checked liveness flag (never
/// trusted from the record alone, since a worker could have crashed
/// after last writing it).
pub fn status(dir: &Path) -> Result<Vec<(NodeRecord, bool)>, DevnetError> {
    let records = list_records(dir)?;
    Ok(records
        .into_iter()
        .map(|r| {
            let alive = is_process_alive(r.pid);
            (r, alive)
        })
        .collect())
}

/// Sends every currently-live node process a termination signal (`kill`,
/// i.e. SIGTERM -- workers install no custom handler, so this terminates
/// them via the default action) and clears the record directory. Returns
/// how many processes were actually still alive to kill.
pub fn down(dir: &Path) -> Result<usize, DevnetError> {
    let records = list_records(dir)?;
    let mut killed = 0;
    for record in &records {
        if is_process_alive(record.pid) {
            let _ = Command::new("kill").arg(record.pid.to_string()).status();
            killed += 1;
        }
    }
    if dir.exists() {
        clear_dir(dir)?;
    }
    Ok(killed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_then_read_record_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        write_record(
            dir.path(),
            0,
            1234,
            "12D3KooWExamplePeerId",
            "/ip4/127.0.0.1/tcp/4001",
            &["peer-a".to_string(), "peer-b".to_string()],
        )
        .unwrap();

        let record = read_record(dir.path(), 0).unwrap().unwrap();
        assert_eq!(record.pid, 1234);
        assert_eq!(record.peer_id, "12D3KooWExamplePeerId");
        assert_eq!(record.listen_addr, "/ip4/127.0.0.1/tcp/4001");
        assert_eq!(record.connected_peers, vec!["peer-a", "peer-b"]);
    }

    #[test]
    fn reading_a_missing_record_returns_none_not_an_error() {
        let dir = tempfile::tempdir().unwrap();
        assert!(read_record(dir.path(), 0).unwrap().is_none());
    }

    #[test]
    fn a_record_with_no_connected_peers_round_trips_as_an_empty_list() {
        let dir = tempfile::tempdir().unwrap();
        write_record(dir.path(), 0, 1, "peer", "/ip4/127.0.0.1/tcp/1", &[]).unwrap();
        let record = read_record(dir.path(), 0).unwrap().unwrap();
        assert!(record.connected_peers.is_empty());
    }

    #[test]
    fn list_records_stops_at_the_first_gap_in_contiguous_indices() {
        let dir = tempfile::tempdir().unwrap();
        write_record(dir.path(), 0, 1, "p0", "/a0", &[]).unwrap();
        write_record(dir.path(), 1, 2, "p1", "/a1", &[]).unwrap();
        // index 2 deliberately skipped
        write_record(dir.path(), 3, 4, "p3", "/a3", &[]).unwrap();

        let records = list_records(dir.path()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].index, 0);
        assert_eq!(records[1].index, 1);
    }

    #[test]
    fn is_process_alive_is_true_for_this_very_test_process_and_false_for_a_bogus_pid() {
        assert!(is_process_alive(std::process::id()));
        // PID 1 on a real Linux system is always init/systemd and always
        // alive, so use an implausibly large PID instead as the "surely
        // dead" case, matching how kill -0 is meant to be used.
        assert!(!is_process_alive(u32::MAX - 1));
    }

    #[test]
    fn up_refuses_to_run_again_over_a_directory_with_a_live_process_recorded() {
        let dir = tempfile::tempdir().unwrap();
        // Record THIS test process's own PID as if it were a devnet node
        // -- it's genuinely alive, so `up` must refuse.
        write_record(
            dir.path(),
            0,
            std::process::id(),
            "peer",
            "/ip4/127.0.0.1/tcp/1",
            &[],
        )
        .unwrap();

        let err = up(1, dir.path()).unwrap_err();
        assert!(matches!(err, DevnetError::AlreadyRunning(_)));
    }

    #[test]
    fn up_clears_stale_records_left_by_a_dead_process_before_spawning_fresh_ones() {
        let dir = tempfile::tempdir().unwrap();
        write_record(dir.path(), 0, u32::MAX - 1, "dead-peer", "/dead", &[]).unwrap();

        // This actually spawns 1 real child process (this compiled test
        // binary would need a `worker` subcommand to succeed long-term,
        // but we only care here that `up` didn't refuse and that it
        // cleared the stale record before attempting to spawn).
        up(0, dir.path()).unwrap(); // 0 nodes: spawns nothing, just clears + validates the path
        assert!(list_records(dir.path()).unwrap().is_empty());
    }

    #[test]
    fn down_on_an_empty_devnet_kills_nothing_and_does_not_error() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path()).unwrap();
        assert_eq!(down(dir.path()).unwrap(), 0);
    }

    #[test]
    fn down_kills_live_processes_and_clears_records() {
        let dir = tempfile::tempdir().unwrap();
        // Spawn a real, genuinely long-running child process (not this
        // test process itself, so killing it doesn't kill the test) to
        // prove `down` actually terminates something real.
        let mut child = Command::new("sleep").arg("30").spawn().unwrap();
        let pid = child.id();
        write_record(dir.path(), 0, pid, "peer", "/ip4/127.0.0.1/tcp/1", &[]).unwrap();
        assert!(is_process_alive(pid));

        let killed = down(dir.path()).unwrap();
        assert_eq!(killed, 1);

        // Real Unix process semantics, not a bug in `down`/`is_process_alive`:
        // SIGTERM makes the child a ZOMBIE, not gone -- it stays in the
        // process table (so `kill -0` still succeeds) until its parent
        // reaps it via wait(). This test process IS that parent (unlike
        // real `devnet up` usage, where the `up` invocation's own process
        // exits almost immediately, reparenting workers to init, which
        // reaps them automatically) -- so explicitly wait() here to
        // actually reap the zombie before checking liveness.
        child.wait().unwrap();
        assert!(!is_process_alive(pid));
        assert!(list_records(dir.path()).unwrap().is_empty());
    }
}
