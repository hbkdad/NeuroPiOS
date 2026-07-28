"""Attack scenario from docs/security/ECONOMIC-ATTACKS.md: 'Griefing' --
deliberately wasting counterparties' resources without a clear profit
motive (e.g., always disputing to force bisection cost).

Assertion under test: the mitigation column's claim that "L4's bisection
cost falls disproportionately on the losing party in a dispute,
disincentivizing frivolous challenges" -- i.e. a challenger who disputes
every job from a target worker with NO informational edge (pure griefing,
not a legitimate challenge) bleeds their own capital every single round,
regardless of how the forfeited bond is split between the worker and
protocol treasury. Contrasted against a challenger who is always right
(worker cheats every time), which is profitable -- proving the mechanism
distinguishes griefing from legitimate challenging rather than just
punishing disputes indiscriminately.
"""
import pytest

from poig_sim.dispute_economics import DisputeEconomicsConfig, simulate_dispute_campaign


def default_config(worker_compensation_fraction: float = 0.5) -> DisputeEconomicsConfig:
    return DisputeEconomicsConfig(
        challenger_bond=10.0,
        worker_compensation_fraction=worker_compensation_fraction,
        fraud_proven_worker_slash=20.0,
        fraud_proven_challenger_reward=15.0,
    )


def test_pure_griefing_against_a_perfectly_honest_worker_bleeds_the_challengers_capital():
    result = simulate_dispute_campaign(
        default_config(), rounds=50, worker_actually_cheats_fraction=0.0
    )
    # Every dispute is forfeited (fraud never proven) -- the challenger's
    # entire bond is lost every round, full stop. This holds regardless of
    # the compensation split (see the next test) -- the challenger never
    # gets any of a forfeited bond back under any split, only the worker
    # and treasury do.
    assert result.challenger_net_pnl == pytest.approx(-50 * 10.0)
    assert result.challenger_net_pnl < 0


def test_griefing_cost_is_lost_regardless_of_how_the_forfeited_bond_is_split():
    """The worker-compensation-fraction tunable changes who RECEIVES the
    forfeited bond, not whether the griefing challenger loses it -- the
    mitigation's "cost falls on the losing party" claim holds at every
    split, not just a favorably-chosen one."""
    for compensation_fraction in (0.0, 0.25, 0.5, 0.75, 1.0):
        result = simulate_dispute_campaign(
            default_config(compensation_fraction), rounds=20, worker_actually_cheats_fraction=0.0
        )
        assert result.challenger_net_pnl == pytest.approx(-20 * 10.0)


def test_zero_worker_compensation_leaves_the_griefed_worker_with_no_direct_payout():
    """Honest documentation of what this mechanism does NOT solve: if
    worker_compensation_fraction is configured to 0, the griefed worker
    receives no direct capital compensation at all -- the entire forfeited
    bond goes to treasury instead. The challenger still loses money (the
    attack is not free), but the worker is not made whole either. This is
    exactly the "exact cost allocation is a simulation-tunable parameter"
    caveat in docs/security/ECONOMIC-ATTACKS.md's mitigation column, made
    concrete rather than left as an assertion."""
    result = simulate_dispute_campaign(
        default_config(worker_compensation_fraction=0.0),
        rounds=10,
        worker_actually_cheats_fraction=0.0,
    )
    assert result.worker_net_pnl == 0.0
    assert result.treasury_net_intake == pytest.approx(10 * 10.0)


def test_worker_compensation_is_proportional_to_the_configured_split():
    result = simulate_dispute_campaign(
        default_config(worker_compensation_fraction=0.6), rounds=10, worker_actually_cheats_fraction=0.0
    )
    assert result.worker_net_pnl == pytest.approx(10 * 10.0 * 0.6)
    assert result.treasury_net_intake == pytest.approx(10 * 10.0 * 0.4)
    # Every forfeited bond is fully accounted for between the two recipients.
    assert result.worker_net_pnl + result.treasury_net_intake == pytest.approx(10 * 10.0)


def test_legitimate_challenging_of_an_always_cheating_worker_is_profitable_for_the_challenger():
    """Control case: this is NOT griefing -- the challenger has a genuine
    informational edge (the worker really does cheat every round) and the
    mechanism correctly rewards them for it, proving the bond-forfeiture
    cost falls on frivolous disputes specifically, not on disputing in
    general."""
    result = simulate_dispute_campaign(
        default_config(), rounds=20, worker_actually_cheats_fraction=1.0
    )
    assert result.challenger_net_pnl > 0
    assert result.challenger_net_pnl == pytest.approx(20 * 15.0)
    assert result.worker_net_pnl == pytest.approx(-20 * 20.0)


def test_mixed_campaign_nets_out_between_the_two_pure_cases():
    """A challenger who is right half the time lands strictly between the
    all-griefing and all-legitimate outcomes -- the model responds
    continuously to how often the challenger is actually correct, not just
    at the two extremes."""
    pure_grief = simulate_dispute_campaign(default_config(), rounds=20, worker_actually_cheats_fraction=0.0)
    pure_legit = simulate_dispute_campaign(default_config(), rounds=20, worker_actually_cheats_fraction=1.0)
    mixed = simulate_dispute_campaign(default_config(), rounds=20, worker_actually_cheats_fraction=0.5)

    assert pure_grief.challenger_net_pnl < mixed.challenger_net_pnl < pure_legit.challenger_net_pnl
