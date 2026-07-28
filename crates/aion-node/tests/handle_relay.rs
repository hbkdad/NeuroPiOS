//! Real integration test proving `NodeHandle` surfaces relay-server
//! activity events (`RelayReservationAccepted`, `RelayCircuitAccepted`)
//! when this node acts as a RELAY for other peers -- the same 3-node
//! scenario `crates/aion-p2p/tests/circuit_relay.rs` proves at the lower
//! level, now observable through the handle API.
//!
//! Topology (same as `circuit_relay.rs`): R relays for A, C connects to A
//! entirely through R without ever dialing A directly. This test's added
//! value over `circuit_relay.rs` is confirming R's OWN `NodeHandle`
//! genuinely reports both relay-server milestones (accepting A's
//! reservation, then accepting C's circuit request) as real events, not
//! just that the end-to-end connection succeeds.

use aion_node::{Node, NodeEvent, NodeRoles, ResourceCaps};
use libp2p::multiaddr::Protocol;
use std::time::Duration;

fn widened_caps() -> ResourceCaps {
    // A fully `ResourceCaps::locked_down()` node's derived
    // `max_established_incoming` caps at exactly 1 (see
    // `ResourceCaps::to_connection_limits`), which is too tight for a
    // relay (R) that needs to accept inbound connections from BOTH A and
    // C simultaneously -- the same class of real, honest connection-limit
    // interaction discovered in `tests/handle_autonat.rs`. Widening the
    // caps here reflects an operator who has actually configured some
    // bandwidth budget, which is the realistic case for a node acting as
    // a relay in the first place (a fully job-locked node wouldn't
    // usefully serve as a relay either).
    ResourceCaps {
        max_bandwidth_mbps: Some(10),
        ..ResourceCaps::locked_down()
    }
}

#[tokio::test]
async fn relay_nodes_handle_reports_reservation_and_circuit_activity() {
    let mut node_r = Node::bootstrap(NodeRoles::new());
    let mut node_a = Node::bootstrap(NodeRoles::new());
    let mut node_c = Node::bootstrap(NodeRoles::new());
    node_r.caps = widened_caps();
    node_a.caps = widened_caps();
    node_c.caps = widened_caps();

    let mut handle_r = node_r.spawn().unwrap();
    let mut handle_a = node_a.spawn().unwrap();
    let mut handle_c = node_c.spawn().unwrap();
    let peer_r_id = handle_r.peer_id();
    let peer_a_id = handle_a.peer_id();

    handle_r.listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap());
    let r_addr = loop {
        if let Some(NodeEvent::NewListenAddr(addr)) = handle_r.next_event().await {
            break addr;
        }
    };
    // R needs a confirmed external address to include in reservation
    // responses -- AutoNAT correctly refuses to confirm one automatically
    // on loopback (`only_global_ips`), so this bridges the gap the same
    // way `crates/aion-p2p/tests/circuit_relay.rs` does at the lower
    // level; a real relay operator with a genuine public IP wouldn't need
    // this manual step.
    handle_r.add_external_address(r_addr.clone());

    // Phase 1: A dials R and reserves a relayed listen address through it.
    let relay_listen_addr = r_addr
        .clone()
        .with(Protocol::P2p(peer_r_id))
        .with(Protocol::P2pCircuit);
    handle_a.dial(r_addr.clone());
    handle_a.listen_on(relay_listen_addr);

    let mut reservation_accepted = false;
    let phase1_deadline = tokio::time::sleep(Duration::from_secs(15));
    tokio::pin!(phase1_deadline);
    while !reservation_accepted {
        tokio::select! {
            _ = &mut phase1_deadline => panic!("phase 1 timed out waiting for R to report the reservation"),
            event_r = handle_r.next_event() => {
                if let Some(NodeEvent::RelayReservationAccepted { src_peer_id }) = event_r {
                    assert_eq!(src_peer_id, peer_a_id);
                    reservation_accepted = true;
                }
            }
            _event_a = handle_a.next_event() => {}
        }
    }

    // Phase 2: C dials ONLY R's address plus A's PeerId via /p2p-circuit.
    let circuit_addr_to_a = r_addr
        .with(Protocol::P2p(peer_r_id))
        .with(Protocol::P2pCircuit)
        .with(Protocol::P2p(peer_a_id));
    handle_c.dial(circuit_addr_to_a);

    let mut circuit_accepted = false;
    let deadline = tokio::time::sleep(Duration::from_secs(15));
    tokio::pin!(deadline);
    while !circuit_accepted {
        tokio::select! {
            _ = &mut deadline => panic!("phase 2 timed out waiting for R to report the relayed circuit"),
            event_r = handle_r.next_event() => {
                if let Some(NodeEvent::RelayCircuitAccepted { src_peer_id, dst_peer_id }) = event_r {
                    assert_eq!(dst_peer_id, peer_a_id);
                    // src_peer_id is C, the peer that dialed through the circuit.
                    let _ = src_peer_id;
                    circuit_accepted = true;
                }
            }
            _event_a = handle_a.next_event() => {}
            _event_c = handle_c.next_event() => {}
        }
    }

    assert!(reservation_accepted);
    assert!(circuit_accepted);
}
