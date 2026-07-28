"""Sanity baseline: an honest, distinct-requester, escrow-backed, verified job
should earn a positive reward. This is the control case the attack tests
below are contrasted against."""
from poig_sim.market import Market
from poig_sim.protocol import Identity


def test_honest_verified_job_earns_positive_reward():
    market = Market()
    requester = Identity.create("requester-alice")
    worker_id = "worker-bob"

    outcome = market.submit_and_settle_job(
        requester=requester,
        worker_id=worker_id,
        job_type="inference",
        base_value=10.0,
        escrow_amount=10.0,
        verification_policy="L4",
        nonce=0,
        fund_escrow=True,
        verification_passed=True,
    )

    assert outcome.reward.useful_work_score == 1.0
    assert outcome.reward.reward > 0.0


def test_unfunded_job_earns_nothing_even_if_verified():
    market = Market()
    requester = Identity.create("requester-alice")

    outcome = market.submit_and_settle_job(
        requester=requester,
        worker_id="worker-bob",
        job_type="inference",
        base_value=10.0,
        escrow_amount=10.0,
        verification_policy="L4",
        nonce=0,
        fund_escrow=False,  # never actually funded
        verification_passed=True,
    )

    assert outcome.reward.useful_work_score == 0.0
    assert outcome.reward.reward == 0.0


def test_replayed_nonce_is_rejected():
    from poig_sim.protocol import Job, NonceTracker, ReplayError, create_job, verify_job

    requester = Identity.create("requester-alice")
    nonces = NonceTracker()
    job1 = create_job(requester, "inference", 10.0, 10.0, "L4", nonce=5)
    verify_job(job1, requester, nonces)  # first use of nonce 5 is fine

    job2 = create_job(requester, "inference", 10.0, 10.0, "L4", nonce=5)  # replay
    try:
        verify_job(job2, requester, nonces)
        assert False, "expected ReplayError"
    except ReplayError:
        pass
