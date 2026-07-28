//! Real integration test proving `NodeHandle` surfaces a genuine DCUtR
//! direct-connection upgrade as `NodeEvent::DirectConnectionUpgrade`, the
//! same hole-punch protocol exchange
//! `crates/aion-p2p/tests/dcutr_hole_punch.rs` proves at the lower level
//! -- now observable through the handle API. Same topology as
//! `tests/handle_relay.rs`: R relays for A, C connects to A entirely
//! through R, and DCUtR then automatically attempts (and, on loopback,
//! succeeds at) upgrading that relayed connection to a direct one.

use aion_node::{Node, NodeEvent, NodeRoles, ResourceCaps};
use libp2p::multiaddr::Protocol;
use std::time::Duration;

fn widened_caps() -> ResourceCaps {
    // Same real, documented connection-limit interaction as
    // tests/handle_relay.rs and tests/handle_autonat.rs: a fully
    // locked-down node's derived per-peer/incoming caps are too tight for
    // this multi-connection scenario (R serving both A and C, plus A and
    // C attempting a NEW direct connection to each other on top of their
    // existing relayed one).
    ResourceCaps {
        max_bandwidth_mbps: Some(10),
        ..ResourceCaps::locked_down()
    }
}

#[tokio::test]
async fn a_relayed_connections_direct_upgrade_surfaces_through_the_handle_api() {
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

    // A needs its own real listen address too, for the direct hole-punch
    // dial to have somewhere genuine to land.
    handle_a.listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap());
    loop {
        if let Some(NodeEvent::NewListenAddr(_)) = handle_a.next_event().await {
            break;
        }
    }

    // Phase 1: A reserves a relayed listen address through R.
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
            _ = &mut phase1_deadline => panic!("phase 1 timed out"),
            event_r = handle_r.next_event() => {
                if let Some(NodeEvent::RelayReservationAccepted { src_peer_id }) = event_r {
                    assert_eq!(src_peer_id, peer_a_id);
                    reservation_accepted = true;
                }
            }
            _event_a = handle_a.next_event() => {}
        }
    }

    // Phase 2: C dials A entirely through R.
    let circuit_addr_to_a = r_addr
        .with(Protocol::P2p(peer_r_id))
        .with(Protocol::P2pCircuit)
        .with(Protocol::P2p(peer_a_id));
    handle_c.dial(circuit_addr_to_a);

    // Phase 3: keep driving all three handles until DCUtR reports a
    // successful direct upgrade on either A's or C's side.
    let mut upgraded = false;
    let deadline = tokio::time::sleep(Duration::from_secs(20));
    tokio::pin!(deadline);
    while !upgraded {
        tokio::select! {
            _ = &mut deadline => panic!("timed out waiting for DirectConnectionUpgrade"),
            _event_r = handle_r.next_event() => {}
            event_a = handle_a.next_event() => {
                if let Some(NodeEvent::DirectConnectionUpgrade { remote_peer_id, error }) = event_a {
                    if remote_peer_id == peer_c_id && error.is_none() {
                        upgraded = true;
                    }
                }
            }
            event_c = handle_c.next_event() => {
                if let Some(NodeEvent::DirectConnectionUpgrade { remote_peer_id, error }) = event_c {
                    if remote_peer_id == peer_a_id && error.is_none() {
                        upgraded = true;
                    }
                }
            }
        }
    }

    assert!(upgraded);
}
