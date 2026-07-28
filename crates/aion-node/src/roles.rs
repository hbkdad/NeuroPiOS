//! Composable node role flags, per docs/ARCHITECTURE.md: "a single physical
//! device may run multiple roles simultaneously... roles are capability
//! flags, not separate binaries."

use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeRoleKind {
    Compute,
    Inference,
    Training,
    Storage,
    Validator,
    Benchmark,
    Gateway,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NodeRoles(BTreeSet<NodeRoleKind>);

impl NodeRoles {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, role: NodeRoleKind) -> Self {
        self.0.insert(role);
        self
    }

    pub fn has(&self, role: NodeRoleKind) -> bool {
        self.0.contains(&role)
    }

    pub fn count(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &NodeRoleKind> {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use NodeRoleKind::*;

    #[test]
    fn a_node_can_hold_multiple_roles_simultaneously() {
        let roles = NodeRoles::new().with(Compute).with(Inference).with(Storage);
        assert!(roles.has(Compute));
        assert!(roles.has(Inference));
        assert!(roles.has(Storage));
        assert!(!roles.has(Validator));
        assert_eq!(roles.count(), 3);
    }

    #[test]
    fn adding_the_same_role_twice_does_not_duplicate() {
        let roles = NodeRoles::new().with(Compute).with(Compute);
        assert_eq!(roles.count(), 1);
    }

    #[test]
    fn empty_roles_by_default() {
        let roles = NodeRoles::new();
        assert_eq!(roles.count(), 0);
    }
}
