"""Multi-dimensional, decaying, non-transferable reputation store.

Per docs/security/THREAT-MODEL.md and docs/ARCHITECTURE.md: reputation is
never a single global scalar in the underlying store (tracked per-dimension
here), is never purchasable/transferable (no API exists in this module to
move a dimension between identities), and decays over time when an identity
is inactive.
"""
from __future__ import annotations

from dataclasses import dataclass, field

# New identities start here: nonzero (so a first job is possible at all) but
# well below an established honest worker's steady-state score, per the
# "new/unproven worker gets max verification rate" design in docs/VERIFICATION.md.
BASELINE_SCORE = 0.15
DECAY_PER_TICK_IDLE = 0.01
MAX_SCORE = 1.0


@dataclass
class ReputationRecord:
    completion_reliability: float = BASELINE_SCORE
    verification_reliability: float = BASELINE_SCORE
    jobs_completed: int = 0
    disputes_lost: int = 0
    last_active_tick: int = 0

    def composite(self) -> float:
        """Bounded [0,1] collapse used as PoIG's ReputationFactor.
        The underlying dimensions remain separately tracked/queryable above."""
        base = 0.5 * self.completion_reliability + 0.5 * self.verification_reliability
        penalty = min(0.5, 0.1 * self.disputes_lost)
        return max(0.0, min(MAX_SCORE, base - penalty))


class ReputationStore:
    def __init__(self) -> None:
        self._records: dict[str, ReputationRecord] = {}

    def get(self, identity_id: str) -> ReputationRecord:
        return self._records.setdefault(identity_id, ReputationRecord())

    def record_successful_job(self, identity_id: str, tick: int, verified: bool) -> None:
        rec = self.get(identity_id)
        rec.jobs_completed += 1
        rec.last_active_tick = tick
        # Reliability grows toward 1.0 with diminishing returns (asymptotic),
        # never in one jump, so a single job can never establish full trust.
        rec.completion_reliability += (MAX_SCORE - rec.completion_reliability) * 0.08
        if verified:
            rec.verification_reliability += (MAX_SCORE - rec.verification_reliability) * 0.08

    def record_dispute_loss(self, identity_id: str, tick: int) -> None:
        rec = self.get(identity_id)
        rec.disputes_lost += 1
        rec.last_active_tick = tick

    def decay(self, identity_id: str, current_tick: int) -> None:
        rec = self.get(identity_id)
        idle_ticks = max(0, current_tick - rec.last_active_tick)
        if idle_ticks <= 0:
            return
        decay = min(rec.completion_reliability - BASELINE_SCORE, DECAY_PER_TICK_IDLE * idle_ticks)
        decay = max(decay, 0.0)
        rec.completion_reliability = max(BASELINE_SCORE, rec.completion_reliability - decay)
        rec.verification_reliability = max(
            BASELINE_SCORE, rec.verification_reliability - decay
        )

    def reputation_factor(self, identity_id: str, current_tick: int) -> float:
        self.decay(identity_id, current_tick)
        return self.get(identity_id).composite()
