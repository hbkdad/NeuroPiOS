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
