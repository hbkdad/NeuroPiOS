//! Real integration test proving `Node::spawn`'s owned background event
//! loop actually works end-to-end -- two nodes gossip a message to each
//! other entirely through `NodeHandle`'s channel-based API, with no test
//! code ever touching a `Swarm` directly (contrast with
//! `tests/node_gossip.rs`, which hand-drives the swarm the older way; this
//! is the same underlying behavior proven through the new API instead).

use aion_node::{Node, NodeEvent, NodeRoles};
use aion_p2p::{Topic, JOB_ANNOUNCEMENTS_TOPIC};
use std::time::Duration;

#[tokio::test]
async fn two_spawned_nodes_gossip_through_their_handles() {
    let node_a = Node::bootstrap(NodeRoles::new());
    let node_b = Node::bootstrap(NodeRoles::new());

    let mut handle_a = node_a.spawn().unwrap();
    let mut handle_b = node_b.spawn().unwrap();

    let topic = Topic::new(JOB_ANNOUNCEMENTS_TOPIC);
    handle_a.subscribe(&topic);
    handle_b.subscribe(&topic);

    handle_a.listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap());

    let listen_addr = loop {
        if let Some(NodeEvent::NewListenAddr(addr)) = handle_a.next_event().await {
            break addr;
        }
    };
    handle_b.dial(listen_addr);

    let payload = b"node-a-says-hello-via-handle".to_vec();
    let mut connected = false;
    let mut published = false;
    let mut received: Option<Vec<u8>> = None;

    let deadline = tokio::time::sleep(Duration::from_secs(15));
    tokio::pin!(deadline);

    while received.is_none() {
        tokio::select! {
            _ = &mut deadline => panic!("timed out waiting for handle-driven gossip roundtrip"),
            event_a = handle_a.next_event() => {
                if let Some(NodeEvent::ConnectionEstablished(_)) = event_a {
                    connected = true;
                }
            }
            event_b = handle_b.next_event() => {
                if let Some(NodeEvent::GossipMessage { data, .. }) = event_b {
                    received = Some(data);
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(300)), if connected && received.is_none() => {
                handle_a.publish(&topic, payload.clone());
                published = true;
            }
        }
    }

    assert!(published);
    assert_eq!(received.unwrap(), payload);
    assert_ne!(handle_a.peer_id(), handle_b.peer_id());
}

#[tokio::test]
async fn a_handles_background_task_stops_once_the_handle_is_dropped() {
    // The background task must not leak/spin forever once nothing owns
    // the handle anymore -- dropping it closes the command channel, which
    // the task's event loop treats as its shutdown signal (see
    // handle.rs's `None => break`).
    let node = Node::bootstrap(NodeRoles::new());
    let handle = node.spawn().unwrap();
    let peer_id = handle.peer_id();
    drop(handle);

    // Not directly observable from outside (the task has no external
    // "I stopped" signal by design -- it's a fire-and-forget background
    // task), so this test's real value is documenting and exercising the
    // drop path without panicking or hanging; a subsequent spawn from a
    // fresh node still works correctly, which is what's asserted.
    let node2 = Node::bootstrap(NodeRoles::new());
    let handle2 = node2.spawn().unwrap();
    assert_ne!(peer_id, handle2.peer_id());
}
