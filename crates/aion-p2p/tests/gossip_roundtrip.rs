//! Real two-peer integration test: two independent libp2p swarms, over
//! real TCP+Noise+Yamux on loopback, discover each other via a manual
//! dial (Kademlia auto-discovery is a follow-up, see src/lib.rs's module
//! doc), subscribe to the same GossipSub topic, and one peer's published
//! message is actually received by the other. This is the first concrete
//! evidence that AION's job-announcement gossip channel (docs/PROTOCOL.md)
//! works end-to-end, not just that it compiles.

use aion_p2p::{build_swarm, Topic, JOB_ANNOUNCEMENTS_TOPIC};
use futures::StreamExt;
use libp2p::{gossipsub, identity, swarm::SwarmEvent, Multiaddr};
use std::time::Duration;

#[tokio::test]
async fn published_message_is_received_by_the_other_peer() {
    let mut swarm_a = build_swarm(identity::Keypair::generate_ed25519()).unwrap();
    let mut swarm_b = build_swarm(identity::Keypair::generate_ed25519()).unwrap();

    let topic = Topic::new(JOB_ANNOUNCEMENTS_TOPIC);
    swarm_a
        .behaviour_mut()
        .gossipsub
        .subscribe(&topic.0)
        .unwrap();
    swarm_b
        .behaviour_mut()
        .gossipsub
        .subscribe(&topic.0)
        .unwrap();

    swarm_a
        .listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .unwrap();

    // Drive swarm_a until it reports the actual listen address the OS assigned.
    let listen_addr: Multiaddr = loop {
        if let SwarmEvent::NewListenAddr { address, .. } = swarm_a.select_next_some().await {
            break address;
        }
    };

    swarm_b.dial(listen_addr).unwrap();

    let publish_payload = b"job-announcement: inference job available".to_vec();
    let mut published = false;
    let mut received_payload: Option<Vec<u8>> = None;

    let deadline = tokio::time::sleep(Duration::from_secs(15));
    tokio::pin!(deadline);

    loop {
        if received_payload.is_some() {
            break;
        }
        tokio::select! {
            _ = &mut deadline => panic!("timed out waiting for gossip message roundtrip"),
            event_a = swarm_a.select_next_some() => {
                if let SwarmEvent::ConnectionEstablished { .. } = &event_a {
                    if !published {
                        // Give gossipsub a moment to complete mesh formation
                        // after the connection comes up before publishing --
                        // publishing immediately on connect can race the
                        // mesh graft. Retried via the outer publish-attempt
                        // loop below in practice; here we just attempt once
                        // connection is up and let gossipsub's own internal
                        // retry/heartbeat handle propagation.
                        let _ = swarm_a
                            .behaviour_mut()
                            .gossipsub
                            .publish(topic.0.clone(), publish_payload.clone());
                        published = true;
                    }
                }
            }
            event_b = swarm_b.select_next_some() => {
                if let SwarmEvent::Behaviour(aion_p2p::AionBehaviourEvent::Gossipsub(
                    gossipsub::Event::Message { message, .. },
                )) = event_b
                {
                    received_payload = Some(message.data);
                }
                // Retry publish periodically from swarm_a's side in case the
                // first publish attempt raced mesh formation -- handled by
                // re-publishing on a timer via the branch below.
            }
            _ = tokio::time::sleep(Duration::from_millis(300)), if published && received_payload.is_none() => {
                let _ = swarm_a
                    .behaviour_mut()
                    .gossipsub
                    .publish(topic.0.clone(), publish_payload.clone());
            }
        }
    }

    assert_eq!(received_payload.unwrap(), publish_payload);
}
