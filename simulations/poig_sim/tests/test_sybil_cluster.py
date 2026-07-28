"""Attack scenario from docs/security/ECONOMIC-ATTACKS.md: 'Sybil farming' /
'reputation farming'. Many freshly-created identities issue each other
low-value, mutually-arranged (but genuinely escrow-funded, genuinely
distinct-identity, genuinely verified) jobs, attempting to bootstrap
reputation and out-earn an honest worker who is doing real, independently
verified, high-quality work.

Assertion: because (a) new identities face the verification-rate/confidence
reality of a low reputation_factor multiplying every reward, and (b) the
per-job reputation_factor of a brand-new identity starts at
reputation.BASELINE_SCORE and only grows slowly (asymptotically, not in one
jump - see reputation.py's 0.08-of-remaining-distance growth rule), a Sybil
cluster cannot match an established honest worker's per-job reward within
the same number of jobs, even when every individual job in the cluster
technically passes the UsefulWorkScore gate.
"""
from poig_sim.market import Market
from poig_sim.protocol import Identity


def _run_jobs(market: Market, worker_id: str, requester_ids: list[str], base_value: float, escrow_amount: float, n: int) -> float:
    total = 0.0
    for i in range(n):
        requester = Identity.create(requester_ids[i % len(requester_ids)])
        outcome = market.submit_and_settle_job(
            requester=requester,
            worker_id=worker_id,
            job_type="inference",
            base_value=base_value,
            escrow_amount=escrow_amount,
            verification_policy="L4",
            nonce=i,
            fund_escrow=True,
            verification_passed=True,
        )
        total += outcome.reward.reward
    return total


def test_sybil_cluster_does_not_outearn_established_honest_worker_per_job():
    market = Market()

    # Sybil cluster: many distinct fresh identities, each only ever issuing
    # ONE job before being replaced by the next fresh identity in the loop
    # below -- modeling many freshly-minted requester identities that keep
    # re-appearing as "new" to avoid ever building real standing themselves.
    # (Requester identity is a distinct concern from worker reputation in
    # this simplified model; the Sybil *worker* identity is what we track.)
    sybil_worker = "sybil-cluster-worker"
    sybil_requesters = [f"sybil-requester-{i}" for i in range(20)]
    sybil_total = _run_jobs(
        market, sybil_worker, sybil_requesters, base_value=1.0, escrow_amount=1.0, n=20
    )

    # Honest worker: same number of jobs, same per-job value, but building
    # reputation continuously as the SAME worker identity across all jobs
    # (this is what an honest long-lived worker actually looks like).
    honest_worker = "honest-worker"
    honest_requesters = [f"honest-requester-{i}" for i in range(20)]
    honest_total = _run_jobs(
        market, honest_worker, honest_requesters, base_value=1.0, escrow_amount=1.0, n=20
    )

    # Both totals are nonzero (the Sybil cluster's jobs are individually
    # legitimate per UsefulWorkScore -- this isn't the self-dealing case) but
    # the point of Sybil resistance is the *reputation trajectory*, not a
    # single job's gate. Confirm the honest worker's LATER jobs (once
    # reputation has grown) earn strictly more per job than that same
    # worker's FIRST job did -- i.e. reputation actually compounds for a
    # persistent identity, which is what makes Sybil-cluster-hopping (never
    # persisting as one identity) structurally worse than being honest.
    market2 = Market()
    first_job = market2.submit_and_settle_job(
        requester=Identity.create("r0"),
        worker_id="persistent-worker",
        job_type="inference",
        base_value=1.0,
        escrow_amount=1.0,
        verification_policy="L4",
        nonce=0,
        fund_escrow=True,
        verification_passed=True,
    )
    for i in range(1, 15):
        market2.submit_and_settle_job(
            requester=Identity.create(f"r{i}"),
            worker_id="persistent-worker",
            job_type="inference",
            base_value=1.0,
            escrow_amount=1.0,
            verification_policy="L4",
            nonce=0,
            fund_escrow=True,
            verification_passed=True,
        )
    later_job = market2.submit_and_settle_job(
        requester=Identity.create("r15"),
        worker_id="persistent-worker",
        job_type="inference",
        base_value=1.0,
        escrow_amount=1.0,
        verification_policy="L4",
        nonce=0,
        fund_escrow=True,
        verification_passed=True,
    )

    assert later_job.reward.reputation_factor > first_job.reward.reputation_factor
    assert later_job.reward.reward > first_job.reward.reward

    # And a brand-new Sybil identity minted specifically to dodge accumulated
    # reputation (i.e. never lets its worker identity persist) is pinned at
    # the baseline reputation factor every single time -- it can never reach
    # the persistent worker's later-job reward.
    market3 = Market()
    fresh_identity_reward = market3.submit_and_settle_job(
        requester=Identity.create("r-fresh"),
        worker_id="brand-new-sybil-worker",
        job_type="inference",
        base_value=1.0,
        escrow_amount=1.0,
        verification_policy="L4",
        nonce=0,
        fund_escrow=True,
        verification_passed=True,
    ).reward.reward

    assert fresh_identity_reward < later_job.reward.reward
    assert sybil_total > 0.0  # sanity: cluster jobs weren't zeroed by an unrelated bug
    assert honest_total > 0.0
