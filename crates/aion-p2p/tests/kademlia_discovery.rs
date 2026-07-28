//! Real Kademlia DHT integration test: two peers connect (the initial
//! connection is a manual dial, as any DHT requires at least one known
//! bootstrap peer -- see src/lib.rs's add_kademlia_bootstrap_peer doc),
//! exchange Identify info, bridge that into their Kademlia routing tables
//! (feed_identify_into_kademlia), and then one peer issues a REAL
//! `get_closest_peers` DHT query for the other peer's ID that goes out
//! over the wire (an actual protobuf FIND_NODE request/response via the
//! custom /aion/kad/0.1.0 protocol) and completes successfully.
//!
//! Honest scope note: in this minimal 2-node network the query legitimately
//! comes back with an EMPTY peer list -- a FIND_NODE responder returns
//! peers from its OWN routing table that are closer to the target than
//! itself, and a fresh 2-node network has no such peers yet. What this
//! test actually proves is that the request/response protocol round-trips
//! correctly end-to-end (request encoded, sent, received, decoded,
//! answered) using AION's own protocol name -- not that peer discovery
//! finds anyone in a network this small. A 3+ node test demonstrating an
//! actual "peer C discovers peer A via peer B, without ever dialing A
//! directly" is the natural follow-up and is not attempted here.

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
async fn get_closest_peers_query_completes_successfully_between_two_peers() {
    let mut swarm_a = build_swarm(identity::Keypair::generate_ed25519()).unwrap();
    let mut swarm_b = build_swarm(identity::Keypair::generate_ed25519()).unwrap();
    let peer_a_id = *swarm_a.local_peer_id();

    swarm_a
        .listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .unwrap();

    let listen_addr: Multiaddr = loop {
        if let SwarmEvent::NewListenAddr { address, .. } = swarm_a.select_next_some().await {
            break address;
        }
    };

    // Manual bootstrap: B learns A's address out-of-band (this is the one
    // piece of information every DHT participant needs before it can join
    // -- it is not a shortcut around discovery, it IS how discovery starts).
    add_kademlia_bootstrap_peer(
        &mut swarm_b.behaviour_mut().kademlia,
        peer_a_id,
        listen_addr.clone(),
    );
    swarm_b.dial(listen_addr).unwrap();

    let mut connected = false;
    let mut query_issued = false;
    let mut query_succeeded = false;

    let deadline = tokio::time::sleep(Duration::from_secs(15));
    tokio::pin!(deadline);

    while !query_succeeded {
        tokio::select! {
            _ = &mut deadline => panic!("timed out waiting for kademlia query to complete"),
            event_a = swarm_a.select_next_some() => {
                if let SwarmEvent::Behaviour(aion_p2p::AionBehaviourEvent::Identify(ref identify_event)) = event_a {
                    feed_identify_into_kademlia(&mut swarm_a.behaviour_mut().kademlia, identify_event);
                }
            }
            event_b = swarm_b.select_next_some() => {
                match &event_b {
                    SwarmEvent::ConnectionEstablished { .. } => {
                        connected = true;
                    }
                    SwarmEvent::Behaviour(aion_p2p::AionBehaviourEvent::Identify(identify_event)) => {
                        feed_identify_into_kademlia(&mut swarm_b.behaviour_mut().kademlia, identify_event);
                    }
                    SwarmEvent::Behaviour(aion_p2p::AionBehaviourEvent::Kademlia(
                        kad::Event::OutboundQueryProgressed {
                            result: QueryResult::GetClosestPeers(Ok(ok)),
                            ..
                        },
                    )) => {
                        // A real DHT FIND_NODE round trip over the wire
                        // completed successfully (protobuf request sent to
                        // A, protobuf response parsed back) -- this is the
                        // actual thing under test, not merely that a
                        // routing-table entry exists in memory.
                        println!("kademlia query returned {} peer(s)", ok.peers.len());
                        query_succeeded = true;
                    }
                    _ => {}
                }

                if connected && !query_issued {
                    // Query for A's own PeerId -- A is the one peer B
                    // actually knows about, so this exercises a real
                    // FIND_NODE request to A over the /aion/kad/0.1.0
                    // protocol rather than a query B could answer purely
                    // from its own (otherwise-empty) routing table.
                    swarm_b.behaviour_mut().kademlia.get_closest_peers(peer_a_id);
                    query_issued = true;
                }
            }
        }
    }

    assert!(query_succeeded);
}
