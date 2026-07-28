//! Real AutoNAT integration test: peer B asks peer A to dial back and
//! confirm B's own listen address is actually reachable -- the genuine
//! wire protocol (a request/response exchange over
//! `/libp2p/autonat/1.0.0`), not a stubbed status flip.
//!
//! Topology: A and B connect (manual dial, as every bootstrap needs at
//! least one known peer -- same pattern as the Kademlia tests). B then
//! runs an AutoNAT probe using A as the dial-back server. A genuinely
//! dials B's advertised loopback address, succeeds (since it's really
//! reachable on this machine), and reports back -- B's `autonat::Behaviour`
//! transitions from `NatStatus::Unknown` to `NatStatus::Public(addr)`.
//!
//! `only_global_ips: false` is required on both sides for this test: the
//! production default (`only_global_ips: true`, see
//! `default_autonat_config`) deliberately refuses to treat a private/
//! loopback address as evidence of public reachability, which is correct
//! for real deployments but would make this loopback-only test
//! unconditionally fail the "public" classification no matter what.
//! Overriding it here is a test-environment necessity, not a relaxation
//! of the real default.

use aion_p2p::{build_swarm_with_limits_and_autonat_config, default_connection_limits};
use futures::StreamExt;
use libp2p::{autonat, identity, swarm::SwarmEvent, Multiaddr};
use std::time::Duration;

fn fast_autonat_config() -> autonat::Config {
    autonat::Config {
        boot_delay: Duration::from_millis(50),
        retry_interval: Duration::from_millis(100),
        refresh_interval: Duration::from_secs(5),
        throttle_server_period: Duration::from_millis(50),
        only_global_ips: false,
        ..Default::default()
    }
}

#[tokio::test]
async fn peer_confirms_its_own_address_is_publicly_reachable_via_a_real_dial_back() {
    let mut swarm_a = build_swarm_with_limits_and_autonat_config(
        identity::Keypair::generate_ed25519(),
        default_connection_limits(),
        fast_autonat_config(),
    )
    .unwrap();
    let mut swarm_b = build_swarm_with_limits_and_autonat_config(
        identity::Keypair::generate_ed25519(),
        default_connection_limits(),
        fast_autonat_config(),
    )
    .unwrap();
    let peer_a_id = *swarm_a.local_peer_id();

    swarm_a
        .listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .unwrap();
    swarm_b
        .listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .unwrap();

    let a_addr: Multiaddr = loop {
        if let SwarmEvent::NewListenAddr { address, .. } = swarm_a.select_next_some().await {
            break address;
        }
    };
    loop {
        if let SwarmEvent::NewListenAddr { .. } = swarm_b.select_next_some().await {
            break;
        }
    }

    // B dials A directly; A becomes a candidate AutoNAT server for B once
    // connected (`Config::use_connected` defaults to true).
    swarm_b.dial(a_addr).unwrap();

    let deadline = tokio::time::sleep(Duration::from_secs(15));
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            _ = &mut deadline => panic!(
                "timed out waiting for B's AutoNAT status to become Public; last status: {:?}",
                swarm_b.behaviour_mut().autonat.nat_status()
            ),
            ev_a = swarm_a.select_next_some() => {
                if let SwarmEvent::Behaviour(aion_p2p::AionBehaviourEvent::Autonat(event)) = &ev_a {
                    println!("[A autonat event] {event:?}");
                }
            }
            ev_b = swarm_b.select_next_some() => {
                if let SwarmEvent::Behaviour(aion_p2p::AionBehaviourEvent::Autonat(event)) = &ev_b {
                    println!("[B autonat event] {event:?}");
                }
            }
        }

        if swarm_b.behaviour_mut().autonat.nat_status().is_public() {
            break;
        }
    }

    let public_addr = swarm_b
        .behaviour_mut()
        .autonat
        .public_address()
        .cloned()
        .expect("nat_status().is_public() implies public_address() is Some");
    // The confirmed address should be a real dialable loopback address
    // A genuinely connected back to, not a placeholder.
    assert!(public_addr.to_string().contains("127.0.0.1"));
    assert_ne!(swarm_b.local_peer_id(), &peer_a_id);
}
