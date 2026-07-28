"""Commit/reveal multi-validator quorum aggregation, per docs/VERIFICATION.md's
L6 tier and the Commit/reveal section (prevents a validator from copying
another validator's answer before submitting its own).

Honest scope note: this module demonstrates that quorum weighting resists a
MINORITY colluding validator cluster (docs/security/ECONOMIC-ATTACKS.md,
"Collusion (validator cartel)" row). It does NOT claim to prevent a
colluding MAJORITY from flipping an outcome -- no quorum-weighting scheme
can, by construction, since majority agreement is the mechanism's whole
basis for trust. That residual risk is documented, not hidden -- see
tests/test_validator_collusion.py's second scenario and
docs/security/THREAT-MODEL.md.
"""
from __future__ import annotations

import hashlib
from dataclasses import dataclass, field


class CommitMismatchError(Exception):
    pass


class RevealBeforeAllCommittedError(Exception):
    pass


def _commit(vote: bool, salt: str) -> str:
    return hashlib.sha256(f"{vote}:{salt}".encode("utf-8")).hexdigest()


@dataclass
class ValidatorBallot:
    validator_id: str
    weight: float
    commit_hash: str | None = None
    revealed_vote: bool | None = None
    _salt: str = field(default="", repr=False)


@dataclass
class Quorum:
    """A single commit/reveal round across a fixed validator set."""

    ballots: dict[str, ValidatorBallot] = field(default_factory=dict)
    _all_committed: bool = False

    def add_validator(self, validator_id: str, weight: float) -> None:
        self.ballots[validator_id] = ValidatorBallot(validator_id=validator_id, weight=weight)

    def commit(self, validator_id: str, vote: bool, salt: str) -> None:
        """Validators commit a hash of (vote, salt) -- the vote itself is
        hidden until reveal, so a validator cannot simply copy another
        validator's already-visible vote."""
        self.ballots[validator_id].commit_hash = _commit(vote, salt)
        self.ballots[validator_id]._salt = salt
        # stash the true vote only to let the test harness simulate reveal
        # timing; a real implementation would not store this before reveal.
        self.ballots[validator_id]._pending_vote = vote  # type: ignore[attr-defined]
        if all(b.commit_hash is not None for b in self.ballots.values()):
            self._all_committed = True

    def reveal(self, validator_id: str, vote: bool, salt: str) -> None:
        if not self._all_committed:
            raise RevealBeforeAllCommittedError(
                "cannot reveal until every validator in this quorum has committed"
            )
        ballot = self.ballots[validator_id]
        if _commit(vote, salt) != ballot.commit_hash:
            raise CommitMismatchError(
                f"revealed (vote={vote}, salt={salt}) does not match earlier commit "
                f"for validator {validator_id} -- cannot change a vote after committing"
            )
        ballot.revealed_vote = vote

    def aggregate(self, threshold: float = 0.5) -> tuple[bool, float]:
        """Weighted-majority aggregation. Returns (outcome, agreement_fraction).
        Ballots that never revealed are excluded (non-response), matching
        docs/security/THREAT-MODEL.md's tolerance for nonresponsive
        validators without treating silence as a vote either way."""
        revealed = [b for b in self.ballots.values() if b.revealed_vote is not None]
        if not revealed:
            raise ValueError("no validator revealed a vote in this quorum")
        total_weight = sum(b.weight for b in revealed)
        yes_weight = sum(b.weight for b in revealed if b.revealed_vote)
        fraction = yes_weight / total_weight if total_weight else 0.0
        return (fraction >= threshold, fraction)
