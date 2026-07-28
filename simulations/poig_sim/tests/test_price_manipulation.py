"""Attack scenario from docs/security/ECONOMIC-ATTACKS.md: 'Price
manipulation' -- coordinated bidding to distort the routing engine's
priceEfficiency signal.

Assertion: because RoutingScore is a weighted combination of six signals
(not price alone), an actor that maxes out ONLY price_efficiency while
every other signal sits at its floor (a brand-new, unreliable, unverified,
badly-located worker offering to work for free) cannot outscore an
established honest worker with strong reliability/performance/reputation
even at a realistic (non-zero) price. This matches the mitigation already
claimed in docs/security/ECONOMIC-ATTACKS.md and gives it real test
coverage instead of leaving it as an unverified claim.
"""
import pytest

from poig_sim.routing import DEFAULT_WEIGHTS, RoutingSignals, max_score_from_single_signal, routing_score


def test_pure_price_manipulation_cannot_beat_an_established_honest_worker():
    price_manipulator = RoutingSignals(
        reliability=0.0,
        performance=0.0,
        price_efficiency=1.0,  # attacker bids essentially free / manipulates price signal to max
        reputation=0.0,
        locality=0.0,
        verification_strength=0.0,
    )
    established_honest_worker = RoutingSignals(
        reliability=0.9,
        performance=0.8,
        price_efficiency=0.5,  # realistic, non-manipulated price
        reputation=0.85,
        locality=0.6,
        verification_strength=0.7,
    )

    manipulator_score = routing_score(price_manipulator)
    honest_score = routing_score(established_honest_worker)

    assert manipulator_score < honest_score
    # And the manipulator's score is capped exactly at price's weight share --
    # no amount of additional price manipulation can push it higher, since
    # every other signal is already at its floor.
    assert manipulator_score == max_score_from_single_signal("price_efficiency")


def test_single_signal_leverage_is_capped_at_that_signals_weight_for_every_signal():
    """General form of the above: for ANY single signal driven to its max
    while all others sit at floor, the resulting RoutingScore never exceeds
    that signal's own configured weight -- this is what "combines multiple
    weighted signals, reducing single-signal manipulation leverage" actually
    means, made concrete and checked."""
    for signal_name, weight in DEFAULT_WEIGHTS.items():
        kwargs = {k: 0.0 for k in DEFAULT_WEIGHTS}
        kwargs[signal_name] = 1.0
        # RoutingSignals field names use price_efficiency/verification_strength
        # already matching DEFAULT_WEIGHTS keys.
        signals = RoutingSignals(**kwargs)
        score = routing_score(signals)
        assert score == weight, f"{signal_name} leverage exceeded its own weight share"


def test_realistic_price_undercutting_still_bounded_by_weight_share():
    """A more realistic manipulator: not a total free-rider on every other
    signal, but specifically dumping price far below market while keeping
    everything else mediocre. Confirms price's marginal contribution stays
    bounded by its weight even when combined with nonzero (but mediocre)
    other signals."""
    aggressive_undercutter = RoutingSignals(
        reliability=0.3,
        performance=0.3,
        price_efficiency=1.0,  # manipulated to the ceiling
        reputation=0.2,
        locality=0.3,
        verification_strength=0.3,
    )
    baseline_at_moderate_price = RoutingSignals(
        reliability=0.3,
        performance=0.3,
        price_efficiency=0.5,  # unmanipulated moderate price, otherwise identical
        reputation=0.2,
        locality=0.3,
        verification_strength=0.3,
    )

    delta = routing_score(aggressive_undercutter) - routing_score(baseline_at_moderate_price)
    # The entire gain from manipulating price from 0.5 -> 1.0 is bounded by
    # weight * (1.0 - 0.5) -- it cannot spill over into affecting the other
    # signals' contributions.
    assert delta == pytest.approx(DEFAULT_WEIGHTS["price_efficiency"] * 0.5)
