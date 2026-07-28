//! AION P2P networking, per docs/adr/0002-p2p-stack.md: rust-libp2p on
//! Tokio. This first pass implements GossipSub pub/sub (job/announcement
//! propagation) over TCP+Noise+Yamux. Kademlia peer discovery, AutoNAT,
//! and circuit relay are deferred to a follow-up -- combining multiple
//! NetworkBehaviours correctly is its own real chunk of work, and gossip
//! pub/sub is independently testable and useful on its own (peers can be
//! dialed manually / via an out-of-band address in the meantime).
//!
//! The P2P identity keypair here is generated via libp2p's own
//! `identity::Keypair` (Ed25519 under the hood) rather than bridged from
//! `aion_crypto::Identity` -- unifying the two key materials 1:1 is a
//! follow-up integration task, not forced here through an awkward
//! raw-secret-bytes export that would weaken `aion_crypto::Identity`'s
//! encapsulation for no real benefit yet.

use libp2p::{
    gossipsub, identify, identity, noise, swarm::NetworkBehaviour, swarm::Swarm, tcp, yamux, PeerId,
};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum P2pError {
    #[error("failed to build gossipsub behaviour: {0}")]
    GossipsubBuild(String),
    #[error("transport/swarm build error: {0}")]
    SwarmBuild(String),
    #[error("failed to subscribe to topic: {0}")]
    Subscribe(String),
    #[error("failed to publish message: {0}")]
    Publish(String),
}

#[derive(NetworkBehaviour)]
pub struct AionBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub identify: identify::Behaviour,
}

pub const IDENTIFY_PROTOCOL_VERSION: &str = "/aion/identify/0.1.0";

/// A gossip topic name. Kept as a thin wrapper (not a raw String) so a
/// future anti-spam policy per docs/adr/0002-p2p-stack.md's requirement
/// ("every new gossip topic gets a peer-scoring/rate-limit policy defined
/// alongside it") has one obvious place to attach to.
pub struct Topic(pub gossipsub::IdentTopic);

impl Topic {
    pub fn new(name: &str) -> Self {
        Topic(gossipsub::IdentTopic::new(name))
    }
}

/// Job/announcement gossip topic name, per docs/PROTOCOL.md's job object.
pub const JOB_ANNOUNCEMENTS_TOPIC: &str = "/aion/job-announcements/0.1.0";

pub fn local_peer_id(keypair: &identity::Keypair) -> PeerId {
    PeerId::from(keypair.public())
}

/// Builds a GossipSub behaviour with sane, explicit defaults (message
/// signing required -- anonymous/unsigned gossip is not permitted, since
/// every AION gossip message should be attributable to a peer for the
/// anti-spam/reputation layers described in docs/security/THREAT-MODEL.md).
pub fn build_gossipsub(keypair: &identity::Keypair) -> Result<gossipsub::Behaviour, P2pError> {
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(1))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .build()
        .map_err(|e| P2pError::GossipsubBuild(e.to_string()))?;

    gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(keypair.clone()),
        gossipsub_config,
    )
    .map_err(|e| P2pError::GossipsubBuild(e.to_string()))
}

pub fn build_identify(keypair: &identity::Keypair) -> identify::Behaviour {
    identify::Behaviour::new(identify::Config::new(
        IDENTIFY_PROTOCOL_VERSION.to_string(),
        keypair.public(),
    ))
}

/// Builds a fully wired TCP+Noise+Yamux swarm with the AION behaviour set.
/// This is the entry point `crates/aion-node` will call once node/P2P
/// wiring happens (not yet done -- this crate is still standalone).
pub fn build_swarm(keypair: identity::Keypair) -> Result<Swarm<AionBehaviour>, P2pError> {
    let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .map_err(|e| P2pError::SwarmBuild(e.to_string()))?
        .with_behaviour(|key| {
            let gossipsub = build_gossipsub(key)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            let identify = build_identify(key);
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(AionBehaviour {
                gossipsub,
                identify,
            })
        })
        .map_err(|e| P2pError::SwarmBuild(e.to_string()))?
        .build();
    Ok(swarm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinct_keypairs_produce_distinct_peer_ids() {
        let a = identity::Keypair::generate_ed25519();
        let b = identity::Keypair::generate_ed25519();
        assert_ne!(local_peer_id(&a), local_peer_id(&b));
    }

    #[test]
    fn same_keypair_is_deterministic_peer_id() {
        let a = identity::Keypair::generate_ed25519();
        assert_eq!(local_peer_id(&a), local_peer_id(&a));
    }

    #[test]
    fn gossipsub_behaviour_builds_successfully() {
        let keypair = identity::Keypair::generate_ed25519();
        assert!(build_gossipsub(&keypair).is_ok());
    }
}
