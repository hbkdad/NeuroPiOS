//! 3-node relayed Kademlia discovery: peer C discovers peer A THROUGH peer
//! B, without ever dialing A directly -- the concrete proof needed for the
//! Phase 3 MVP exit criterion "three machines discover each other."
//!
//! This was a genuinely open, root-caused-but-unsolved bug through
//! milestone 11 (see README.md's former "Known issue" section, now
//! resolved). Root cause, found via `RUST_LOG=libp2p_kad=trace` protocol
//! tracing: `libp2p_swarm::Config`'s default `idle_connection_timeout` is
//! `Duration::ZERO`. B genuinely computed the correct FIND_NODE response
//! for C (`kad::Event::InboundRequest { request: FindNode {
//! num_closer_peers: 1 } }`), but the B<->C connection was then torn down
//! via `KeepAliveTimeout` before the response was confirmed delivered,
//! because no behaviour handler reported further pending work on that
//! connection -- a zero-grace-period race between "handler says no more
//! work" and "the response bytes have actually reached the peer." Fixed in
//! `build_swarm_with_limits` by setting a nonzero
//! `with_idle_connection_timeout`. This test is the regression test for
//! that fix.

use aion_p2p::{add_kademlia_bootstrap_peer, build_swarm, feed_identify_into_kademlia};
use futures::StreamExt;
use libp2p::{
    identity,
    kad::{self, QueryResult},
    swarm::SwarmEvent,
    Multiaddr,
};
use std::time::Duration;

#[tokio::test]
async fn c_discovers_a_via_b_without_ever_dialing_a_directly() {
    let mut swarm_a = build_swarm(identity::Keypair::generate_ed25519()).unwrap();
    let mut swarm_b = build_swarm(identity::Keypair::generate_ed25519()).unwrap();
    let mut swarm_c = build_swarm(identity::Keypair::generate_ed25519()).unwrap();
    let peer_a_id = *swarm_a.local_peer_id();
    let peer_b_id = *swarm_b.local_peer_id();

    swarm_a
        .listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .unwrap();
    swarm_b
        .listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .unwrap();
    swarm_c
        .listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .unwrap();

    let a_addr: Multiaddr = loop {
        if let SwarmEvent::NewListenAddr { address, .. } = swarm_a.select_next_some().await {
            break address;
        }
    };
    let b_addr: Multiaddr = loop {
        if let SwarmEvent::NewListenAddr { address, .. } = swarm_b.select_next_some().await {
            break address;
        }
    };
    loop {
        if let SwarmEvent::NewListenAddr { .. } = swarm_c.select_next_some().await {
            break;
        }
    }

    // Phase 1: B dials A directly, so B's routing table genuinely contains
    // A (the fact under test is what C learns via B, not what B already
    // knows -- that part must be real, not assumed).
    add_kademlia_bootstrap_peer(
        &mut swarm_b.behaviour_mut().kademlia,
        peer_a_id,
        a_addr.clone(),
    );
    swarm_b.dial(a_addr).unwrap();

    let mut b_connected_to_a = false;
    let phase1_deadline = tokio::time::sleep(Duration::from_secs(10));
    tokio::pin!(phase1_deadline);
    while !b_connected_to_a {
        tokio::select! {
            _ = &mut phase1_deadline => panic!("phase 1 timed out: B never connected to A"),
            ev_a = swarm_a.select_next_some() => {
                if let SwarmEvent::Behaviour(aion_p2p::AionBehaviourEvent::Identify(ref e)) = ev_a {
                    feed_identify_into_kademlia(&mut swarm_a.behaviour_mut().kademlia, e);
                }
            }
            ev_b = swarm_b.select_next_some() => {
                if let SwarmEvent::ConnectionEstablished { peer_id, .. } = &ev_b {
                    if *peer_id == peer_a_id {
                        b_connected_to_a = true;
                    }
                }
            }
        }
    }

    // Phase 2: C dials ONLY B -- C never learns A's address directly, only
    // B's PeerId+address. Then C asks Kademlia to find A.
    add_kademlia_bootstrap_peer(&mut swarm_c.behaviour_mut().kademlia, peer_b_id, b_addr);
    swarm_c.dial(peer_b_id).unwrap();

    let mut c_connected_to_b = false;
    let mut query_issued = false;
    let mut discovered_peers: Vec<libp2p::PeerId> = Vec::new();
    let mut query_succeeded = false;
    let deadline = tokio::time::sleep(Duration::from_secs(15));
    tokio::pin!(deadline);

    while !query_succeeded {
        tokio::select! {
            _ = &mut deadline => panic!("phase 2 timed out waiting for C's relayed query to complete"),
            ev_a = swarm_a.select_next_some() => {
                if let SwarmEvent::Behaviour(aion_p2p::AionBehaviourEvent::Identify(ref e)) = ev_a {
                    feed_identify_into_kademlia(&mut swarm_a.behaviour_mut().kademlia, e);
                }
            }
            ev_b = swarm_b.select_next_some() => {
                if let SwarmEvent::Behaviour(aion_p2p::AionBehaviourEvent::Identify(ref e)) = ev_b {
                    feed_identify_into_kademlia(&mut swarm_b.behaviour_mut().kademlia, e);
                }
            }
            ev_c = swarm_c.select_next_some() => {
                match &ev_c {
                    SwarmEvent::ConnectionEstablished { peer_id, .. } if *peer_id == peer_b_id => {
                        c_connected_to_b = true;
                    }
                    SwarmEvent::Behaviour(aion_p2p::AionBehaviourEvent::Kademlia(
                        kad::Event::OutboundQueryProgressed {
                            result: QueryResult::GetClosestPeers(Ok(ok)),
                            ..
                        },
                    )) => {
                        // libp2p 0.56 enriches GetClosestPeersOk::peers from
                        // Vec<PeerId> to Vec<PeerInfo> (peer_id + known addrs).
                        discovered_peers = ok.peers.iter().map(|info| info.peer_id).collect();
                        query_succeeded = true;
                    }
                    _ => {}
                }

                if c_connected_to_b && !query_issued {
                    swarm_c.behaviour_mut().kademlia.get_closest_peers(peer_a_id);
                    query_issued = true;
                }
            }
        }
    }

    // The actual claim under test: C's routing table now genuinely
    // contains A, learned entirely via B's FIND_NODE response -- C never
    // dialed A's address directly at any point in this test.
    assert!(
        discovered_peers.contains(&peer_a_id),
        "C should have discovered A via B's relayed FIND_NODE response, got: {discovered_peers:?}"
    );
    let c_kbucket_has_a = swarm_c
        .behaviour_mut()
        .kademlia
        .kbucket(peer_a_id)
        .map(|kb| kb.iter().any(|e| *e.node.key.preimage() == peer_a_id))
        .unwrap_or(false);
    assert!(
        c_kbucket_has_a,
        "C's own routing table should contain A after the relayed discovery"
    );
}
