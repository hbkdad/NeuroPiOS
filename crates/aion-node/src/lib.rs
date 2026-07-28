//! AION node runtime: identity, hardware detection, composable roles, and
//! mandatory resource-safety caps. See docs/ARCHITECTURE.md's Node
//! Architecture and Node Safety sections.

pub mod hardware;
pub mod resource_caps;
pub mod roles;

pub use hardware::HardwareProfile;
pub use resource_caps::ResourceCaps;
pub use roles::{NodeRoleKind, NodeRoles};

use aion_crypto::Identity;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NodeError {
    #[error("failed to start P2P layer: {0}")]
    P2p(#[from] aion_p2p::P2pError),
}

/// A node's P2P identity, kept separate from any wallet identity per
/// docs/ARCHITECTURE.md's Identity Separation section -- the wallet
/// identity lives with the chosen L2's account-abstraction tooling
/// (docs/adr/0001-settlement-layer.md), not in this struct, so a node's
/// P2P PeerID is never bound 1:1 to a wallet address at this layer.
pub struct Node {
    pub p2p_identity: Identity,
    pub hardware: HardwareProfile,
    pub caps: ResourceCaps,
    pub roles: NodeRoles,
}

impl Node {
    /// Detects real hardware, generates a fresh P2P identity, and starts
    /// with `ResourceCaps::locked_down()` -- a node never begins accepting
    /// work until the operator explicitly widens its caps.
    pub fn bootstrap(roles: NodeRoles) -> Self {
        Node {
            p2p_identity: Identity::generate(),
            hardware: HardwareProfile::detect(),
            caps: ResourceCaps::locked_down(),
            roles,
        }
    }

    pub fn p2p_public_key_bytes(&self) -> [u8; 32] {
        self.p2p_identity.public_key_bytes()
    }

    /// Builds a P2P swarm using THIS node's own identity (bridged via
    /// `aion_p2p::keypair_from_identity`, per docs/ARCHITECTURE.md's
    /// Identity Separation section) -- the node's PeerID is genuinely
    /// derived from its `p2p_identity`, not a separately-generated key.
    /// This does not start listening or dialing anything; the caller
    /// drives the returned swarm (see crates/aion-p2p's tests for the
    /// event-loop pattern this is meant to be driven with).
    pub fn build_swarm(&self) -> Result<libp2p::swarm::Swarm<aion_p2p::AionBehaviour>, NodeError> {
        let keypair = aion_p2p::keypair_from_identity(&self.p2p_identity)?;
        Ok(aion_p2p::build_swarm(keypair)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use roles::NodeRoleKind;

    #[test]
    fn a_freshly_bootstrapped_node_accepts_no_work_by_default() {
        let node = Node::bootstrap(NodeRoles::new().with(NodeRoleKind::Compute));
        assert!(!node.caps.allows_new_work(0.0, 0, 12));
    }

    #[test]
    fn bootstrap_detects_real_hardware_and_generates_a_usable_identity() {
        let node = Node::bootstrap(NodeRoles::new().with(NodeRoleKind::Inference));
        assert!(node.hardware.cpu_cores >= 1);
        // identity is actually usable for signing, not a placeholder
        let sig = node.p2p_identity.sign(b"hello");
        assert!(aion_crypto::verify(&node.p2p_public_key_bytes(), b"hello", &sig).is_ok());
    }

    #[test]
    fn two_bootstrapped_nodes_get_distinct_identities() {
        let a = Node::bootstrap(NodeRoles::new());
        let b = Node::bootstrap(NodeRoles::new());
        assert_ne!(a.p2p_public_key_bytes(), b.p2p_public_key_bytes());
    }
}
