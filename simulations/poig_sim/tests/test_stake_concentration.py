"""Attack scenario from docs/security/ECONOMIC-ATTACKS.md: 'Stake
concentration' -- a small number of actors accumulate enough stake to
dominate quorums/governance.

This reuses poig_sim/quorum.py's weighted aggregation (the same mechanism
already exercised by test_validator_collusion.py's minority/majority
scenarios) but isolates the variable that actually matters here: weight
concentration, independent of headcount. A minority BY COUNT that also
happens to be a majority BY WEIGHT still flips the outcome; the same
headcount minority with EQUAL weights does not. This is the concrete
demonstration behind docs/POIG-SPEC.md's ReputationFactor design note
("reputation... is never purchasable or transferable") and
docs/security/THREAT-MODEL.md's residual-risk framing for quorum-based
verification: weight, not headcount, is the actual security parameter.
"""
import pytest

from poig_sim.quorum import Quorum


def _run_quorum(weights: dict[str, float], dishonest_ids: set[str]) -> tuple[bool, float]:
    """Ground truth: the result should be `False` (fail). Honest validators
    vote correctly (False); dishonest ones vote incorrectly (True)."""
    q = Quorum()
    for vid, w in weights.items():
        q.add_validator(vid, weight=w)

    for vid in weights:
        vote = vid in dishonest_ids
        q.commit(vid, vote=vote, salt=f"salt-{vid}")
    for vid in weights:
        vote = vid in dishonest_ids
        q.reveal(vid, vote=vote, salt=f"salt-{vid}")

    return q.aggregate(threshold=0.5)


def test_stake_concentrated_minority_dominates_despite_being_a_headcount_minority():
    # 8 honest validators, equal weight 1 each (total honest weight = 8).
    honest = {f"honest-{i}": 1.0 for i in range(8)}
    # Only 2 dishonest validators BY COUNT (a clear headcount minority: 2 of 10)
    # but each holds a concentrated stake of 20 (total dishonest weight = 40).
    whales = {f"whale-{i}": 20.0 for i in range(2)}

    weights = {**honest, **whales}
    outcome_passed, agreement = _run_quorum(weights, dishonest_ids=set(whales))

    # Ground truth was "should fail" (False); the stake-concentrated
    # minority flips it to "pass" (True) purely via weight, despite being
    # outnumbered 8-to-2 by headcount.
    assert outcome_passed is True
    assert agreement == pytest.approx(40 / 48)


def test_the_same_headcount_minority_fails_to_dominate_with_equal_weights():
    # Identical 8-honest-vs-2-dishonest headcount split, but this time
    # NO stake concentration -- every validator (honest or not) has equal
    # weight 1. This isolates weight concentration as the actual variable:
    # same headcount minority, same votes, different outcome.
    honest = {f"honest-{i}": 1.0 for i in range(8)}
    equal_weight_minority = {f"whale-{i}": 1.0 for i in range(2)}

    weights = {**honest, **equal_weight_minority}
    outcome_passed, agreement = _run_quorum(weights, dishonest_ids=set(equal_weight_minority))

    assert outcome_passed is False
    assert agreement == pytest.approx(2 / 10)


def test_dominance_threshold_is_continuous_in_stake_share_not_headcount():
    """Sweeps the whale's per-validator weight and confirms the outcome
    flips exactly when the whales' aggregate weight share crosses 50%,
    regardless of the (constant) 8-vs-2 headcount split -- i.e. the
    quorum's safety property really is governed by weight distribution,
    not by how many distinct identities happen to hold that weight."""
    honest = {f"honest-{i}": 1.0 for i in range(8)}  # total honest weight = 8

    # Just under the tipping point: whales' combined weight (2 * 3.9 = 7.8) < 8
    weights_below = {**honest, **{f"whale-{i}": 3.9 for i in range(2)}}
    outcome_below, _ = _run_quorum(weights_below, dishonest_ids={"whale-0", "whale-1"})
    assert outcome_below is False

    # Just over the tipping point: whales' combined weight (2 * 4.1 = 8.2) > 8
    weights_above = {**honest, **{f"whale-{i}": 4.1 for i in range(2)}}
    outcome_above, _ = _run_quorum(weights_above, dishonest_ids={"whale-0", "whale-1"})
    assert outcome_above is True
