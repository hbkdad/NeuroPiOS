"""Deterministic Routing Score per docs/ARCHITECTURE.md:

RoutingScore = w1*reliability + w2*performance + w3*priceEfficiency
             + w4*reputation + w5*locality + w6*verificationStrength

Default weights below are simulation starting points (not tuned against real
data -- see docs/ARCHITECTURE.md's explicit caveat) but their SHAPE is what
this module's test coverage actually validates: that no single signal (price
in particular, per docs/security/ECONOMIC-ATTACKS.md's "Price manipulation"
row) can dominate the score beyond its own weight share, regardless of how
extreme that one signal is driven.
"""
from __future__ import annotations

from dataclasses import dataclass

DEFAULT_WEIGHTS = {
    "reliability": 0.25,
    "performance": 0.20,
    "price_efficiency": 0.20,
    "reputation": 0.15,
    "locality": 0.10,
    "verification_strength": 0.10,
}

assert abs(sum(DEFAULT_WEIGHTS.values()) - 1.0) < 1e-9, "routing weights must sum to 1.0"


@dataclass
class RoutingSignals:
    reliability: float
    performance: float
    price_efficiency: float
    reputation: float
    locality: float
    verification_strength: float

    def clamped(self) -> "RoutingSignals":
        def c(x: float) -> float:
            return max(0.0, min(1.0, x))

        return RoutingSignals(
            reliability=c(self.reliability),
            performance=c(self.performance),
            price_efficiency=c(self.price_efficiency),
            reputation=c(self.reputation),
            locality=c(self.locality),
            verification_strength=c(self.verification_strength),
        )


def routing_score(signals: RoutingSignals, weights: dict[str, float] = DEFAULT_WEIGHTS) -> float:
    s = signals.clamped()
    return (
        weights["reliability"] * s.reliability
        + weights["performance"] * s.performance
        + weights["price_efficiency"] * s.price_efficiency
        + weights["reputation"] * s.reputation
        + weights["locality"] * s.locality
        + weights["verification_strength"] * s.verification_strength
    )


def max_score_from_single_signal(signal_name: str, weights: dict[str, float] = DEFAULT_WEIGHTS) -> float:
    """The theoretical maximum RoutingScore achievable by maxing ONLY the
    named signal while every other signal sits at its floor (0.0). Used to
    assert that price-manipulation leverage is capped at that signal's own
    weight share, per the mitigation claimed in docs/security/ECONOMIC-ATTACKS.md."""
    return weights[signal_name]
