//! PoIG reward calculation per docs/POIG-SPEC.md. Ported from and pinned
//! against the validated Python reference at
//! `simulations/poig_sim/poig_sim/poig.py`.
//!
//! ```text
//! Reward = BaseValue * UsefulWorkScore * VerificationConfidence * DemandFactor
//!          * QualityFactor * ImprovementFactor * ReputationFactor * EfficiencyFactor
//! ```
//!
//! This is the load-bearing anti-fraud logic: `UsefulWorkScore` is the gate
//! that enforces the orchestration brief's core security rule ("never
//! equate computation occurred with useful AI computation occurred"). It is
//! only nonzero when the job traces back to real, signed, escrow-backed
//! external demand from a requester distinct from the worker, or to network
//! benchmark provenance.
//!
//! Honest scope note: this module ports `poig.py` only, not
//! `simulations/poig_sim/poig_sim/market.py`'s job-orchestration layer
//! (job creation, escrow bookkeeping, verification-tier dispatch) --
//! `verification_confidence_value` and `reputation_factor_value` are taken
//! as inputs here exactly as the Python reference does, sourced from
//! `crates/aion-verifier`'s `tier` module and this crate's own
//! `reputation` module respectively once a real caller wires them
//! together (not yet done -- no orchestration layer exists in Rust yet).

/// `DemandFactor` is capped to prevent a thin/manipulable demand signal
/// (e.g. a single large self-funded escrow) from unboundedly amplifying
/// reward, per docs/security/ECONOMIC-ATTACKS.md's "fake demand / wash
/// activity" row.
pub const DEMAND_FACTOR_CAP: f64 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UsefulWorkResult {
    pub score: f64,
    pub reason: &'static str,
}

/// The anti-farming gate. Returns `0.0` unless the work traces to real,
/// independent, signed, escrow-backed demand (or network-benchmark
/// provenance).
pub fn useful_work_score(
    worker_id: &str,
    requester_id: &str,
    escrow_funded: bool,
    signature_valid: bool,
    is_network_benchmark: bool,
) -> UsefulWorkResult {
    if is_network_benchmark {
        return UsefulWorkResult {
            score: 1.0,
            reason: "network-issued benchmark provenance",
        };
    }
    if !signature_valid {
        return UsefulWorkResult {
            score: 0.0,
            reason: "requester signature invalid",
        };
    }
    if !escrow_funded {
        return UsefulWorkResult {
            score: 0.0,
            reason: "no funded escrow backing this job",
        };
    }
    if worker_id == requester_id {
        // Self-dealing: the same identity is both requester and worker.
        // This alone is disqualifying regardless of escrow, because the
        // "external demand" this factor is supposed to certify does not
        // exist when the requester and the worker are the same actor.
        return UsefulWorkResult {
            score: 0.0,
            reason: "requester and worker are the same identity (self-dealing)",
        };
    }
    UsefulWorkResult {
        score: 1.0,
        reason: "signed, escrow-backed job from a distinct requester",
    }
}

pub fn demand_factor(escrow_amount: f64, base_value: f64) -> f64 {
    if base_value <= 0.0 {
        return 0.0;
    }
    let raw = escrow_amount / base_value;
    raw.clamp(0.0, DEMAND_FACTOR_CAP)
}

#[derive(Debug, Clone, PartialEq)]
pub struct RewardBreakdown {
    pub reward: f64,
    pub useful_work_score: f64,
    pub useful_work_reason: &'static str,
    pub verification_confidence: f64,
    pub demand_factor: f64,
    pub quality_factor: f64,
    pub improvement_factor: f64,
    pub reputation_factor: f64,
    pub efficiency_factor: f64,
}

/// Inputs to [`compute_reward`]. `quality_factor`, `improvement_factor`,
/// and `efficiency_factor` default to `1.0` and `is_network_benchmark`
/// defaults to `false` via [`ComputeRewardInput::new`], matching the
/// Python reference's keyword-argument defaults -- override the public
/// fields directly for the non-default cases.
pub struct ComputeRewardInput<'a> {
    pub base_value: f64,
    pub worker_id: &'a str,
    pub requester_id: &'a str,
    pub escrow_amount: f64,
    pub escrow_funded: bool,
    pub signature_valid: bool,
    pub verification_confidence_value: f64,
    pub reputation_factor_value: f64,
    pub quality_factor: f64,
    pub improvement_factor: f64,
    pub efficiency_factor: f64,
    pub is_network_benchmark: bool,
}

impl<'a> ComputeRewardInput<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        base_value: f64,
        worker_id: &'a str,
        requester_id: &'a str,
        escrow_amount: f64,
        escrow_funded: bool,
        signature_valid: bool,
        verification_confidence_value: f64,
        reputation_factor_value: f64,
    ) -> Self {
        Self {
            base_value,
            worker_id,
            requester_id,
            escrow_amount,
            escrow_funded,
            signature_valid,
            verification_confidence_value,
            reputation_factor_value,
            quality_factor: 1.0,
            improvement_factor: 1.0,
            efficiency_factor: 1.0,
            is_network_benchmark: false,
        }
    }
}

pub fn compute_reward(input: &ComputeRewardInput) -> RewardBreakdown {
    let uws = useful_work_score(
        input.worker_id,
        input.requester_id,
        input.escrow_funded,
        input.signature_valid,
        input.is_network_benchmark,
    );
    let dfactor = demand_factor(input.escrow_amount, input.base_value);

    let reward = input.base_value
        * uws.score
        * input.verification_confidence_value
        * dfactor
        * input.quality_factor
        * input.improvement_factor
        * input.reputation_factor_value
        * input.efficiency_factor;

    RewardBreakdown {
        reward,
        useful_work_score: uws.score,
        useful_work_reason: uws.reason,
        verification_confidence: input.verification_confidence_value,
        demand_factor: dfactor,
        quality_factor: input.quality_factor,
        improvement_factor: input.improvement_factor,
        reputation_factor: input.reputation_factor_value,
        efficiency_factor: input.efficiency_factor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn honest_distinct_requester_escrow_backed_signed_job_is_useful() {
        let result = useful_work_score("worker-bob", "requester-alice", true, true, false);
        assert_eq!(result.score, 1.0);
    }

    #[test]
    fn self_dealing_scores_zero_even_with_valid_signature_and_funded_escrow() {
        let result = useful_work_score("same-id", "same-id", true, true, false);
        assert_eq!(result.score, 0.0);
        assert_eq!(
            result.reason,
            "requester and worker are the same identity (self-dealing)"
        );
    }

    #[test]
    fn unfunded_escrow_scores_zero_even_if_otherwise_valid() {
        let result = useful_work_score("worker-bob", "requester-alice", false, true, false);
        assert_eq!(result.score, 0.0);
    }

    #[test]
    fn invalid_signature_scores_zero() {
        let result = useful_work_score("worker-bob", "requester-alice", true, false, false);
        assert_eq!(result.score, 0.0);
    }

    #[test]
    fn network_benchmark_provenance_bypasses_every_other_check() {
        // Even the same identity as both "worker" and "requester", with
        // an unfunded escrow and an invalid signature, still scores 1.0
        // when the job is network-issued benchmark provenance -- matching
        // the Python reference's check ordering (is_network_benchmark is
        // evaluated first, before any of the other gates).
        let result = useful_work_score("same-id", "same-id", false, false, true);
        assert_eq!(result.score, 1.0);
        assert_eq!(result.reason, "network-issued benchmark provenance");
    }

    #[test]
    fn demand_factor_is_the_escrow_to_base_value_ratio_clamped_to_the_cap() {
        assert!((demand_factor(10.0, 10.0) - 1.0).abs() < 1e-12);
        assert!((demand_factor(5.0, 10.0) - 0.5).abs() < 1e-12);
        // Way over base value: clamped at DEMAND_FACTOR_CAP, not allowed
        // to unboundedly amplify reward.
        assert!((demand_factor(1000.0, 10.0) - DEMAND_FACTOR_CAP).abs() < 1e-12);
    }

    #[test]
    fn demand_factor_is_zero_when_base_value_is_non_positive() {
        assert_eq!(demand_factor(10.0, 0.0), 0.0);
        assert_eq!(demand_factor(10.0, -5.0), 0.0);
    }

    #[test]
    fn honest_verified_job_earns_positive_reward() {
        // Mirrors test_poig_basic.py's
        // test_honest_verified_job_earns_positive_reward.
        let input = ComputeRewardInput::new(
            10.0,
            "worker-bob",
            "requester-alice",
            10.0,
            true,
            true,
            1.0,
            1.0,
        );
        let breakdown = compute_reward(&input);
        assert_eq!(breakdown.useful_work_score, 1.0);
        assert!(breakdown.reward > 0.0);
    }

    #[test]
    fn unfunded_job_earns_nothing_even_if_verified() {
        // Mirrors test_poig_basic.py's
        // test_unfunded_job_earns_nothing_even_if_verified.
        let input = ComputeRewardInput::new(
            10.0,
            "worker-bob",
            "requester-alice",
            10.0,
            false, // never actually funded
            true,
            1.0,
            1.0,
        );
        let breakdown = compute_reward(&input);
        assert_eq!(breakdown.useful_work_score, 0.0);
        assert_eq!(breakdown.reward, 0.0);
    }

    #[test]
    fn self_dealing_job_earns_nothing_regardless_of_every_other_factor_being_maxed() {
        // Mirrors test_self_dealing.py's gate-holds assertion: even with
        // every other multiplicative factor at its maximum, self-dealing
        // alone zeroes the reward.
        let mut input =
            ComputeRewardInput::new(100.0, "same-id", "same-id", 200.0, true, true, 1.0, 1.0);
        input.quality_factor = 2.0;
        input.improvement_factor = 2.0;
        input.efficiency_factor = 2.0;
        let breakdown = compute_reward(&input);
        assert_eq!(breakdown.reward, 0.0);
    }

    #[test]
    fn low_reputation_factor_scales_down_reward_proportionally() {
        let low = ComputeRewardInput::new(
            10.0,
            "worker-bob",
            "requester-alice",
            10.0,
            true,
            true,
            1.0,
            crate::reputation::BASELINE_SCORE,
        );
        let high = ComputeRewardInput::new(
            10.0,
            "worker-bob",
            "requester-alice",
            10.0,
            true,
            true,
            1.0,
            1.0,
        );
        let low_reward = compute_reward(&low).reward;
        let high_reward = compute_reward(&high).reward;
        assert!(low_reward < high_reward);
        assert!((low_reward - high_reward * crate::reputation::BASELINE_SCORE).abs() < 1e-9);
    }
}
