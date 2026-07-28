//! Verification tiers (L0-L6) per docs/VERIFICATION.md and
//! docs/adr/0003-verification-tiering.md. This is the Rust port of
//! simulations/poig_sim/poig_sim/verification.py's tier-confidence and
//! adaptive-rate logic -- that Python module is the validated reference
//! this crate must not silently diverge from (see
//! crates/aion-reputation's README convention once that crate exists).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tier {
    L0,
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
}

/// The verification-rate floor: even a maximally-reputable worker is
/// checked at least this often. Never zero, per docs/VERIFICATION.md's
/// non-negotiable rule.
pub const VERIFICATION_RATE_FLOOR: f64 = 0.05;
pub const VERIFICATION_RATE_NEW_IDENTITY: f64 = 1.0;

impl Tier {
    /// Base confidence per tier if the check is actually performed and
    /// passes. L0 is explicitly non-reward-eligible per docs/VERIFICATION.md.
    pub fn base_confidence(self) -> f64 {
        match self {
            Tier::L0 => 0.0,
            Tier::L1 => 0.3,
            Tier::L2 => 0.6,
            Tier::L3 => 0.75,
            Tier::L4 => 0.85,
            Tier::L5 => 0.95,
            Tier::L6 => 1.0,
        }
    }
}

/// `VerificationConfidence` factor for docs/POIG-SPEC.md's reward formula.
pub fn verification_confidence(tier: Tier, passed: bool) -> f64 {
    if !passed {
        return 0.0;
    }
    tier.base_confidence()
}

/// Returns the probability in `[FLOOR, 1.0]` that a job is actually
/// checked.
///
/// - New/low-reputation workers: rate stays near 1.0 regardless of job value.
/// - Established workers: rate falls toward the floor for low-value jobs...
/// - ...but any high-value job forces a high rate regardless of reputation
///   (L6 quorum territory), per docs/VERIFICATION.md's adaptive quorum rule.
pub fn adaptive_verification_rate(
    reputation_score: f64,
    job_value: f64,
    high_value_threshold: f64,
) -> f64 {
    let reputation_score = reputation_score.clamp(0.0, 1.0);
    let mut base_rate = VERIFICATION_RATE_NEW_IDENTITY
        - reputation_score * (VERIFICATION_RATE_NEW_IDENTITY - VERIFICATION_RATE_FLOOR);
    if job_value >= high_value_threshold {
        base_rate = base_rate.max(0.8);
    }
    base_rate.clamp(VERIFICATION_RATE_FLOOR, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_confidences_match_the_validated_python_reference() {
        // Pinned against simulations/poig_sim/poig_sim/verification.py's
        // TIER_BASE_CONFIDENCE table -- must not silently drift.
        assert_eq!(Tier::L0.base_confidence(), 0.0);
        assert_eq!(Tier::L1.base_confidence(), 0.3);
        assert_eq!(Tier::L2.base_confidence(), 0.6);
        assert_eq!(Tier::L3.base_confidence(), 0.75);
        assert_eq!(Tier::L4.base_confidence(), 0.85);
        assert_eq!(Tier::L5.base_confidence(), 0.95);
        assert_eq!(Tier::L6.base_confidence(), 1.0);
    }

    #[test]
    fn l0_is_never_reward_eligible_even_when_passed() {
        assert_eq!(verification_confidence(Tier::L0, true), 0.0);
    }

    #[test]
    fn failed_verification_is_always_zero_confidence_regardless_of_tier() {
        for tier in [Tier::L1, Tier::L2, Tier::L3, Tier::L4, Tier::L5, Tier::L6] {
            assert_eq!(verification_confidence(tier, false), 0.0);
        }
    }

    #[test]
    fn new_identity_faces_maximum_verification_rate() {
        let rate = adaptive_verification_rate(0.0, 1.0, 100.0);
        assert_eq!(rate, VERIFICATION_RATE_NEW_IDENTITY);
    }

    #[test]
    fn established_identity_low_value_job_approaches_the_floor() {
        let rate = adaptive_verification_rate(1.0, 1.0, 100.0);
        assert!((rate - VERIFICATION_RATE_FLOOR).abs() < 1e-9);
    }

    #[test]
    fn verification_rate_never_goes_below_the_floor_even_for_a_perfect_reputation() {
        let rate = adaptive_verification_rate(1.0, 0.0, 100.0);
        assert!(rate >= VERIFICATION_RATE_FLOOR);
    }

    #[test]
    fn high_value_job_forces_a_high_rate_regardless_of_reputation() {
        // Even a maximally-reputable worker gets a high check rate on a
        // high-value job -- the whole point of the L6 escalation rule.
        let rate = adaptive_verification_rate(1.0, 1000.0, 100.0);
        assert!(rate >= 0.8);
    }

    #[test]
    fn reputation_score_is_clamped_before_use() {
        // Out-of-range inputs (defensive) don't produce an out-of-range rate.
        let rate_low = adaptive_verification_rate(-5.0, 1.0, 100.0);
        let rate_high = adaptive_verification_rate(5.0, 1.0, 100.0);
        assert!((0.0..=1.0).contains(&rate_low));
        assert!((0.0..=1.0).contains(&rate_high));
    }
}
