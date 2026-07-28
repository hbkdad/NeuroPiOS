//! Real integration test proving `NodeHandle` surfaces relay-CLIENT
//! events -- the confirmation side complementing `tests/handle_relay.rs`'s
//! relay-SERVER events. Same 3-node topology (R relays for A, C connects
//! to A entirely through R): this test instead checks A's and C's OWN
//! handles, proving each side of the circuit genuinely observes its own
//! relay-client milestone, not just that the connection succeeds:
//! - A (which reserved a relayed listen address on R) sees
//!   `RelayClientReservationAccepted` when R accepts it, and
//!   `RelayClientInboundCircuitEstablished` when C connects to it through
//!   that reservation.
//! - C (which dialed A through R) sees
//!   `RelayClientOutboundCircuitEstablished` once its own circuit
//!   connection is up.

use aion_node::{Node, NodeEvent, NodeRoles, ResourceCaps};
use libp2p::multiaddr::Protocol;
use std::time::Duration;

fn widened_caps() -> ResourceCaps {
    // Same real, documented connection-limit interaction as
    // tests/handle_relay.rs and tests/handle_autonat.rs.
    ResourceCaps {
        max_bandwidth_mbps: Some(10),
        ..ResourceCaps::locked_down()
    }
}

#[tokio::test]
async fn relay_client_events_surface_on_both_the_reserving_and_dialing_peers() {
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
    let peer_c_id = handle_c.peer_id();

    handle_r.listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap());
    let r_addr = loop {
        if let Some(NodeEvent::NewListenAddr(addr)) = handle_r.next_event().await {
            break addr;
        }
    };
    handle_r.add_external_address(r_addr.clone());

    // Phase 1: A dials R and reserves a relayed listen address through it.
    let relay_listen_addr = r_addr
        .clone()
        .with(Protocol::P2p(peer_r_id))
        .with(Protocol::P2pCircuit);
    handle_a.dial(r_addr.clone());
    handle_a.listen_on(relay_listen_addr);

    let mut a_reservation_accepted = false;
    let phase1_deadline = tokio::time::sleep(Duration::from_secs(15));
    tokio::pin!(phase1_deadline);
    while !a_reservation_accepted {
        tokio::select! {
            _ = &mut phase1_deadline => panic!("phase 1 timed out waiting for A's own reservation confirmation"),
            _event_r = handle_r.next_event() => {}
            event_a = handle_a.next_event() => {
                if let Some(NodeEvent::RelayClientReservationAccepted { relay_peer_id }) = event_a {
                    assert_eq!(relay_peer_id, peer_r_id);
                    a_reservation_accepted = true;
                }
            }
        }
    }

    // Phase 2: C dials ONLY R's address plus A's PeerId via /p2p-circuit.
    let circuit_addr_to_a = r_addr
        .with(Protocol::P2p(peer_r_id))
        .with(Protocol::P2pCircuit)
        .with(Protocol::P2p(peer_a_id));
    handle_c.dial(circuit_addr_to_a);

    let mut c_outbound_circuit_established = false;
    let mut a_inbound_circuit_established = false;
    let deadline = tokio::time::sleep(Duration::from_secs(15));
    tokio::pin!(deadline);
    while !(c_outbound_circuit_established && a_inbound_circuit_established) {
        tokio::select! {
            _ = &mut deadline => panic!(
                "phase 2 timed out: c_outbound_circuit_established={c_outbound_circuit_established} a_inbound_circuit_established={a_inbound_circuit_established}"
            ),
            _event_r = handle_r.next_event() => {}
            event_a = handle_a.next_event() => {
                if let Some(NodeEvent::RelayClientInboundCircuitEstablished { src_peer_id }) = event_a {
                    assert_eq!(src_peer_id, peer_c_id);
                    a_inbound_circuit_established = true;
                }
            }
            event_c = handle_c.next_event() => {
                if let Some(NodeEvent::RelayClientOutboundCircuitEstablished { relay_peer_id }) = event_c {
                    assert_eq!(relay_peer_id, peer_r_id);
                    c_outbound_circuit_established = true;
                }
            }
        }
    }

    assert!(a_reservation_accepted);
    assert!(c_outbound_circuit_established);
    assert!(a_inbound_circuit_established);
}
