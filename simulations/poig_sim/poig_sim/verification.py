"""Verification tiers (L0-L6) per docs/VERIFICATION.md and docs/adr/0003-verification-tiering.md.

This module models verification *outcomes* abstractly (a confidence value per
tier, plus adaptive verification rate) rather than actually re-executing any
job, since no real inference runtime exists in this simulator (see
docs/research/CAPABILITY-MATRIX.md). It exists to prove the *rate/floor*
logic and the confidence-to-reward wiring, not to prove any tier's real-world
statistical soundness.
"""
from __future__ import annotations

# Base confidence per tier if the check is actually performed and passes.
# L0 is explicitly non-reward-eligible per docs/VERIFICATION.md.
TIER_BASE_CONFIDENCE = {
    "L0": 0.0,
    "L1": 0.3,
    "L2": 0.6,
    "L3": 0.75,
    "L4": 0.85,
    "L5": 0.95,
    "L6": 1.0,
}

# The verification-rate floor: even a maximally-reputable worker is checked
# at least this often. Never zero, per docs/VERIFICATION.md's non-negotiable rule.
VERIFICATION_RATE_FLOOR = 0.05
VERIFICATION_RATE_NEW_IDENTITY = 1.0


def adaptive_verification_rate(reputation_score: float, job_value: float, high_value_threshold: float) -> float:
    """Returns the probability in [FLOOR, 1.0] that this job is actually checked.

    - New/low-reputation workers: rate stays near 1.0 regardless of job value.
    - Established workers: rate falls toward the floor for low-value jobs...
    - ...but any high-value job forces a high rate regardless of reputation
      (L6 quorum territory), per docs/VERIFICATION.md's adaptive quorum rule.
    """
    reputation_score = max(0.0, min(1.0, reputation_score))
    base_rate = VERIFICATION_RATE_NEW_IDENTITY - reputation_score * (
        VERIFICATION_RATE_NEW_IDENTITY - VERIFICATION_RATE_FLOOR
    )
    if job_value >= high_value_threshold:
        base_rate = max(base_rate, 0.8)
    return max(VERIFICATION_RATE_FLOOR, min(1.0, base_rate))


def verification_confidence(tier: str, passed: bool) -> float:
    """VerificationConfidence factor for docs/POIG-SPEC.md's reward formula."""
    if tier not in TIER_BASE_CONFIDENCE:
        raise ValueError(f"unknown verification tier: {tier}")
    if not passed:
        return 0.0
    return TIER_BASE_CONFIDENCE[tier]
