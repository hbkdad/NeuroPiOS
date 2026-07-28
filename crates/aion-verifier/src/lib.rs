//! AION verification tiers L0-L6, per docs/VERIFICATION.md and
//! docs/adr/0003-verification-tiering.md.
//!
//! Status: L1 (commitment) and L2 (replicated execution with numeric
//! tolerance) are implemented as real, tested logic. L3 (deterministic
//! execution), L4 (optimistic dispute/bisection), L5 (ZKML), and L6
//! (multi-validator quorum -- partially covered already by
//! `simulations/poig_sim/poig_sim/quorum.py`, not yet ported to Rust) are
//! not implemented here.

pub mod commitment;
pub mod replicated;
pub mod tier;

pub use commitment::check_l1;
pub use replicated::{evaluate_replicated_results, ReplicatedResult, ReplicationOutcome};
pub use tier::{adaptive_verification_rate, verification_confidence, Tier};
