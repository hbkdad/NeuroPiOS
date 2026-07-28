//! Real DCUtR (Direct Connection Upgrade through Relay) integration test:
//! once C connects to A through relay R (the same scenario
//! `tests/circuit_relay.rs` proves), DCUtR automatically attempts to
//! upgrade that relayed connection to a direct one via hole-punching --
//! and on loopback, where there's no real NAT in the way, that direct
//! dial genuinely succeeds.
//!
//! Honest scope note: DCUtR is purely reactive (it observes relayed
//! connections and acts on its own, there's no command to "trigger" it),
//! so this test drives the exact same circuit-relay setup as
//! `circuit_relay.rs` and then simply keeps polling until a
//! `dcutr::Event` with a successful result arrives, proving the upgrade
//! behaviour is real and wired in correctly -- not that hole-punching
//! works across a real NAT (this dev container has no real NAT to test
//! against; the loopback environment proves the protocol mechanics, the
//! same honest limitation every other test in this crate that runs over
//! 127.0.0.1 has).

use aion_p2p::build_swarm;
use futures::StreamExt;
use libp2p::{identity, multiaddr::Protocol, swarm::SwarmEvent, Multiaddr};
use std::time::Duration;

#[tokio::test]
async fn a_relayed_connection_is_upgraded_to_a_direct_one_via_hole_punching() {
    let mut swarm_r = build_swarm(identity::Keypair::generate_ed25519()).unwrap();
    let mut swarm_a = build_swarm(identity::Keypair::generate_ed25519()).unwrap();
    let mut swarm_c = build_swarm(identity::Keypair::generate_ed25519()).unwrap();
    let peer_r_id = *swarm_r.local_peer_id();
    let peer_a_id = *swarm_a.local_peer_id();
    let peer_c_id = *swarm_c.local_peer_id();

    swarm_r
        .listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .unwrap();
    let r_addr: Multiaddr = loop {
        if let SwarmEvent::NewListenAddr { address, .. } = swarm_r.select_next_some().await {
            break address;
        }
    };
    // Same test-environment necessity as circuit_relay.rs: R needs a
    // confirmed external address to include in reservation responses,
    // which AutoNAT correctly refuses to provide automatically on
    // loopback.
    swarm_r.add_external_address(r_addr.clone());

    // A must also have a real listen address for hole-punching to have
    // somewhere genuine to dial once the direct-attempt phase begins.
    swarm_a
        .listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .unwrap();

    // Phase 1: A dials R directly, then reserves a relayed listen address.
    let relay_listen_addr = r_addr
        .clone()
        .with(Protocol::P2p(peer_r_id))
        .with(Protocol::P2pCircuit);
    swarm_a.dial(r_addr.clone()).unwrap();
    swarm_a.listen_on(relay_listen_addr).unwrap();

    let mut a_connected_to_r = false;
    let mut a_reservation_confirmed = false;
    let phase1_deadline = tokio::time::sleep(Duration::from_secs(15));
    tokio::pin!(phase1_deadline);
    while !(a_connected_to_r && a_reservation_confirmed) {
        tokio::select! {
            _ = &mut phase1_deadline => panic!("phase 1 timed out"),
            ev_r = swarm_r.select_next_some() => { let _ = ev_r; }
            ev_a = swarm_a.select_next_some() => {
                match &ev_a {
                    SwarmEvent::ConnectionEstablished { peer_id, .. } if *peer_id == peer_r_id => {
                        a_connected_to_r = true;
                    }
                    SwarmEvent::NewListenAddr { address, .. } if address.to_string().contains("p2p-circuit") => {
                        a_reservation_confirmed = true;
                    }
                    _ => {}
                }
            }
        }
    }

    // Phase 2: C dials A entirely through R (never learns A's direct address).
    let circuit_addr_to_a = r_addr
        .with(Protocol::P2p(peer_r_id))
        .with(Protocol::P2pCircuit)
        .with(Protocol::P2p(peer_a_id));
    swarm_c.dial(circuit_addr_to_a).unwrap();

    // Phase 3: keep driving all three swarms until DCUtR reports a
    // successful upgrade on EITHER side of the relayed connection (A or
    // C) -- both peers run DCUtR simultaneously per the protocol.
    let mut dcutr_succeeded = false;
    let deadline = tokio::time::sleep(Duration::from_secs(20));
    tokio::pin!(deadline);
    while !dcutr_succeeded {
        tokio::select! {
            _ = &mut deadline => panic!("timed out waiting for DCUtR to report a successful direct upgrade"),
            ev_r = swarm_r.select_next_some() => { let _ = ev_r; }
            ev_a = swarm_a.select_next_some() => {
                if let SwarmEvent::Behaviour(aion_p2p::AionBehaviourEvent::Dcutr(event)) = &ev_a {
                    println!("[A dcutr event] {event:?}");
                    if event.remote_peer_id == peer_c_id && event.result.is_ok() {
                        dcutr_succeeded = true;
                    }
                }
            }
            ev_c = swarm_c.select_next_some() => {
                if let SwarmEvent::Behaviour(aion_p2p::AionBehaviourEvent::Dcutr(event)) = &ev_c {
                    println!("[C dcutr event] {event:?}");
                    if event.remote_peer_id == peer_a_id && event.result.is_ok() {
                        dcutr_succeeded = true;
                    }
                }
            }
        }
    }

    assert!(dcutr_succeeded);
}
