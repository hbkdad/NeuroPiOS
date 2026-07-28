//! Real integration test proving `aion-node` actually owns and drives a
//! P2P swarm built from ITS OWN identity, not a throwaway one -- two
//! `Node`s gossip a message to each other, and the swarm's on-wire PeerID
//! is checked against the node's `aion_crypto::Identity` public key.

use aion_node::{Node, NodeRoles};
use aion_p2p::{Topic, JOB_ANNOUNCEMENTS_TOPIC};
use futures::StreamExt;
use libp2p::{gossipsub, swarm::SwarmEvent, Multiaddr, PeerId};
use std::time::Duration;

#[tokio::test]
async fn two_nodes_gossip_using_their_own_identities() {
    let node_a = Node::bootstrap(NodeRoles::new());
    let node_b = Node::bootstrap(NodeRoles::new());

    // The swarm's local PeerID must actually correspond to the node's own
    // aion_crypto identity -- not some unrelated key the P2P layer made up.
    let mut swarm_a = node_a.build_swarm().unwrap();
    let mut swarm_b = node_b.build_swarm().unwrap();

    let expected_peer_id_a = PeerId::from_public_key(
        &libp2p::identity::PublicKey::try_decode_protobuf(&{
            // aion_crypto exposes only raw ed25519 public key bytes; wrap
            // them the same way aion_p2p::keypair_from_identity does, via
            // the round-trip through a bridged keypair, to avoid
            // depending on protobuf-encoding details in this test.
            let kp = aion_p2p::keypair_from_identity(&node_a.p2p_identity).unwrap();
            kp.public().encode_protobuf()
        })
        .unwrap(),
    );
    assert_eq!(*swarm_a.local_peer_id(), expected_peer_id_a);

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
    let listen_addr: Multiaddr = loop {
        if let SwarmEvent::NewListenAddr { address, .. } = swarm_a.select_next_some().await {
            break address;
        }
    };
    swarm_b.dial(listen_addr).unwrap();

    let payload = b"node-a-says-hello".to_vec();
    let mut published = false;
    let mut received: Option<Vec<u8>> = None;

    let deadline = tokio::time::sleep(Duration::from_secs(15));
    tokio::pin!(deadline);

    while received.is_none() {
        tokio::select! {
            _ = &mut deadline => panic!("timed out waiting for node-level gossip roundtrip"),
            event_a = swarm_a.select_next_some() => {
                if let SwarmEvent::ConnectionEstablished { .. } = &event_a {
                    if !published {
                        let _ = swarm_a.behaviour_mut().gossipsub.publish(topic.0.clone(), payload.clone());
                        published = true;
                    }
                }
            }
            event_b = swarm_b.select_next_some() => {
                if let SwarmEvent::Behaviour(aion_p2p::AionBehaviourEvent::Gossipsub(
                    gossipsub::Event::Message { message, .. },
                )) = event_b
                {
                    received = Some(message.data);
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(300)), if published && received.is_none() => {
                let _ = swarm_a.behaviour_mut().gossipsub.publish(topic.0.clone(), payload.clone());
            }
        }
    }

    assert_eq!(received.unwrap(), payload);
}
