//! AION P2P networking, per docs/adr/0002-p2p-stack.md: rust-libp2p on
//! Tokio. GossipSub (job/announcement propagation), Identify, Kademlia
//! (peer discovery), AutoNAT (reachability detection), and circuit relay
//! (the actual NAT-traversal transport AutoNAT alone can only detect the
//! need for) over TCP+Noise+Yamux.
//!
//! Every node built by this crate gets BOTH relay roles -- relay-client
//! (so it can dial an otherwise-unreachable peer through a relay) and
//! relay-server (so it can help relay for others) -- rather than treating
//! relaying as opt-in. This matches the same "no free-riders" choice
//! already made for Kademlia (every node is forced into server mode, per
//! `build_kademlia`'s doc), not a new policy invented for relay.
//!
//! The P2P identity keypair can be either a fresh libp2p-generated Ed25519
//! key (`identity::Keypair::generate_ed25519()`) or bridged from an
//! `aion_crypto::Identity` via `keypair_from_identity` -- the latter is
//! what `crates/aion-node` uses, so a node's P2P PeerID and its
//! `aion_crypto::Identity` (docs/ARCHITECTURE.md's Identity Separation
//! section) are genuinely the same key material, not two unrelated keys
//! that happen to both be called "identity".

use libp2p::{
    autonat, connection_limits, gossipsub, identify, identity, kad, noise, relay,
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
    pub connection_limits: connection_limits::Behaviour,
    pub autonat: autonat::Behaviour,
    /// Relay-server role: this node can act as a relay for other peers.
    pub relay: relay::Behaviour,
    /// Relay-client role: this node can dial other peers THROUGH a relay
    /// (a `/p2p-circuit` address) when a direct connection isn't possible.
    pub relay_client: relay::client::Behaviour,
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

/// Deliberate, non-default topic scoring parameters for the job-announcements
/// topic, per docs/adr/0002-p2p-stack.md's requirement that every gossip
/// topic get an explicit peer-scoring policy, not library defaults left
/// untouched. Values are a documented starting point -- like every other
/// weight/threshold in this codebase (see docs/POIG-SPEC.md's equivalent
/// caveat), they need real-network-data tuning before being treated as
/// production-final, not assumed correct because they compile.
pub fn job_announcements_topic_score_params() -> gossipsub::TopicScoreParams {
    gossipsub::TopicScoreParams {
        topic_weight: 1.0,
        // P1: reward peers who have been meshed in longer -- a crude but
        // real Sybil-resistance signal (fresh identities can't instantly
        // accrue mesh tenure), consistent with docs/VERIFICATION.md's
        // "new/unproven identities get less trust" principle.
        time_in_mesh_weight: 1.0,
        time_in_mesh_quantum: Duration::from_secs(1),
        time_in_mesh_cap: 300.0, // caps out after 5 minutes of good standing
        // P2: reward peers who are first to deliver a valid message --
        // rewards genuinely well-connected/useful relayers.
        first_message_deliveries_weight: 1.0,
        first_message_deliveries_decay: 0.5,
        first_message_deliveries_cap: 50.0,
        // P3: penalize a meshed peer who isn't actually delivering
        // messages promptly (a peer that joined the mesh but free-rides
        // or withholds traffic). Activation delay avoids punishing a peer
        // in its first few seconds of membership.
        mesh_message_deliveries_weight: -1.0,
        mesh_message_deliveries_decay: 0.5,
        mesh_message_deliveries_cap: 50.0,
        mesh_message_deliveries_threshold: 5.0,
        mesh_message_deliveries_window: Duration::from_millis(500),
        mesh_message_deliveries_activation: Duration::from_secs(10),
        // P3b: sticky penalty for peers pruned while under-delivering --
        // discourages "join, do nothing, get pruned, immediately rejoin."
        mesh_failure_penalty_weight: -1.0,
        mesh_failure_penalty_decay: 0.5,
        // P4: the core anti-spam signal -- publishing invalid/malformed
        // job-announcement messages is penalized quadratically, so a
        // handful of mistakes are forgiven but sustained spam compounds
        // fast.
        invalid_message_deliveries_weight: -20.0,
        invalid_message_deliveries_decay: 0.3,
    }
}

/// Peer-score parameters and thresholds for the whole `AionBehaviour`
/// GossipSub instance. `app_specific_weight` is left available for
/// `crates/aion-reputation` to plug AION's own multi-dimensional
/// reputation score (docs/security/THREAT-MODEL.md) into gossipsub's
/// scoring in a later phase -- not wired yet, since that reputation store
/// doesn't exist as real Rust code outside the Phase 2 Python simulator.
pub fn build_peer_score_params() -> (gossipsub::PeerScoreParams, gossipsub::PeerScoreThresholds) {
    let mut params = gossipsub::PeerScoreParams::default();
    params.topics.insert(
        Topic::new(JOB_ANNOUNCEMENTS_TOPIC).0.hash(),
        job_announcements_topic_score_params(),
    );
    (params, gossipsub::PeerScoreThresholds::default())
}

/// Builds a GossipSub behaviour with sane, explicit defaults (message
/// signing required -- anonymous/unsigned gossip is not permitted, since
/// every AION gossip message should be attributable to a peer for the
/// anti-spam/reputation layers described in docs/security/THREAT-MODEL.md)
/// and real peer-scoring activated (not left as library defaults) per
/// docs/adr/0002-p2p-stack.md's anti-spam requirement.
pub fn build_gossipsub(keypair: &identity::Keypair) -> Result<gossipsub::Behaviour, P2pError> {
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(1))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .build()
        .map_err(|e| P2pError::GossipsubBuild(e.to_string()))?;

    let mut behaviour = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(keypair.clone()),
        gossipsub_config,
    )
    .map_err(|e| P2pError::GossipsubBuild(e.to_string()))?;

    let (score_params, score_thresholds) = build_peer_score_params();
    behaviour
        .with_peer_score(score_params, score_thresholds)
        .map_err(P2pError::GossipsubBuild)?;

    Ok(behaviour)
}

pub fn build_identify(keypair: &identity::Keypair) -> identify::Behaviour {
    identify::Behaviour::new(identify::Config::new(
        IDENTIFY_PROTOCOL_VERSION.to_string(),
        keypair.public(),
    ))
}

/// Builds Kademlia in **server mode** (`kad::Mode::Server`) explicitly,
/// rather than relying on the library default (`Mode::Client`, which only
/// switches to server mode automatically once an external address is
/// confirmed -- e.g. via AutoNAT, which this crate doesn't implement yet,
/// see the module doc). A client-mode node issues queries but silently
/// refuses to answer other peers' FIND_NODE requests (its handler denies
/// the inbound substream upgrade outright), which would make every AION
/// node a DHT free-rider and break discovery entirely. AION nodes are
/// expected to be reachable compute/storage/validator participants, so
/// defaulting to server mode is the correct choice here -- a node that
/// truly can't accept inbound connections (e.g. behind a NAT with no port
/// forwarding) is a separate, real problem that AutoNAT/relay (still
/// deferred) will need to solve, not something this default should mask.
pub fn build_kademlia(local_peer_id: PeerId) -> kad::Behaviour<kad::store::MemoryStore> {
    let store = kad::store::MemoryStore::new(local_peer_id);
    let mut config = kad::Config::default();
    config.set_protocol_names(vec![libp2p::StreamProtocol::new(KADEMLIA_PROTOCOL_NAME)]);
    let mut behaviour = kad::Behaviour::with_config(local_peer_id, store, config);
    behaviour.set_mode(Some(kad::Mode::Server));
    behaviour
}

/// AutoNAT's production default `Config`: `only_global_ips: true` (the
/// correct setting for real internet-facing nodes -- a private/loopback
/// address should never be treated as evidence of public reachability).
/// Tests that run entirely over `127.0.0.1` must override this (see
/// `tests/autonat_reachability.rs`) since a real deployment's assumption
/// doesn't hold on loopback -- that override is a test-environment
/// necessity, not a relaxation of the production default.
pub fn default_autonat_config() -> autonat::Config {
    autonat::Config::default()
}

/// Builds AutoNAT (per docs/adr/0002-p2p-stack.md's still-deferred NAT
/// item, now started): a single `autonat::Behaviour` acts as BOTH a
/// dial-back-requesting client (to learn whether the local node's own
/// address is publicly reachable) and a dial-back-performing server (to
/// help other peers learn the same about themselves) -- this dual role is
/// the library's own design, not an AION-specific simplification. This is
/// reachability DETECTION only; it does not itself provide a NAT-traversal
/// transport (that's circuit relay, still not implemented -- see the
/// crate-level module doc and README's "Not yet done" section).
pub fn build_autonat(local_peer_id: PeerId, config: autonat::Config) -> autonat::Behaviour {
    autonat::Behaviour::new(local_peer_id, config)
}

/// Builds the relay-SERVER role (this node can act as a relay for other
/// peers, holding reservations and forwarding circuit traffic on their
/// behalf) with the library's own default rate limits (per-peer and
/// per-IP reservation/circuit caps) -- a starting point, not tuned
/// against real network load, same caveat as every other default in this
/// crate.
pub fn build_relay_server(local_peer_id: PeerId) -> relay::Behaviour {
    relay::Behaviour::new(local_peer_id, relay::Config::default())
}

/// Conservative default connection limits, per docs/adr/0002-p2p-stack.md's
/// anti-spam requirement and docs/ARCHITECTURE.md's Node Safety principle
/// extended to network resources, not just CPU/memory/storage: a node
/// never accepts unbounded connections just because gossipsub/kademlia
/// alone don't cap them. `max_established_incoming` is set independently
/// from the outgoing/per-peer/total caps specifically to prevent an
/// eclipse attack (an adversary surrounding a node with only inbound
/// connections it controls) -- per this behaviour's own documented
/// recommendation. These are starting values, not tuned against real
/// network load; see the same tunable-not-final caveat that applies to
/// every other weight/threshold in this codebase.
pub fn default_connection_limits() -> connection_limits::ConnectionLimits {
    connection_limits::ConnectionLimits::default()
        .with_max_pending_incoming(Some(30))
        .with_max_pending_outgoing(Some(30))
        .with_max_established_incoming(Some(100))
        .with_max_established_outgoing(Some(100))
        .with_max_established_per_peer(Some(4))
        .with_max_established(Some(200))
}

/// Builds a fully wired TCP+Noise+Yamux swarm with the AION behaviour set
/// (GossipSub + Identify + Kademlia + connection limits + AutoNAT) and the
/// default connection limits and AutoNAT config above. This is the entry
/// point `crates/aion-node` uses to actually start a node's P2P layer.
pub fn build_swarm(keypair: identity::Keypair) -> Result<Swarm<AionBehaviour>, P2pError> {
    build_swarm_with_limits(keypair, default_connection_limits())
}

/// Same as `build_swarm`, but with caller-supplied connection limits --
/// exists so tests (and, later, `aion-node`'s operator-configured
/// resource caps) can override the defaults rather than being stuck with
/// them. Uses the production-default AutoNAT config; see
/// `build_swarm_with_limits_and_autonat_config` to override that too.
pub fn build_swarm_with_limits(
    keypair: identity::Keypair,
    limits: connection_limits::ConnectionLimits,
) -> Result<Swarm<AionBehaviour>, P2pError> {
    build_swarm_with_limits_and_autonat_config(keypair, limits, default_autonat_config())
}

/// Same as `build_swarm_with_limits`, but also with a caller-supplied
/// AutoNAT config -- exists specifically so loopback-only integration
/// tests can set `only_global_ips: false` (see
/// `tests/autonat_reachability.rs`) without weakening the production
/// default that every other caller gets.
pub fn build_swarm_with_limits_and_autonat_config(
    keypair: identity::Keypair,
    limits: connection_limits::ConnectionLimits,
    autonat_config: autonat::Config,
) -> Result<Swarm<AionBehaviour>, P2pError> {
    let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .map_err(|e| P2pError::SwarmBuild(e.to_string()))?
        .with_relay_client(noise::Config::new, yamux::Config::default)
        .map_err(|e| P2pError::SwarmBuild(e.to_string()))?
        .with_behaviour(|key, relay_client| {
            let gossipsub = build_gossipsub(key)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            let identify = build_identify(key);
            let local_peer_id = PeerId::from(key.public());
            let kademlia = build_kademlia(local_peer_id);
            let connection_limits = connection_limits::Behaviour::new(limits);
            let autonat = build_autonat(local_peer_id, autonat_config);
            let relay = build_relay_server(local_peer_id);
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(AionBehaviour {
                gossipsub,
                identify,
                kademlia,
                connection_limits,
                autonat,
                relay,
                relay_client,
            })
        })
        .map_err(|e| P2pError::SwarmBuild(e.to_string()))?
        // libp2p-swarm's default `idle_connection_timeout` is `Duration::ZERO`:
        // the instant no behaviour handler reports pending work on a
        // connection, the swarm tears it down with no grace period at all.
        // For a request/response protocol like Kademlia, the RESPONDER side
        // can report "no more work" as soon as it has queued its reply,
        // which races the reply actually being flushed to the socket -- the
        // connection can close before the bytes go out, silently dropping
        // an otherwise-correct response. Root-caused via
        // RUST_LOG=libp2p_kad=trace tracing of a failing 3-node relayed
        // discovery case (see crates/aion-p2p/README.md): B genuinely
        // computed and queued a FIND_NODE response
        // (`InboundRequest { FindNode { num_closer_peers: 1 } }`), but the
        // B<->C connection closed via `KeepAliveTimeout` ~90ms later, before
        // the requester ever received it, and the query failed
        // (`requests: 1, success: 0, failure: 1`). A nonzero grace period
        // gives in-flight protocol responses time to actually be written
        // before an idle connection is reclaimed.
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(30)))
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
    fn gossipsub_behaviour_builds_successfully_with_peer_scoring_active() {
        let keypair = identity::Keypair::generate_ed25519();
        assert!(build_gossipsub(&keypair).is_ok());
    }

    #[test]
    fn peer_score_params_are_valid_per_gossipsubs_own_validation() {
        let (params, thresholds) = build_peer_score_params();
        assert!(params.validate().is_ok());
        assert!(thresholds.validate().is_ok());
    }

    #[test]
    fn job_announcements_topic_has_explicit_non_default_scoring_not_left_as_library_defaults() {
        let configured = job_announcements_topic_score_params();
        let library_default = gossipsub::TopicScoreParams::default();
        // The point of this crate's peer-scoring work is that every topic
        // gets a DELIBERATE policy, per docs/adr/0002-p2p-stack.md -- not
        // that gossipsub's library defaults happen to be used untouched.
        assert_ne!(
            configured.invalid_message_deliveries_weight,
            library_default.invalid_message_deliveries_weight,
            "job-announcements topic should have a deliberately-tuned anti-spam penalty, not the library default"
        );
        // Sign/direction sanity per gossipsub's own documented invariants:
        // reward weights must be >= 0, penalty weights must be <= 0.
        assert!(configured.time_in_mesh_weight >= 0.0);
        assert!(configured.first_message_deliveries_weight >= 0.0);
        assert!(configured.mesh_message_deliveries_weight <= 0.0);
        assert!(configured.mesh_failure_penalty_weight <= 0.0);
        assert!(configured.invalid_message_deliveries_weight <= 0.0);
    }

    #[test]
    fn build_peer_score_params_registers_the_job_announcements_topic_specifically() {
        let (params, _) = build_peer_score_params();
        let topic_hash = Topic::new(JOB_ANNOUNCEMENTS_TOPIC).0.hash();
        assert!(params.topics.contains_key(&topic_hash));
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
