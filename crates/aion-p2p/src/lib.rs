//! AION P2P networking, per docs/adr/0002-p2p-stack.md: rust-libp2p on
//! Tokio. GossipSub (job/announcement propagation), Identify, and
//! Kademlia (peer discovery) over TCP+Noise+Yamux. AutoNAT and circuit
//! relay (NAT traversal) are still deferred to a follow-up.
//!
//! The P2P identity keypair can be either a fresh libp2p-generated Ed25519
//! key (`identity::Keypair::generate_ed25519()`) or bridged from an
//! `aion_crypto::Identity` via `keypair_from_identity` -- the latter is
//! what `crates/aion-node` uses, so a node's P2P PeerID and its
//! `aion_crypto::Identity` (docs/ARCHITECTURE.md's Identity Separation
//! section) are genuinely the same key material, not two unrelated keys
//! that happen to both be called "identity".

use libp2p::{
    gossipsub, identify, identity, kad, noise,
    swarm::{NetworkBehaviour, Swarm},
    tcp, yamux, Multiaddr, PeerId,
};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum P2pError {
    #[error("failed to build gossipsub behaviour: {0}")]
    GossipsubBuild(String),
    #[error("transport/swarm build error: {0}")]
    SwarmBuild(String),
    #[error("failed to bridge aion-crypto identity into a libp2p keypair: {0}")]
    IdentityBridge(String),
}

#[derive(NetworkBehaviour)]
pub struct AionBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub identify: identify::Behaviour,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
}

pub const IDENTIFY_PROTOCOL_VERSION: &str = "/aion/identify/0.1.0";
/// A custom (non-default) Kademlia protocol name, deliberately distinct
/// from libp2p's default `/ipfs/kad/1.0.0` so an AION node never
/// accidentally joins or pollutes the public IPFS DHT.
pub const KADEMLIA_PROTOCOL_NAME: &str = "/aion/kad/0.1.0";

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

/// Bridges an `aion_crypto::Identity`'s key material into a
/// `libp2p::identity::Keypair`, so the node's P2P PeerID is derived from
/// the SAME Ed25519 secret as its `aion_crypto::Identity`.
pub fn keypair_from_identity(
    identity: &aion_crypto::Identity,
) -> Result<identity::Keypair, P2pError> {
    let mut secret_bytes = identity.to_bytes();
    identity::Keypair::ed25519_from_bytes(&mut secret_bytes)
        .map_err(|e| P2pError::IdentityBridge(e.to_string()))
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

pub fn build_kademlia(local_peer_id: PeerId) -> kad::Behaviour<kad::store::MemoryStore> {
    let store = kad::store::MemoryStore::new(local_peer_id);
    let mut config = kad::Config::default();
    config.set_protocol_names(vec![libp2p::StreamProtocol::new(KADEMLIA_PROTOCOL_NAME)]);
    kad::Behaviour::with_config(local_peer_id, store, config)
}

/// Builds a fully wired TCP+Noise+Yamux swarm with the AION behaviour set
/// (GossipSub + Identify + Kademlia). This is the entry point
/// `crates/aion-node` uses to actually start a node's P2P layer.
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
            let kademlia = build_kademlia(PeerId::from(key.public()));
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(AionBehaviour {
                gossipsub,
                identify,
                kademlia,
            })
        })
        .map_err(|e| P2pError::SwarmBuild(e.to_string()))?
        .build();
    Ok(swarm)
}

/// Feeds addresses learned via Identify into Kademlia's routing table.
/// Kademlia does not automatically learn peer addresses from Identify --
/// a caller's event loop must bridge the two explicitly by calling this
/// on every `identify::Event::Received`. This is the standard integration
/// pattern for combining these two protocols in libp2p.
pub fn feed_identify_into_kademlia(
    kademlia: &mut kad::Behaviour<kad::store::MemoryStore>,
    event: &identify::Event,
) {
    if let identify::Event::Received { peer_id, info, .. } = event {
        for addr in &info.listen_addrs {
            kademlia.add_address(peer_id, addr.clone());
        }
    }
}

/// Manually seeds a peer into Kademlia's routing table. Every DHT needs at
/// least one already-known peer to bootstrap from -- this is not a
/// workaround, it's inherent to how Kademlia (and every other DHT) joins a
/// network for the first time.
pub fn add_kademlia_bootstrap_peer(
    kademlia: &mut kad::Behaviour<kad::store::MemoryStore>,
    peer_id: PeerId,
    addr: Multiaddr,
) {
    kademlia.add_address(&peer_id, addr);
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

    #[test]
    fn bridged_identity_produces_the_same_peer_id_as_its_aion_crypto_public_key() {
        let identity = aion_crypto::Identity::generate();
        let keypair = keypair_from_identity(&identity).unwrap();

        // Not just "deterministic" -- the ACTUAL key material must match:
        // the libp2p keypair's public key bytes must equal
        // aion_crypto::Identity's own public key bytes, proving this is
        // genuinely the same Ed25519 key, not just a reproducibly-derived
        // different one.
        let libp2p_public_bytes = keypair
            .public()
            .try_into_ed25519()
            .expect("bridged keypair should be Ed25519")
            .to_bytes();
        assert_eq!(libp2p_public_bytes, identity.public_key_bytes());

        // Deterministic: bridging the same aion_crypto::Identity twice
        // (by reconstructing from its exported bytes) yields the same PeerId.
        let identity2 = aion_crypto::Identity::from_bytes(&identity.to_bytes());
        let keypair2 = keypair_from_identity(&identity2).unwrap();
        assert_eq!(local_peer_id(&keypair), local_peer_id(&keypair2));
    }

    #[test]
    fn swarm_builds_successfully_from_a_bridged_identity() {
        let identity = aion_crypto::Identity::generate();
        let keypair = keypair_from_identity(&identity).unwrap();
        assert!(build_swarm(keypair).is_ok());
    }
}
