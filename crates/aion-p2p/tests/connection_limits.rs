//! Real enforcement test for connection limits (docs/adr/0002-p2p-stack.md's
//! anti-spam requirement): a peer configured with zero allowed inbound
//! connections actually rejects a real dial attempt, rather than the
//! `connection_limits::Behaviour` merely being present but inert.

use aion_p2p::{build_swarm_with_limits, default_connection_limits};
use futures::StreamExt;
use libp2p::{connection_limits::ConnectionLimits, identity, swarm::SwarmEvent, Multiaddr};
use std::time::Duration;

#[tokio::test]
async fn peer_with_zero_incoming_limit_rejects_a_real_connection_attempt() {
    let restrictive_limits = ConnectionLimits::default().with_max_established_incoming(Some(0));

    let mut swarm_a =
        build_swarm_with_limits(identity::Keypair::generate_ed25519(), restrictive_limits).unwrap();
    let mut swarm_b = build_swarm_with_limits(
        identity::Keypair::generate_ed25519(),
        default_connection_limits(),
    )
    .unwrap();

    swarm_a
        .listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .unwrap();
    let listen_addr: Multiaddr = loop {
        if let SwarmEvent::NewListenAddr { address, .. } = swarm_a.select_next_some().await {
            break address;
        }
    };

    swarm_b.dial(listen_addr).unwrap();

    let deadline = tokio::time::sleep(Duration::from_secs(10));
    tokio::pin!(deadline);

    let mut a_rejected = false;

    // The connection_limits check runs in `handle_established_inbound_connection`,
    // i.e. at the point the connection would become established -- so B
    // may briefly observe ConnectionEstablished before the limit tears it
    // back down on A's side. The authoritative signal that the limit
    // actually did something is A's IncomingConnectionError, which is what
    // this test asserts on; it would not fire at all if the limit were a
    // no-op.
    while !a_rejected {
        tokio::select! {
            _ = &mut deadline => panic!("timed out waiting for the connection-limit rejection"),
            event_a = swarm_a.select_next_some() => {
                if let SwarmEvent::IncomingConnectionError { .. } = event_a {
                    a_rejected = true;
                }
            }
            _event_b = swarm_b.select_next_some() => {}
        }
    }

    assert!(
        a_rejected,
        "A should have rejected the inbound connection due to its zero-incoming limit"
    );
}

#[tokio::test]
async fn peer_with_normal_limits_accepts_a_connection_the_zero_limit_peer_would_reject() {
    // Control case: the SAME dial succeeds when the limit isn't zero,
    // proving the rejection above is actually caused by the configured
    // limit and not some unrelated transport failure.
    let mut swarm_a = build_swarm_with_limits(
        identity::Keypair::generate_ed25519(),
        default_connection_limits(),
    )
    .unwrap();
    let mut swarm_b = build_swarm_with_limits(
        identity::Keypair::generate_ed25519(),
        default_connection_limits(),
    )
    .unwrap();

    swarm_a
        .listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .unwrap();
    let listen_addr: Multiaddr = loop {
        if let SwarmEvent::NewListenAddr { address, .. } = swarm_a.select_next_some().await {
            break address;
        }
    };

    swarm_b.dial(listen_addr).unwrap();

    let deadline = tokio::time::sleep(Duration::from_secs(10));
    tokio::pin!(deadline);
    let mut connected = false;
    while !connected {
        tokio::select! {
            _ = &mut deadline => panic!("timed out waiting for a normal connection to establish"),
            _event_a = swarm_a.select_next_some() => {}
            event_b = swarm_b.select_next_some() => {
                if let SwarmEvent::ConnectionEstablished { .. } = event_b {
                    connected = true;
                }
            }
        }
    }
    assert!(connected);
}
