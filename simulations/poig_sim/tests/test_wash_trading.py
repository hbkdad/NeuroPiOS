"""Attack scenario from docs/security/ECONOMIC-ATTACKS.md: 'Fake demand /
wash activity' -- coordinated actors (here: two sock-puppet identities
controlled by the same attacker, one always the requester, one always the
worker) create real-looking job flow purely by recycling their own
capital, to see whether the reward mechanism itself can be net-profited
from by wash trading, as opposed to merely faking activity/reputation
(which docs/security/ECONOMIC-ATTACKS.md's Sybil row already covers).

This does NOT re-test the self-dealing gate (same identity as requester
and worker, covered by test_self_dealing.py) -- these are two genuinely
DISTINCT identities, satisfying UsefulWorkScore's distinct-identity check
on every round, which is exactly what makes wash trading a different
attack from self-dealing.

Finding: the multiplicative reward formula in docs/POIG-SPEC.md
structurally bounds cumulative reward earned to strictly less than
cumulative capital escrowed, across every round tested, even at the
attacker's most favorable settings (escrow at 2x base_value to max out
DemandFactor's cap, L4 verification passing every time, and reputation
allowed to mature toward its ceiling over many rounds). The asymptotic
reward/escrow ratio approaches VerificationConfidence's L4 value (0.85)
times DemandFactor's normalized contribution, never reaching or exceeding
1.0 -- so pure capital-recycling wash trading is a losing proposition
under the reward formula itself. This does NOT mean wash trading is
harmless (it can still be used to cheaply fabricate apparent network
activity/volume for optics, which is a different concern the mitigation
column already flags as needing graph analysis), but it does mean the
formula's multiplicative structure -- not graph analysis -- is what
prevents the ring from directly profiting via reward extraction.
"""
from poig_sim.market import Market
from poig_sim.protocol import Identity


def _run_wash_trading_ring(rounds: int, base_value: float, escrow_multiplier: float) -> tuple[float, float]:
    """A persistent worker identity (building reputation over time) paid
    by a fresh distinct requester identity each round, escrow set to
    `escrow_multiplier * base_value` (2.0 maxes out DemandFactor's cap).
    Returns (cumulative_escrow, cumulative_reward)."""
    market = Market()
    worker_id = "wash-worker"
    escrow_amount = escrow_multiplier * base_value

    cumulative_escrow = 0.0
    cumulative_reward = 0.0
    for i in range(rounds):
        requester = Identity.create(f"wash-requester-{i}")
        outcome = market.submit_and_settle_job(
            requester=requester,
            worker_id=worker_id,
            job_type="inference",
            base_value=base_value,
            escrow_amount=escrow_amount,
            verification_policy="L4",
            nonce=0,
            fund_escrow=True,
            verification_passed=True,
        )
        cumulative_escrow += escrow_amount
        cumulative_reward += outcome.reward.reward

    return cumulative_escrow, cumulative_reward


def test_wash_trading_never_nets_more_reward_than_capital_committed():
    cumulative_escrow, cumulative_reward = _run_wash_trading_ring(
        rounds=30, base_value=10.0, escrow_multiplier=2.0  # 2x maxes DemandFactor's cap
    )

    assert cumulative_reward < cumulative_escrow
    # Well below, not just marginally -- confirms this isn't a near-miss.
    assert cumulative_reward / cumulative_escrow < 0.6


def test_wash_trading_ratio_stays_bounded_even_as_reputation_matures():
    """Runs long enough for the worker's reputation to approach its
    ceiling (asymptotic growth per reputation.py's 0.08-of-remaining-gap
    rule) and confirms the per-round reward/escrow ratio still never
    reaches 1.0 -- the attacker's BEST case (matured reputation, maxed
    DemandFactor, L4 passing every time) is still a structural loss."""
    market = Market()
    worker_id = "wash-worker-long-run"
    base_value = 10.0
    escrow_amount = 20.0  # 2x base_value

    last_ratio = None
    for i in range(200):
        requester = Identity.create(f"wash-requester-long-{i}")
        outcome = market.submit_and_settle_job(
            requester=requester,
            worker_id=worker_id,
            job_type="inference",
            base_value=base_value,
            escrow_amount=escrow_amount,
            verification_policy="L4",
            nonce=0,
            fund_escrow=True,
            verification_passed=True,
        )
        last_ratio = outcome.reward.reward / escrow_amount

    # Reputation is now near its ceiling (200 rounds of 8%-of-remaining-gap
    # growth). Even here, the per-round ratio stays below the L4
    # verification-confidence ceiling (0.85) applied to the capped
    # demand factor -- never at or above 1.0.
    assert last_ratio is not None
    assert last_ratio < 0.9


def test_wash_trading_is_a_genuinely_different_attack_from_self_dealing():
    """Sanity check that this scenario is NOT accidentally hitting the
    self-dealing gate (which would make this an uninteresting duplicate of
    test_self_dealing.py) -- UsefulWorkScore must be 1.0 (distinct
    identities, real gate pass) on every round, with the LOSS coming from
    the OTHER factors in the formula, not from UsefulWorkScore zeroing it."""
    market = Market()
    requester = Identity.create("wash-requester-single")
    outcome = market.submit_and_settle_job(
        requester=requester,
        worker_id="wash-worker-single",  # distinct from requester
        job_type="inference",
        base_value=10.0,
        escrow_amount=20.0,
        verification_policy="L4",
        nonce=0,
        fund_escrow=True,
        verification_passed=True,
    )
    assert outcome.reward.useful_work_score == 1.0
    assert outcome.reward.reward > 0.0  # not zeroed like self-dealing would be
