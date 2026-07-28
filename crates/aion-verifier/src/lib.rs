//! AION verification tiers L0-L6, per docs/VERIFICATION.md and
//! docs/adr/0003-verification-tiering.md.
//!
//! Status: L1 (commitment), L2 (replicated execution with numeric
//! tolerance), L3 (deterministic execution via a real `wasmi` WASM
//! sandbox, see deterministic.rs's module doc for scope), L4 (optimistic
//! dispute state machine -- not the bisection arbitration itself, see
//! dispute.rs's module doc), and L6 (commit/reveal weighted quorum) are
//! implemented as real, tested logic. L5 (ZKML) is not implemented here.

pub mod commitment;
pub mod deterministic;
pub mod dispute;
pub mod quorum;
pub mod replicated;
pub mod tier;

pub use commitment::check_l1;
pub use deterministic::{
    compare_replicated_outputs, execute_deterministic, verify_deterministic_replication,
    DeterministicExecutionError, DeterministicReplicationOutcome, DeterministicVerificationError,
};
pub use dispute::{ChallengeError, ChallengeState, ChallengeWindow};
pub use quorum::{Quorum, QuorumError};
pub use replicated::{evaluate_replicated_results, ReplicatedResult, ReplicationOutcome};
pub use tier::{adaptive_verification_rate, verification_confidence, Tier};
