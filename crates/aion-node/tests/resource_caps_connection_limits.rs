//! Real end-to-end proof that `Node::build_swarm()` actually derives its
//! P2P connection limits from the node's OWN `ResourceCaps`
//! (`ResourceCaps::to_connection_limits`), not an unrelated fixed
//! default -- closing the gap `crates/aion-p2p/README.md` used to flag
//! ("a node's configured bandwidth/resource caps and its P2P connection
//! limits are still two independent, unconnected knobs").
//!
//! A freshly-`bootstrap()`ed node (`ResourceCaps::locked_down()`) allows
//! exactly one established inbound connection (see
//! `ResourceCaps::to_connection_limits`'s doc: enough for baseline
//! protocol membership, since locked-down means "no job-payload bandwidth
//! budgeted yet," not "unreachable"). This test proves that cap is
//! actually enforced by the real swarm the node builds, not just correct
//! in isolation (unit-tested via `Debug`-string inspection in
//! `resource_caps.rs`) -- the same "prove enforcement, not just
//! construction" discipline `crates/aion-p2p/tests/connection_limits.rs`
//! already applies one layer down.

use aion_node::{Node, NodeRoles};
use futures::StreamExt;
use libp2p::{swarm::SwarmEvent, Multiaddr};
use std::time::Duration;

#[tokio::test]
async fn a_freshly_bootstrapped_nodes_second_inbound_connection_is_rejected() {
    let node_a = Node::bootstrap(NodeRoles::new());
    let node_b = Node::bootstrap(NodeRoles::new());
    let node_c = Node::bootstrap(NodeRoles::new());

    let mut swarm_a = node_a.build_swarm().unwrap();
    let mut swarm_b = node_b.build_swarm().unwrap();
    let mut swarm_c = node_c.build_swarm().unwrap();

    swarm_a
        .listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .unwrap();
    let listen_addr: Multiaddr = loop {
        if let SwarmEvent::NewListenAddr { address, .. } = swarm_a.select_next_some().await {
            break address;
        }
    };

    // B dials first and should be accepted -- A's locked-down cap allows
    // exactly one established inbound connection.
    swarm_b.dial(listen_addr.clone()).unwrap();

    let deadline = tokio::time::sleep(Duration::from_secs(10));
    tokio::pin!(deadline);
    let mut b_connected = false;
    let mut c_dialed = false;
    let mut a_rejected_c = false;

    while !a_rejected_c {
        tokio::select! {
            _ = &mut deadline => panic!(
                "timed out: b_connected={b_connected} c_dialed={c_dialed} a_rejected_c={a_rejected_c}"
            ),
            event_a = swarm_a.select_next_some() => {
                if let SwarmEvent::IncomingConnectionError { .. } = event_a {
                    a_rejected_c = true;
                }
            }
            event_b = swarm_b.select_next_some() => {
                if let SwarmEvent::ConnectionEstablished { .. } = event_b {
                    b_connected = true;
                }
            }
            _event_c = swarm_c.select_next_some() => {}
        }

        // Only dial C once B is confirmed connected, so the second
        // connection is genuinely the one that exceeds the cap of one.
        if b_connected && !c_dialed {
            swarm_c.dial(listen_addr.clone()).unwrap();
            c_dialed = true;
        }
    }

    assert!(b_connected, "B (the first dial) should have been accepted");
    assert!(
        a_rejected_c,
        "C (the second dial) should have been rejected by A's derived connection limit of 1"
    );
}
