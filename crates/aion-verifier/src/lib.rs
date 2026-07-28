//! AION verification tiers L0-L6, per docs/VERIFICATION.md and
//! docs/adr/0003-verification-tiering.md.
//!
//! Status: L1 (commitment), L2 (replicated execution with numeric
//! tolerance), and L6 (commit/reveal weighted quorum) are implemented as
//! real, tested logic. L3 (deterministic execution), L4 (optimistic
//! dispute/bisection), and L5 (ZKML) are not implemented here.

pub mod commitment;
pub mod quorum;
pub mod replicated;
pub mod tier;

pub use commitment::check_l1;
pub use quorum::{Quorum, QuorumError};
pub use replicated::{evaluate_replicated_results, ReplicatedResult, ReplicationOutcome};
pub use tier::{adaptive_verification_rate, verification_confidence, Tier};
