//! AION cryptographic primitives: Ed25519 P2P identity signing and
//! SHA-256 commit/reveal. Reviewed, audited libraries only -- see
//! AGENTS.md's "never invent cryptography" rule and
//! .claude/agents/aion-crypto.md.

pub mod commit_reveal;
pub mod identity;

pub use commit_reveal::{commit, verify_reveal, Commitment};
pub use identity::{verify, Identity, SignatureError};
