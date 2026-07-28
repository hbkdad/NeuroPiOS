//! The actual node logic run inside each `aion devnet up`-spawned child
//! process. Runs forever (until killed by `aion devnet down`'s SIGTERM,
//! which terminates the process via the default action since no custom
//! handler is installed here), driving one real `aion_node::Node`.

use crate::devnet::{self, DevnetError};
use aion_node::{Node, NodeEvent, NodeRoles, ResourceCaps};
use libp2p::Multiaddr;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// How long a non-bootstrap worker waits for node 0's record to appear
/// before giving up -- bounded rather than an infinite wait, so a crashed
/// bootstrap node produces a real, visible error instead of every other
/// worker hanging forever.
const BOOTSTRAP_WAIT_TIMEOUT: Duration = Duration::from_secs(30);
const BOOTSTRAP_POLL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Error)]
pub enum WorkerError {
    #[error(transparent)]
    Devnet(#[from] DevnetError),
    #[error("failed to start this node's P2P layer: {0}")]
    Node(#[from] aion_node::NodeError),
    #[error("node 0's recorded listen address {0:?} did not parse as a multiaddr: {1}")]
    BadBootstrapAddr(String, libp2p::multiaddr::Error),
}

/// A devnet node needs real connectivity (unlike `ResourceCaps::locked_down()`'s
/// production-correct "accept no work" default), so widen bandwidth the
/// same documented way every multi-node P2P/handle test in this workspace
/// already does.
fn devnet_node_caps() -> ResourceCaps {
    ResourceCaps {
        max_bandwidth_mbps: Some(10),
        ..ResourceCaps::locked_down()
    }
}

async fn wait_for_bootstrap_record(
    dir: &std::path::Path,
) -> Result<devnet::NodeRecord, WorkerError> {
    let deadline = tokio::time::Instant::now() + BOOTSTRAP_WAIT_TIMEOUT;
    loop {
        if let Some(record) = devnet::read_record(dir, 0)? {
            return Ok(record);
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(DevnetError::BootstrapTimeout(BOOTSTRAP_WAIT_TIMEOUT).into());
        }
        tokio::time::sleep(BOOTSTRAP_POLL_INTERVAL).await;
    }
}

pub async fn run(index: usize, dir: PathBuf) -> Result<(), WorkerError> {
    let mut node = Node::bootstrap(NodeRoles::new());
    node.caps = devnet_node_caps();
    let mut handle = node.spawn()?;
    handle.listen_on(
        "/ip4/127.0.0.1/tcp/0"
            .parse()
            .expect("static addr is valid"),
    );

    let listen_addr = loop {
        if let Some(NodeEvent::NewListenAddr(addr)) = handle.next_event().await {
            break addr;
        }
    };
    let peer_id = handle.peer_id();
    let pid = std::process::id();
    let mut connected: Vec<String> = Vec::new();
    // These println!s are this worker's real log output -- `devnet up`
    // redirects each worker's stdout to its own node-<index>.log file
    // (see devnet.rs's module doc), and `aion devnet logs <index>`
    // reads exactly this.
    println!("node {index}: listening on {listen_addr}, peer_id={peer_id}");
    devnet::write_record(
        &dir,
        index,
        pid,
        &peer_id.to_string(),
        &listen_addr.to_string(),
        &connected,
    )?;

    if index != 0 {
        let bootstrap = wait_for_bootstrap_record(&dir).await?;
        let addr: Multiaddr = bootstrap
            .listen_addr
            .parse()
            .map_err(|e| WorkerError::BadBootstrapAddr(bootstrap.listen_addr.clone(), e))?;
        println!("node {index}: dialing bootstrap node 0 at {addr}");
        handle.dial(addr);
    }

    loop {
        if let Some(NodeEvent::ConnectionEstablished(peer)) = handle.next_event().await {
            let peer_str = peer.to_string();
            if !connected.contains(&peer_str) {
                println!("node {index}: connected to peer {peer_str}");
                connected.push(peer_str);
                devnet::write_record(
                    &dir,
                    index,
                    pid,
                    &peer_id.to_string(),
                    &listen_addr.to_string(),
                    &connected,
                )?;
            }
        }
    }
}
