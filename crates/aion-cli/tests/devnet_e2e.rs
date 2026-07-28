//! Real end-to-end proof of the devnet: actually spawns the compiled
//! `aion` binary as genuine OS subprocesses (not in-process tokio tasks,
//! unlike every other multi-node test in this workspace so far) via
//! `env!("CARGO_BIN_EXE_aion")`, drives them through `devnet up`, polls
//! for real P2P connectivity between separate processes, then tears them
//! down and confirms they're actually gone.

use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

const AION_BIN: &str = env!("CARGO_BIN_EXE_aion");

fn run(args: &[&str]) -> std::process::Output {
    Command::new(AION_BIN)
        .args(args)
        .output()
        .expect("failed to run aion binary")
}

fn status_lines(dir: &Path) -> Vec<String> {
    let output = run(&["devnet", "status", "--dir", dir.to_str().unwrap()]);
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect()
}

fn wait_until(mut condition: impl FnMut() -> bool, timeout: Duration, poll: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if condition() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(poll);
    }
}

#[test]
fn a_two_node_devnet_actually_discovers_and_connects_across_real_processes() {
    let dir = tempfile::tempdir().unwrap();
    let dir_str = dir.path().to_str().unwrap();

    let up_output = run(&["devnet", "up", "--nodes", "2", "--dir", dir_str]);
    assert!(
        up_output.status.success(),
        "devnet up failed: {}",
        String::from_utf8_lossy(&up_output.stderr)
    );

    // Real proof of genuine cross-process P2P connectivity: poll `status`
    // (which itself re-derives liveness via `kill -0` and reads each
    // worker's own connected_peers line, last written when a real
    // ConnectionEstablished event fired in that process) until BOTH nodes
    // report at least one connected peer, or time out.
    let connected = wait_until(
        || {
            let lines = status_lines(dir.path());
            lines.len() == 2
                && lines
                    .iter()
                    .all(|line| !line.contains("connected_peers=[]"))
        },
        Duration::from_secs(20),
        Duration::from_millis(250),
    );
    let final_status = status_lines(dir.path());
    assert!(
        connected,
        "nodes never reported connecting to each other; last status:\n{}",
        final_status.join("\n")
    );

    // Both report alive=true while connected.
    assert!(final_status.iter().all(|line| line.contains("alive=true")));

    let down_output = run(&["devnet", "down", "--dir", dir_str]);
    assert!(down_output.status.success());
    assert!(String::from_utf8_lossy(&down_output.stdout).contains("stopped 2 devnet node"));

    // Confirm status now reports nothing (down cleared the records).
    let after_down = run(&["devnet", "status", "--dir", dir_str]);
    assert!(String::from_utf8(after_down.stdout)
        .unwrap()
        .contains("no devnet nodes recorded"));
}

#[test]
fn up_refuses_a_second_run_while_the_first_devnet_is_still_live() {
    let dir = tempfile::tempdir().unwrap();
    let dir_str = dir.path().to_str().unwrap();

    let first = run(&["devnet", "up", "--nodes", "1", "--dir", dir_str]);
    assert!(first.status.success());

    // Give the single node a moment to actually start and record itself.
    // (status_lines always yields at least one line -- "no devnet nodes
    // recorded..." when there's nothing yet -- so check for an actual
    // node record line, not merely non-empty output.)
    wait_until(
        || {
            status_lines(dir.path())
                .iter()
                .any(|l| l.starts_with("node-"))
        },
        Duration::from_secs(10),
        Duration::from_millis(100),
    );

    let second = run(&["devnet", "up", "--nodes", "1", "--dir", dir_str]);
    assert!(!second.status.success());
    assert!(String::from_utf8_lossy(&second.stderr).contains("already"));

    // Clean up so the temp dir's spawned process doesn't linger past the test.
    run(&["devnet", "down", "--dir", dir_str]);
}
