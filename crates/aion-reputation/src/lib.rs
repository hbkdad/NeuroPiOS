//! AION multi-dimensional decaying reputation store and PoIG reward
//! computation, per docs/POIG-SPEC.md and docs/ARCHITECTURE.md.
//!
//! Ported from and pinned against the validated Python reference at
//! `simulations/poig_sim/poig_sim/reputation.py` and `poig.py` --
//! `simulations/poig_sim/README.md` describes what those modules were
//! already proven to do via `market.py`-orchestrated integration tests
//! (Sybil-cluster resistance, self-dealing gate, wash-trading bound).
//! This crate re-implements the same logic in real Rust with dedicated
//! direct unit tests on top (the Python reference only exercised this
//! logic indirectly through `market.py`'s job orchestration).
//!
//! Status: `reputation` and `poig` are real, tested Rust. NOT done: this
//! crate has no job-orchestration layer of its own (that remains
//! `market.py`-only, Python side) -- `compute_reward`'s
//! `verification_confidence_value` and `reputation_factor_value` inputs
//! are the caller's responsibility to source from `crates/aion-verifier`
//! and this crate's own `ReputationStore` respectively, exactly as the
//! Python reference takes them as already-computed inputs too.

pub mod poig;
pub mod reputation;

pub use poig::{
    compute_reward, demand_factor, useful_work_score, ComputeRewardInput, RewardBreakdown,
    UsefulWorkResult, DEMAND_FACTOR_CAP,
};
pub use reputation::{
    ReputationRecord, ReputationStore, BASELINE_SCORE, DECAY_PER_TICK_IDLE, MAX_SCORE,
};
