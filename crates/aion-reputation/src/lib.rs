//! AION multi-dimensional decaying reputation store and PoIG reward
//! computation, per docs/POIG-SPEC.md and docs/ARCHITECTURE.md.
//!
//! Ported from and pinned against the validated Python reference at
//! `simulations/poig_sim/poig_sim/reputation.py`, `poig.py`, and
//! `routing.py` -- `simulations/poig_sim/README.md` describes what those
//! modules were already proven to do via `market.py`-orchestrated
//! integration tests (Sybil-cluster resistance, self-dealing gate,
//! wash-trading bound, price-manipulation leverage cap). This crate
//! re-implements the same logic in real Rust with dedicated direct unit
//! tests on top (the Python reference only exercised `reputation.py` and
//! `routing.py` indirectly through `market.py`'s job orchestration).
//!
//! Status: `reputation`, `poig`, and `routing` are real, tested Rust. NOT
//! done: this crate has no job-orchestration layer of its own (that
//! remains `market.py`-only, Python side) -- `compute_reward`'s
//! `verification_confidence_value` and `reputation_factor_value` inputs
//! are the caller's responsibility to source from `crates/aion-verifier`
//! and this crate's own `ReputationStore` respectively, exactly as the
//! Python reference takes them as already-computed inputs too.

pub mod poig;
pub mod reputation;
pub mod routing;

pub use poig::{
    compute_reward, demand_factor, useful_work_score, ComputeRewardInput, RewardBreakdown,
    UsefulWorkResult, DEMAND_FACTOR_CAP,
};
pub use reputation::{
    ReputationRecord, ReputationStore, BASELINE_SCORE, DECAY_PER_TICK_IDLE, MAX_SCORE,
};
pub use routing::{
    max_score_from_single_signal, routing_score, RoutingSignal, RoutingSignals, RoutingWeights,
};
