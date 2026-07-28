"""PoIG reward calculation per docs/POIG-SPEC.md.

Reward = BaseValue * UsefulWorkScore * VerificationConfidence * DemandFactor
         * QualityFactor * ImprovementFactor * ReputationFactor * EfficiencyFactor

This is the load-bearing anti-fraud logic: UsefulWorkScore is the gate that
enforces the orchestration brief's core security rule ("never equate
computation occurred with useful AI computation occurred"). It is only
nonzero when the job traces back to real, signed, escrow-backed external
demand from a requester distinct from the worker, or to network benchmark
provenance.
"""
from __future__ import annotations

from dataclasses import dataclass

# DemandFactor is capped to prevent a thin/manipulable demand signal (e.g. a
# single large self-funded escrow) from unboundedly amplifying reward, per
# docs/security/ECONOMIC-ATTACKS.md's "fake demand / wash activity" row.
DEMAND_FACTOR_CAP = 2.0


@dataclass
class UsefulWorkResult:
    score: float
    reason: str


def useful_work_score(
    *,
    worker_id: str,
    requester_id: str,
    escrow_funded: bool,
    signature_valid: bool,
    is_network_benchmark: bool = False,
) -> UsefulWorkResult:
    """The anti-farming gate. Returns 0.0 unless the work traces to real,
    independent, signed, escrow-backed demand (or network-benchmark
    provenance)."""
    if is_network_benchmark:
        return UsefulWorkResult(1.0, "network-issued benchmark provenance")
    if not signature_valid:
        return UsefulWorkResult(0.0, "requester signature invalid")
    if not escrow_funded:
        return UsefulWorkResult(0.0, "no funded escrow backing this job")
    if worker_id == requester_id:
        # Self-dealing: the same identity is both requester and worker.
        # This alone is disqualifying regardless of escrow, because the
        # "external demand" this factor is supposed to certify does not
        # exist when the requester and the worker are the same actor.
        return UsefulWorkResult(0.0, "requester and worker are the same identity (self-dealing)")
    return UsefulWorkResult(1.0, "signed, escrow-backed job from a distinct requester")


def demand_factor(escrow_amount: float, base_value: float) -> float:
    if base_value <= 0:
        return 0.0
    raw = escrow_amount / base_value
    return max(0.0, min(DEMAND_FACTOR_CAP, raw))


@dataclass
class RewardBreakdown:
    reward: float
    useful_work_score: float
    useful_work_reason: str
    verification_confidence: float
    demand_factor: float
    quality_factor: float
    improvement_factor: float
    reputation_factor: float
    efficiency_factor: float


def compute_reward(
    *,
    base_value: float,
    worker_id: str,
    requester_id: str,
    escrow_amount: float,
    escrow_funded: bool,
    signature_valid: bool,
    verification_confidence_value: float,
    reputation_factor_value: float,
    quality_factor: float = 1.0,
    improvement_factor: float = 1.0,
    efficiency_factor: float = 1.0,
    is_network_benchmark: bool = False,
) -> RewardBreakdown:
    uws = useful_work_score(
        worker_id=worker_id,
        requester_id=requester_id,
        escrow_funded=escrow_funded,
        signature_valid=signature_valid,
        is_network_benchmark=is_network_benchmark,
    )
    dfactor = demand_factor(escrow_amount, base_value)

    reward = (
        base_value
        * uws.score
        * verification_confidence_value
        * dfactor
        * quality_factor
        * improvement_factor
        * reputation_factor_value
        * efficiency_factor
    )
    return RewardBreakdown(
        reward=reward,
        useful_work_score=uws.score,
        useful_work_reason=uws.reason,
        verification_confidence=verification_confidence_value,
        demand_factor=dfactor,
        quality_factor=quality_factor,
        improvement_factor=improvement_factor,
        reputation_factor=reputation_factor_value,
        efficiency_factor=efficiency_factor,
    )
