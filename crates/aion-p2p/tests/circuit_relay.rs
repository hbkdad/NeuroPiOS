//! Real circuit relay integration test: peer C connects to peer A entirely
//! THROUGH relay node R, without C ever learning or dialing A's direct
//! address -- the actual NAT-traversal transport AutoNAT (milestone 15)
//! only detects the need for, but cannot itself provide.
//!
//! Topology:
//! 1. R listens on a real TCP address and acts as a relay server
//!    (`relay::Behaviour`, every node built by this crate has this role --
//!    see the crate-level module doc).
//! 2. A dials R directly (the one piece of out-of-band information every
//!    relay client needs -- the relay's own address -- same bootstrap
//!    pattern as every other test in this crate), then makes a
//!    RESERVATION with R by calling `listen_on` a `/p2p-circuit` address
//!    built from R's address. This is the standard libp2p pattern: a
//!    relayed "listen address" is a real request/response reservation
//!    protocol exchange with R, not a local-only construct.
//! 3. C dials `R_addr/p2p/R_peer_id/p2p-circuit/p2p/A_peer_id` -- an
//!    address that names ONLY R directly; A's own direct TCP address never
//!    appears anywhere in what C knows or dials. R relays the connection
//!    through to A per A's reservation.
//!
//! What's under test: C and A end up with a genuine, authenticated,
//! end-to-end encrypted connection (real Noise handshake, real PeerId
//! verification) despite C never having A's direct address -- this is
//! exactly the scenario circuit relay exists for (a node unreachable
//! behind a NAT that AutoNAT would report as `NatStatus::Private`).

use aion_p2p::build_swarm;
use futures::StreamExt;
use libp2p::{identity, multiaddr::Protocol, swarm::SwarmEvent, Multiaddr};
use std::time::Duration;

#[tokio::test]
async fn c_connects_to_a_entirely_through_the_relay_without_ever_dialing_a_directly() {
    let mut swarm_r = build_swarm(identity::Keypair::generate_ed25519()).unwrap();
    let mut swarm_a = build_swarm(identity::Keypair::generate_ed25519()).unwrap();
    let mut swarm_c = build_swarm(identity::Keypair::generate_ed25519()).unwrap();
    let peer_r_id = *swarm_r.local_peer_id();
    let peer_a_id = *swarm_a.local_peer_id();

    swarm_r
        .listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .unwrap();
    let r_addr: Multiaddr = loop {
        if let SwarmEvent::NewListenAddr { address, .. } = swarm_r.select_next_some().await {
            break address;
        }
    };
    // In a real deployment, R's relay::Behaviour would learn its own
    // externally-dialable address via AutoNAT confirming it (the reservation
    // response to A must include at least one address for A to be reachable
    // at -- see libp2p-relay's `ReserveError::Protocol(NoAddressesInReservation)`).
    // AutoNAT deliberately refuses to treat a loopback address as evidence of
    // public reachability (`only_global_ips`, see `default_autonat_config`),
    // so on loopback that confirmation never happens automatically. This one
    // explicit call bridges that gap for the test the same way
    // `autonat_reachability.rs` overrides `only_global_ips` -- a
    // test-environment necessity for a loopback-only network, not something
    // a real relay operator would need to do by hand.
    swarm_r.add_external_address(r_addr.clone());

    // Phase 1: A dials R directly, then reserves a relayed listen address
    // through R -- a real reservation protocol exchange, not a local stub.
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
            _ = &mut phase1_deadline => panic!(
                "phase 1 timed out: a_connected_to_r={a_connected_to_r} a_reservation_confirmed={a_reservation_confirmed}"
            ),
            ev_r = swarm_r.select_next_some() => {
                println!("[R event] {ev_r:?}");
            }
            ev_a = swarm_a.select_next_some() => {
                println!("[A event] {ev_a:?}");
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
    println!("phase 1 done: A has a confirmed reservation on R");

    // Phase 2: C dials ONLY R's address plus A's PeerId via /p2p-circuit --
    // C never learns or uses A's direct TCP address anywhere in this test.
    let circuit_addr_to_a = r_addr
        .with(Protocol::P2p(peer_r_id))
        .with(Protocol::P2pCircuit)
        .with(Protocol::P2p(peer_a_id));
    swarm_c.dial(circuit_addr_to_a).unwrap();

    let mut c_connected_to_a = false;
    let mut a_connected_to_c = false;
    let deadline = tokio::time::sleep(Duration::from_secs(15));
    tokio::pin!(deadline);
    while !(c_connected_to_a && a_connected_to_c) {
        tokio::select! {
            _ = &mut deadline => panic!(
                "phase 2 timed out: c_connected_to_a={c_connected_to_a} a_connected_to_c={a_connected_to_c}"
            ),
            ev_r = swarm_r.select_next_some() => {
                println!("[R event] {ev_r:?}");
            }
            ev_a = swarm_a.select_next_some() => {
                println!("[A event] {ev_a:?}");
                if let SwarmEvent::ConnectionEstablished { peer_id, .. } = &ev_a {
                    if *peer_id == *swarm_c.local_peer_id() {
                        a_connected_to_c = true;
                    }
                }
            }
            ev_c = swarm_c.select_next_some() => {
                println!("[C event] {ev_c:?}");
                if let SwarmEvent::ConnectionEstablished { peer_id, .. } = &ev_c {
                    if *peer_id == peer_a_id {
                        c_connected_to_a = true;
                    }
                }
            }
        }
    }

    // The actual claim under test: C is genuinely connected to A, having
    // dialed only an address that names R -- A's own direct address was
    // never part of what C dialed.
    assert!(c_connected_to_a);
    assert!(a_connected_to_c);
}
