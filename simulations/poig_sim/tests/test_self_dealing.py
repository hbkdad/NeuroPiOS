"""Attack scenario from docs/security/ECONOMIC-ATTACKS.md: 'Self-dealing jobs' /
'arithmetic-only mining'. An actor creates jobs paid to itself (or to a
worker identity it also controls) with no real independent external demand,
and tries to farm PoIG reward.

Assertion: UsefulWorkScore (and therefore Reward) stays at exactly 0,
regardless of how much escrow the actor funds itself, because
UsefulWorkScore's self-dealing check does not depend on the escrow amount.
"""
from poig_sim.market import Market
from poig_sim.protocol import Identity


def test_self_dealing_same_identity_as_requester_and_worker_earns_zero():
    market = Market()
    attacker = Identity.create("attacker")

    outcome = market.submit_and_settle_job(
        requester=attacker,
        worker_id=attacker.identity_id,  # same identity as requester
        job_type="inference",
        base_value=10.0,
        escrow_amount=1_000_000.0,  # attacker tries to fund itself lavishly
        verification_policy="L4",
        nonce=0,
        fund_escrow=True,
        verification_passed=True,
    )

    assert outcome.reward.useful_work_score == 0.0
    assert outcome.reward.reward == 0.0
    assert "self-dealing" in outcome.reward.useful_work_reason


def test_self_dealing_via_sock_puppet_requester_still_requires_real_escrow():
    """A more subtle variant: the attacker creates a *different-looking*
    requester identity (a sock puppet), but never actually funds escrow
    (because funding would require real capital from a real counterparty).
    This should also earn zero, via the escrow-funded check rather than the
    same-identity check -- demonstrating the gate has two independent legs,
    not just one."""
    market = Market()
    sock_puppet_requester = Identity.create("sock-puppet")
    worker_id = "attacker-worker"

    outcome = market.submit_and_settle_job(
        requester=sock_puppet_requester,
        worker_id=worker_id,
        job_type="inference",
        base_value=10.0,
        escrow_amount=10.0,
        verification_policy="L4",
        nonce=0,
        fund_escrow=False,  # claims escrow but never actually funds it
        verification_passed=True,
    )

    assert outcome.reward.useful_work_score == 0.0
    assert outcome.reward.reward == 0.0


def test_repeated_self_dealing_attempts_never_accumulate_reward():
    """Confirms the gate holds across many attempts, not just the first one --
    an attacker retrying the same strategy should never eventually 'get lucky'."""
    market = Market()
    attacker = Identity.create("persistent-attacker")

    total_reward = 0.0
    for i in range(50):
        outcome = market.submit_and_settle_job(
            requester=attacker,
            worker_id=attacker.identity_id,
            job_type="inference",
            base_value=10.0,
            escrow_amount=10.0,
            verification_policy="L4",
            nonce=i,
            fund_escrow=True,
            verification_passed=True,
        )
        total_reward += outcome.reward.reward

    assert total_reward == 0.0
