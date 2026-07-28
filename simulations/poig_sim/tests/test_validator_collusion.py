"""Attack scenario from docs/security/ECONOMIC-ATTACKS.md: 'Collusion
(validator cartel)'. Two scenarios, both run for real and reported honestly:

1. MINORITY collusion: a colluding cluster below the aggregation threshold
   tries to flip a quorum outcome in favor of a malicious worker's bad
   result. Expected: blocked -- the honest majority's weight still carries
   the aggregate.
2. MAJORITY collusion: the colluding cluster IS the majority. Expected:
   the outcome flips -- this is a genuine, honestly-documented residual
   risk (see quorum.py's module docstring and docs/security/THREAT-MODEL.md),
   not something this design claims to prevent. No quorum-weighting scheme
   can resist a colluding majority by construction.

Also covers: commit/reveal actually prevents a validator from copying
another validator's vote after seeing it (a validator cannot reveal a vote
that doesn't match its earlier commit).
"""
import pytest

from poig_sim.quorum import CommitMismatchError, Quorum, RevealBeforeAllCommittedError


def _build_quorum(honest_ids: list[str], colluding_ids: list[str]) -> Quorum:
    q = Quorum()
    for vid in honest_ids + colluding_ids:
        q.add_validator(vid, weight=1.0)
    return q


def test_minority_collusion_does_not_flip_the_outcome():
    honest = [f"honest-{i}" for i in range(7)]
    colluding = [f"colluder-{i}" for i in range(3)]  # minority: 3 of 10
    q = _build_quorum(honest, colluding)

    # ground truth: the worker's result was actually BAD (should fail).
    for vid in honest:
        q.commit(vid, vote=False, salt=f"salt-{vid}")  # honest: votes fail (correct)
    for vid in colluding:
        q.commit(vid, vote=True, salt=f"salt-{vid}")  # colluders: vote pass anyway

    for vid in honest:
        q.reveal(vid, vote=False, salt=f"salt-{vid}")
    for vid in colluding:
        q.reveal(vid, vote=True, salt=f"salt-{vid}")

    outcome_passed, agreement = q.aggregate(threshold=0.5)

    assert outcome_passed is False  # correct outcome held: minority collusion blocked
    assert agreement == pytest.approx(0.3)


def test_majority_collusion_does_flip_the_outcome_documented_residual_risk():
    honest = [f"honest-{i}" for i in range(4)]
    colluding = [f"colluder-{i}" for i in range(6)]  # majority: 6 of 10
    q = _build_quorum(honest, colluding)

    for vid in honest:
        q.commit(vid, vote=False, salt=f"salt-{vid}")
    for vid in colluding:
        q.commit(vid, vote=True, salt=f"salt-{vid}")
    for vid in honest:
        q.reveal(vid, vote=False, salt=f"salt-{vid}")
    for vid in colluding:
        q.reveal(vid, vote=True, salt=f"salt-{vid}")

    outcome_passed, agreement = q.aggregate(threshold=0.5)

    # Documenting the known limitation, not hiding it: a colluding majority
    # DOES flip the aggregate outcome in this design. Mitigations for this
    # live outside quorum weighting itself (bonding cost, reputation
    # consequence for a validator later proven wrong via a higher tier or a
    # fraud proof under ADR-0007, cross-checking against L2/L4 evidence at
    # higher value -- see docs/adr/0003-verification-tiering.md's L6 note
    # that it layers on L2/L4 rather than standing alone).
    assert outcome_passed is True
    assert agreement == pytest.approx(0.6)


def test_commit_reveal_prevents_copying_another_validators_vote():
    q = Quorum()
    q.add_validator("v1", weight=1.0)
    q.add_validator("v2", weight=1.0)

    q.commit("v1", vote=True, salt="s1")
    q.commit("v2", vote=True, salt="s2")

    # v2 tries to reveal a DIFFERENT vote than it committed to (e.g. having
    # seen v1's vote leak some other way and deciding to copy a vote it
    # didn't actually commit to) -- must be rejected.
    with pytest.raises(CommitMismatchError):
        q.reveal("v2", vote=False, salt="s2")


def test_reveal_before_all_committed_is_rejected():
    q = Quorum()
    q.add_validator("v1", weight=1.0)
    q.add_validator("v2", weight=1.0)

    q.commit("v1", vote=True, salt="s1")
    # v2 has not committed yet.
    with pytest.raises(RevealBeforeAllCommittedError):
        q.reveal("v1", vote=True, salt="s1")
