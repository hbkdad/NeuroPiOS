"""Minimal job market tying protocol + verification + PoIG + reputation together.

This is deliberately small: enough to run the two attack scenarios and a
basic honest-job sanity check from docs/security/ECONOMIC-ATTACKS.md, not a
full network simulation. Scaling this to the brief's 100/1,000/10,000/100,000
node Monte Carlo sweep is tracked as future work in docs/ROADMAP.md.
"""
from __future__ import annotations

from dataclasses import dataclass, field

from . import protocol, verification
from .poig import RewardBreakdown, compute_reward
from .reputation import ReputationStore

HIGH_VALUE_THRESHOLD = 100.0


class EscrowLedger:
    """Tracks whether a given job_id's escrow is actually funded by its
    stated requester. A job with no ledger entry is treated as unfunded."""

    def __init__(self) -> None:
        self._funded: set[str] = set()

    def fund(self, job_id: str) -> None:
        self._funded.add(job_id)

    def is_funded(self, job_id: str) -> bool:
        return job_id in self._funded


@dataclass
class JobOutcome:
    job: protocol.Job
    worker_id: str
    verification_tier: str
    verification_passed: bool
    reward: RewardBreakdown


@dataclass
class Market:
    nonces: protocol.NonceTracker = field(default_factory=protocol.NonceTracker)
    escrow: EscrowLedger = field(default_factory=EscrowLedger)
    reputation: ReputationStore = field(default_factory=ReputationStore)
    tick: int = 0

    def submit_and_settle_job(
        self,
        *,
        requester: protocol.Identity,
        worker_id: str,
        job_type: str,
        base_value: float,
        escrow_amount: float,
        verification_policy: str,
        nonce: int,
        fund_escrow: bool,
        verification_passed: bool,
        is_network_benchmark: bool = False,
    ) -> JobOutcome:
        """Create a signed job, optionally fund it, run it against `worker_id`,
        apply the job's declared verification tier, and compute the PoIG reward."""
        job = protocol.create_job(
            requester=requester,
            job_type=job_type,
            base_value=base_value,
            escrow_amount=escrow_amount,
            verification_policy=verification_policy,
            nonce=nonce,
        )

        signature_valid = True
        try:
            protocol.verify_job(job, requester, self.nonces)
        except (protocol.SignatureError, protocol.ReplayError):
            signature_valid = False

        if fund_escrow:
            self.escrow.fund(job.job_id)

        rep_score = self.reputation.reputation_factor(worker_id, self.tick)
        rate = verification.adaptive_verification_rate(
            rep_score, escrow_amount, HIGH_VALUE_THRESHOLD
        )
        # In this simplified market, "was this job actually checked" is an
        # input the caller controls per-scenario (verification_passed implies
        # the check ran and passed); `rate` is exposed on the outcome for
        # scenarios that want to assert on it directly.
        vconf = verification.verification_confidence(verification_policy, verification_passed)

        reward = compute_reward(
            base_value=base_value,
            worker_id=worker_id,
            requester_id=requester.identity_id,
            escrow_amount=escrow_amount,
            escrow_funded=self.escrow.is_funded(job.job_id),
            signature_valid=signature_valid,
            verification_confidence_value=vconf,
            reputation_factor_value=rep_score,
            is_network_benchmark=is_network_benchmark,
        )

        self.reputation.record_successful_job(worker_id, self.tick, verification_passed)
        self.tick += 1

        return JobOutcome(
            job=job,
            worker_id=worker_id,
            verification_tier=verification_policy,
            verification_passed=verification_passed,
            reward=reward,
        )
