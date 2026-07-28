//! Deterministic Routing Score, per docs/ARCHITECTURE.md. Ported from and
//! pinned against the validated Python reference at
//! `simulations/poig_sim/poig_sim/routing.py`.
//!
//! ```text
//! RoutingScore = w1*reliability + w2*performance + w3*priceEfficiency
//!              + w4*reputation + w5*locality + w6*verificationStrength
//! ```
//!
//! The default weights below are simulation starting points (not tuned
//! against real data -- same caveat as the Python reference and
//! docs/ARCHITECTURE.md), but their SHAPE is what this module's tests
//! actually validate: that no single signal (price in particular, per
//! docs/security/ECONOMIC-ATTACKS.md's "Price manipulation" row) can
//! dominate the score beyond its own weight share, regardless of how
//! extreme that one signal is driven.
//!
//! One behavior-preserving improvement over the Python reference: instead
//! of a stringly-typed `dict` key for [`max_score_from_single_signal`],
//! this port uses a [`RoutingSignal`] enum -- an unknown/misspelled
//! signal name is a compile error here instead of a Python `KeyError` at
//! call time.

/// Per-signal weights for [`routing_score`]. Must sum to 1.0 -- verified
/// by a test on [`RoutingWeights::default_weights`] below, mirroring the
/// Python reference's module-level `assert` on `DEFAULT_WEIGHTS` (a test
/// is the idiomatic Rust equivalent for a compile-time-known constant,
/// rather than a runtime assertion on every construction).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoutingWeights {
    pub reliability: f64,
    pub performance: f64,
    pub price_efficiency: f64,
    pub reputation: f64,
    pub locality: f64,
    pub verification_strength: f64,
}

impl RoutingWeights {
    pub fn default_weights() -> Self {
        Self {
            reliability: 0.25,
            performance: 0.20,
            price_efficiency: 0.20,
            reputation: 0.15,
            locality: 0.10,
            verification_strength: 0.10,
        }
    }

    fn weight_of(&self, signal: RoutingSignal) -> f64 {
        match signal {
            RoutingSignal::Reliability => self.reliability,
            RoutingSignal::Performance => self.performance,
            RoutingSignal::PriceEfficiency => self.price_efficiency,
            RoutingSignal::Reputation => self.reputation,
            RoutingSignal::Locality => self.locality,
            RoutingSignal::VerificationStrength => self.verification_strength,
        }
    }
}

/// Identifies a single routing signal, for [`max_score_from_single_signal`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingSignal {
    Reliability,
    Performance,
    PriceEfficiency,
    Reputation,
    Locality,
    VerificationStrength,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoutingSignals {
    pub reliability: f64,
    pub performance: f64,
    pub price_efficiency: f64,
    pub reputation: f64,
    pub locality: f64,
    pub verification_strength: f64,
}

impl RoutingSignals {
    pub fn clamped(&self) -> Self {
        let c = |x: f64| x.clamp(0.0, 1.0);
        Self {
            reliability: c(self.reliability),
            performance: c(self.performance),
            price_efficiency: c(self.price_efficiency),
            reputation: c(self.reputation),
            locality: c(self.locality),
            verification_strength: c(self.verification_strength),
        }
    }
}

pub fn routing_score(signals: &RoutingSignals, weights: &RoutingWeights) -> f64 {
    let s = signals.clamped();
    weights.reliability * s.reliability
        + weights.performance * s.performance
        + weights.price_efficiency * s.price_efficiency
        + weights.reputation * s.reputation
        + weights.locality * s.locality
        + weights.verification_strength * s.verification_strength
}

/// The theoretical maximum `RoutingScore` achievable by maxing ONLY the
/// named signal while every other signal sits at its floor (0.0). Used to
/// assert that price-manipulation leverage is capped at that signal's own
/// weight share, per the mitigation claimed in
/// docs/security/ECONOMIC-ATTACKS.md.
pub fn max_score_from_single_signal(signal: RoutingSignal, weights: &RoutingWeights) -> f64 {
    weights.weight_of(signal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_weights_sum_to_one() {
        let w = RoutingWeights::default_weights();
        let sum = w.reliability
            + w.performance
            + w.price_efficiency
            + w.reputation
            + w.locality
            + w.verification_strength;
        assert!((sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn all_signals_at_floor_scores_zero() {
        let signals = RoutingSignals {
            reliability: 0.0,
            performance: 0.0,
            price_efficiency: 0.0,
            reputation: 0.0,
            locality: 0.0,
            verification_strength: 0.0,
        };
        let score = routing_score(&signals, &RoutingWeights::default_weights());
        assert_eq!(score, 0.0);
    }

    #[test]
    fn all_signals_at_ceiling_scores_one_since_weights_sum_to_one() {
        let signals = RoutingSignals {
            reliability: 1.0,
            performance: 1.0,
            price_efficiency: 1.0,
            reputation: 1.0,
            locality: 1.0,
            verification_strength: 1.0,
        };
        let score = routing_score(&signals, &RoutingWeights::default_weights());
        assert!((score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn signals_outside_the_unit_range_are_clamped_before_scoring() {
        let signals = RoutingSignals {
            reliability: 5.0,  // way over ceiling
            performance: -3.0, // way under floor
            price_efficiency: 0.5,
            reputation: 0.5,
            locality: 0.5,
            verification_strength: 0.5,
        };
        let clamped = signals.clamped();
        assert_eq!(clamped.reliability, 1.0);
        assert_eq!(clamped.performance, 0.0);
    }

    #[test]
    fn price_manipulation_leverage_is_capped_at_prices_own_weight_share() {
        // Mirrors simulations/poig_sim/tests/test_price_manipulation.py's
        // real security-relevant assertion: a worker that manipulates
        // ONLY its price signal -- maxed out, with every other signal at
        // the floor -- cannot outscore a routing weight scheme beyond
        // that one signal's own configured weight share (0.20 by
        // default), not the full 1.0.
        let weights = RoutingWeights::default_weights();
        let manipulated_only_price = RoutingSignals {
            reliability: 0.0,
            performance: 0.0,
            price_efficiency: 1.0, // maxed
            reputation: 0.0,
            locality: 0.0,
            verification_strength: 0.0,
        };
        let score = routing_score(&manipulated_only_price, &weights);
        assert!((score - weights.price_efficiency).abs() < 1e-9);
        assert_eq!(
            max_score_from_single_signal(RoutingSignal::PriceEfficiency, &weights),
            weights.price_efficiency
        );
    }

    #[test]
    fn max_score_from_single_signal_matches_each_signals_own_weight() {
        let weights = RoutingWeights::default_weights();
        assert_eq!(
            max_score_from_single_signal(RoutingSignal::Reliability, &weights),
            weights.reliability
        );
        assert_eq!(
            max_score_from_single_signal(RoutingSignal::Performance, &weights),
            weights.performance
        );
        assert_eq!(
            max_score_from_single_signal(RoutingSignal::Reputation, &weights),
            weights.reputation
        );
        assert_eq!(
            max_score_from_single_signal(RoutingSignal::Locality, &weights),
            weights.locality
        );
        assert_eq!(
            max_score_from_single_signal(RoutingSignal::VerificationStrength, &weights),
            weights.verification_strength
        );
    }

    #[test]
    fn a_mixed_realistic_signal_profile_scores_the_expected_weighted_sum() {
        let weights = RoutingWeights::default_weights();
        let signals = RoutingSignals {
            reliability: 0.9,
            performance: 0.8,
            price_efficiency: 0.6,
            reputation: 0.7,
            locality: 0.5,
            verification_strength: 1.0,
        };
        let expected = 0.25 * 0.9 + 0.20 * 0.8 + 0.20 * 0.6 + 0.15 * 0.7 + 0.10 * 0.5 + 0.10 * 1.0;
        let score = routing_score(&signals, &weights);
        assert!((score - expected).abs() < 1e-9);
    }
}
