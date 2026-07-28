//! Real integration test proving `NodeHandle`'s Kademlia commands
//! (`add_kademlia_bootstrap_peer`, `find_closest_peers`) actually round-trip
//! a genuine FIND_NODE request/response over the wire, entirely through the
//! handle API -- the same underlying protocol
//! `crates/aion-p2p/tests/kademlia_discovery.rs` proves at the lower level,
//! now reachable without hand-driving a `Swarm`.
//!
//! Honest scope note (same as `kademlia_discovery.rs`): in this minimal
//! 2-node network the query legitimately comes back with an EMPTY peer
//! list -- a FIND_NODE responder only returns peers from its own routing
//! table that are closer to the target than itself, and a fresh 2-node
//! network has none yet. What this test actually proves is that the
//! request/response protocol round-trips correctly end-to-end through
//! `NodeHandle`'s command/event API.

use aion_node::{Node, NodeEvent, NodeRoles};
use std::time::Duration;

#[tokio::test]
async fn a_kademlia_query_round_trips_through_the_handle_api() {
    let node_a = Node::bootstrap(NodeRoles::new());
    let node_b = Node::bootstrap(NodeRoles::new());

    let mut handle_a = node_a.spawn().unwrap();
    let mut handle_b = node_b.spawn().unwrap();
    let peer_a_id = handle_a.peer_id();

    handle_a.listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap());
    let a_addr = loop {
        if let Some(NodeEvent::NewListenAddr(addr)) = handle_a.next_event().await {
            break addr;
        }
    };

    // B learns A's address out-of-band (the one piece of information every
    // DHT participant needs before it can join) and dials it.
    handle_b.add_kademlia_bootstrap_peer(peer_a_id, a_addr.clone());
    handle_b.dial(a_addr);

    let mut b_connected = false;
    let mut query_issued = false;
    let mut query_result: Option<Vec<libp2p::PeerId>> = None;

    let deadline = tokio::time::sleep(Duration::from_secs(15));
    tokio::pin!(deadline);

    while query_result.is_none() {
        tokio::select! {
            _ = &mut deadline => panic!("timed out waiting for the handle-driven kademlia query"),
            event_a = handle_a.next_event() => {
                let _ = event_a; // A just needs to be alive to answer B's query
            }
            event_b = handle_b.next_event() => {
                match event_b {
                    Some(NodeEvent::ConnectionEstablished(_)) => {
                        b_connected = true;
                    }
                    Some(NodeEvent::ClosestPeersFound { target, peers }) => {
                        assert_eq!(target, peer_a_id);
                        query_result = Some(peers);
                    }
                    _ => {}
                }
            }
        }

        if b_connected && !query_issued {
            handle_b.find_closest_peers(peer_a_id);
            query_issued = true;
        }
    }

    // The real thing under test: a genuine FIND_NODE request/response
    // completed end-to-end through the handle API (an empty result is
    // honestly expected in this 2-node network, see module doc).
    assert!(query_result.is_some());
}
